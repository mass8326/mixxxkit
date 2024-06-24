use crate::cli::{extensions::NormalizePath, validators};
use crate::database::functions::{crates, tracks};
use crate::database::{disable_fk, enable_fk, get_mixxx_directory, get_sqlite_connection};
use crate::error::MixxxkitExit;
use clap::Parser;
use inquire::error::InquireResult;
use inquire::{CustomUserError, Text};
use log::{debug, error, info, trace, warn};
use sea_orm::{ConnectionTrait, TransactionTrait};
use std::io::Error;
use std::{
    collections::{HashMap, HashSet},
    fs::{read_dir, read_to_string, File},
    io::{self, BufRead, ErrorKind},
    path::{Path, PathBuf},
};
use yaml_rust::YamlLoader;

#[derive(Parser, Debug, Default)]
pub struct Args {
    pub path: Option<String>,
}

pub async fn run(args: &Args) -> Result<(), CustomUserError> {
    let path_maybe = match &args.path {
        Some(input) => Some(input.to_owned()),
        None => prompt()?,
    };
    let Some(dir) = path_maybe else {
        return Ok(());
    };

    let url: PathBuf = [get_mixxx_directory()?, "mixxxdb.sqlite".into()]
        .into_iter()
        .collect();
    let db = &get_sqlite_connection(&url.to_string_lossy()).await?;
    disable_fk(db).await?;
    let txn = db.begin().await?;

    let crate_map = get_crate_map(&dir)?;
    let mut cleared_crates: HashSet<i32> = HashSet::new();

    let paths = read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| "m3u8" == ext));
    for path in paths {
        let Ok(lines) = read_lines(&path) else {
            continue;
        };

        let filestem = path.file_stem().unwrap().to_string_lossy().to_string();
        let crate_names = crate_map
            .as_ref()
            .and_then(|map| {
                let vec = map.get(&filestem)?;
                trace!(r#"Mapping "{filestem}" to ["{}"]"#, vec.join(r#"", ""#));
                Some(vec.to_owned())
            })
            .unwrap_or_else(|| vec![filestem])
            .into_iter();
        let mut crate_ids: Vec<i32> = Vec::with_capacity(crate_names.len());
        for name in crate_names {
            let prefixed = "[MixxxKit] ".to_owned() + &name;
            if let Ok(id) = crates::get_by_name_or_create(&txn, &prefixed).await {
                crate_ids.push(id);
            };
        }

        debug!(r#"Clearing crates for "{}"#, path.to_string_lossy());
        for id in &crate_ids {
            clear_crate(*id, &mut cleared_crates, &txn).await;
        }

        debug!(
            r#"Connecting tracks from "{}" to crate ids [{}]"#,
            path.to_string_lossy(),
            crate_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(r#"", ""#)
        );
        for maybe_line in lines {
            import_line_into(maybe_line, &path, &crate_ids, &txn).await;
        }
    }

    txn.commit().await?;
    enable_fk(db).await?;
    info!("Successfully imported crates");
    Ok(())
}

fn prompt() -> InquireResult<Option<String>> {
    Text::new("Path to playlists folder:")
        .with_validator(validators::Directory::Required)
        .prompt_skippable()
        .map(|maybe| maybe.map(NormalizePath::normalize_path))
}
async fn clear_crate<C: ConnectionTrait>(id: i32, cleared_crates: &mut HashSet<i32>, db: &C) {
    if cleared_crates.contains(&id) {
        return;
    }
    cleared_crates.insert(id);
    let Err(err) = crates::clear_tracks(db, id).await else {
        debug!(r#"Cleared tracks from crate id "{id}""#);
        return;
    };
    let name = crates::get_by_id(db, id)
        .await
        .ok()
        .flatten()
        .map_or("<N/A>".to_owned(), |found| found.name);
    warn!(r#"Unable to clear tracks from crate [{id}, "{name}"]: {err:?}"#,);
}

async fn import_line_into<C: ConnectionTrait>(
    maybe_line: Result<String, Error>,
    path: &PathBuf,
    crate_ids: &Vec<i32>,
    db: &C,
) {
    let Ok(line) = maybe_line else {
        return;
    };
    let mut line_pathbuf = PathBuf::from(&line);
    if line_pathbuf.is_absolute() {
        line_pathbuf = [path, &line_pathbuf].iter().collect();
    };
    if !line_pathbuf.try_exists().is_ok_and(|bool| bool) {
        return;
    }
    for crate_id in crate_ids {
        let location_path = &line_pathbuf.to_string_lossy().normalize_path();
        let Ok(Some(track)) = tracks::get_by_location(db, location_path).await else {
            warn!(r#"Could not find track location "{location_path}""#,);
            continue;
        };
        let track_id = track.id;
        debug!(
            r#"Connecting "{location_path}" with track_id "{}" to crate id "{crate_id}""#,
            track_id,
        );
        if let Err(err) = crates::connect_track(db, *crate_id, track_id).await {
            warn!(
                r#"Could not add "{location_path}" with track_id "{}" to crate id "{crate_id}": {:?}"#,
                track_id, err
            );
        };
    }
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_crate_map<P: AsRef<Path>>(
    dir: P,
) -> Result<Option<HashMap<String, Vec<String>>>, CustomUserError> {
    let path = dir.as_ref().join("mixxxkit.crates.yaml");
    let contents = match read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            return match err.kind() {
                ErrorKind::NotFound => Ok(None),
                ErrorKind::PermissionDenied => {
                    error!("mixxxkit.crates.yaml found but permissions are insufficient to read");
                    return Err(Box::new(MixxxkitExit::Abort));
                }
                _ => {
                    error!("mixxxkit.crates.yaml found but ran into {err:?}");
                    return Err(Box::new(MixxxkitExit::Abort));
                }
            };
        }
    };
    Ok(Some(parse_crate_map(&contents)?))
}

fn parse_crate_map(str: &str) -> Result<HashMap<String, Vec<String>>, CustomUserError> {
    let Ok(docs) = YamlLoader::load_from_str(str) else {
        error!("mixxxkit.crates.yaml found but not parseable");
        return Err(Box::new(MixxxkitExit::Abort));
    };
    let Some(forward_map) = docs[0]["mappings"].as_hash() else {
        error!("mixxxkit.crates.yaml found but not parseable");
        return Err(Box::new(MixxxkitExit::Abort));
    };
    let mut reverse_map: HashMap<String, Vec<String>> = HashMap::new();
    for (key_raw, arr_raw) in forward_map {
        let (Some(key), Some(arr)) = (key_raw.as_str(), arr_raw.as_vec()) else {
            continue;
        };
        for val_raw in arr {
            let Some(val) = val_raw.as_str() else {
                continue;
            };
            let vec = reverse_map.entry(val.to_owned()).or_default();
            vec.push(key.to_owned());
        }
    }
    Ok(reverse_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn parses_map() {
        let str = indoc! {"
            mappings:
                fruit:
                    - apple
                    - tomato
                vegetable:
                    - leek
                    - tomato
        "};
        let map = parse_crate_map(str).unwrap();
        let vec = map.get("tomato").unwrap();
        assert!(["fruit", "vegetable"]
            .into_iter()
            .all(|subject| vec.contains(&subject.to_string())));
    }
}
