use crate::cli::{extensions::NormalizePath, validators};
use crate::database::{
    begin_transaction, commit_transaction,
    functions::crates::{clear_crate_tracks, connect_track_by_location, get_by_name_or_create},
    get_mixxx_directory, get_sqlite_connection,
};
use crate::error::MixxxkitExit;
use inquire::error::InquireResult;
use inquire::{CustomUserError, Text};
use log::{error, info, trace, warn};
use std::{
    collections::{HashMap, HashSet},
    fs::{read_dir, read_to_string, File},
    io::{self, BufRead, ErrorKind},
    path::{Path, PathBuf},
};
use yaml_rust::YamlLoader;

pub async fn run() -> Result<(), CustomUserError> {
    let Some(dir) = prompt()? else {
        return Ok(());
    };
    let url: PathBuf = [get_mixxx_directory()?, "mixxxdb.sqlite".into()]
        .into_iter()
        .collect();

    let db = &get_sqlite_connection(&url.to_string_lossy()).await?;
    let txn = begin_transaction(db).await?;

    let crate_map = get_crate_map(&dir)?;
    let mut cleared_crates: HashSet<i32> = HashSet::new();
    for maybe_entry in read_dir(dir)? {
        let Ok(entry) = maybe_entry else {
            continue;
        };
        let path = entry.path();
        if !path.extension().is_some_and(|ext| "m3u8" == ext) {
            continue;
        }
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
            if let Ok(id) = get_by_name_or_create(&txn, &prefixed).await {
                crate_ids.push(id);
            };
        }

        for id in &crate_ids {
            if !cleared_crates.contains(id) {
                cleared_crates.insert(*id);
                match clear_crate_tracks(&txn, *id).await {
                    Ok(()) => trace!(r#"Cleared tracks from crate id "{id}""#),
                    Err(_) => warn!(r#"Unable to clear tracks from crate id "{id}""#),
                }
            }
        }

        trace!(
            r#"Connecting tracks to crate ids ["{}"]"#,
            crate_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(r#"", ""#)
        );

        for maybe_line in lines {
            let Ok(line) = maybe_line else {
                continue;
            };
            let mut line_pathbuf = PathBuf::from(&line);
            if line_pathbuf.is_absolute() {
                line_pathbuf = [&path, &line_pathbuf].iter().collect();
            };
            if !line_pathbuf.try_exists().is_ok_and(|bool| bool) {
                continue;
            }
            for crate_id in &crate_ids {
                connect_track_by_location(
                    &txn,
                    *crate_id,
                    &line_pathbuf.to_string_lossy().normalize_path(),
                )
                .await?;
            }
        }
    }

    commit_transaction(txn).await?;
    info!("Successfully imported crates");
    Ok(())
}

fn prompt() -> InquireResult<Option<String>> {
    Text::new("Path to playlists folder:")
        .with_validator(validators::Directory::Required)
        .prompt_skippable()
        .map(|maybe| maybe.map(NormalizePath::normalize_path))
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
                    return Err(Box::new(MixxxkitExit));
                }
                _ => {
                    error!("mixxxkit.crates.yaml found but ran into {err:?}");
                    return Err(Box::new(MixxxkitExit));
                }
            };
        }
    };
    Ok(Some(parse_crate_map(&contents)?))
}

fn parse_crate_map(str: &str) -> Result<HashMap<String, Vec<String>>, CustomUserError> {
    let Ok(docs) = YamlLoader::load_from_str(str) else {
        error!("mixxxkit.crates.yaml found but not parseable");
        return Err(Box::new(MixxxkitExit));
    };
    let Some(forward_map) = docs[0]["mappings"].as_hash() else {
        error!("mixxxkit.crates.yaml found but not parseable");
        return Err(Box::new(MixxxkitExit));
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
