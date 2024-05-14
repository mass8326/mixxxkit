use crate::cli::{extensions::NormalizePath, validators::directory};
use futures_util::future::join_all;
use inquire::Text;
use mixxxkit::{
    database::{
        disable_fk_constraints, enable_fk_constraints, get_mixxx_directory, get_sqlite_connection,
    },
    queries::crates::{clear_crate_tracks, connect_track_by_location, get_by_name_or_create},
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{read_dir, read_to_string, File},
    io::{self, BufRead, ErrorKind},
    path::{Path, PathBuf},
};
use yaml_rust::YamlLoader;

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let dir = prompt();
    let url: PathBuf = [get_mixxx_directory(), "mixxxdb.sqlite".into()]
        .into_iter()
        .collect();
    let db = &get_sqlite_connection(&url.to_string_lossy()).await?;
    disable_fk_constraints(db).await?;

    let crate_map = get_crate_map(&dir);
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
        let crate_futures = crate_map
            .as_ref()
            .and_then(|map| {
                let vec = map.get(&filestem)?;
                println!(r#"Mapping "{filestem}" to ["{}"]"#, vec.join(r#"", ""#));
                Some(vec.to_owned())
            })
            .unwrap_or_else(|| vec![filestem])
            .into_iter()
            .map(|name| async move {
                let prefixed = "[MixxxKit] ".to_owned() + &name;
                let Ok(id) = get_by_name_or_create(db, &prefixed).await else {
                    return None;
                };
                Some(id)
            });

        let crate_ids: Vec<i32> = join_all(crate_futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        for id in &crate_ids {
            if !cleared_crates.contains(id) {
                cleared_crates.insert(*id);
                match clear_crate_tracks(db, *id).await {
                    Ok(()) => println!(r#"Cleared tracks from crate id "{id}""#),
                    Err(_) => println!(r#"Unable to clear tracks from crate id "{id}"!"#),
                }
            }
        }

        println!(
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
                    db,
                    *crate_id,
                    &line_pathbuf.to_string_lossy().normalize_path(),
                )
                .await?;
            }
        }
    }

    enable_fk_constraints(db).await?;
    Ok(())
}

fn prompt() -> String {
    Text::new("Path to playlists folder:")
        .with_validator(directory::Validator)
        .prompt()
        .unwrap()
        .normalize_path()
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_crate_map<P: AsRef<Path>>(dir: P) -> Option<HashMap<String, Vec<String>>> {
    let path = dir.as_ref().join("mixxxkit.crates.yaml");
    let contents = match read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            return match error.kind() {
                ErrorKind::NotFound => None,
                ErrorKind::PermissionDenied => {
                    panic!("mixxxkit.crates.yaml found but permissions are insufficient to read!")
                }
                _ => {
                    panic!("mixxxkit.crates.yaml found but ran into {error:?}")
                }
            };
        }
    };
    parse_crate_map(&contents).or_else(|| panic!("mixxxkit.crates.yaml found but not parseable!"))
}

fn parse_crate_map(str: &str) -> Option<HashMap<String, Vec<String>>> {
    let Ok(docs) = YamlLoader::load_from_str(str) else {
        return None;
    };
    let forward_map = docs[0]["mappings"].as_hash()?;
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
    Some(reverse_map)
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
