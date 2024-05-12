use crate::cli::{extensions::NormalizePath, validators::directory};
use inquire::Text;
use mixxxkit::{
    database::{
        disable_fk_constraints, enable_fk_constraints, get_mixxx_directory, get_sqlite_connection,
    },
    queries::crates::{clear_crate_tracks, connect_track_by_location, get_by_name_or_create},
};
use std::{
    error::Error,
    fs::{read_dir, File},
    io::{self, BufRead},
    path::{Path, PathBuf},
};

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let dir = prompt();
    let url: PathBuf = [get_mixxx_directory(), "mixxxdb.sqlite".into()]
        .into_iter()
        .collect();
    let db = &get_sqlite_connection(&url.to_string_lossy()).await?;
    disable_fk_constraints(db).await?;

    for maybe_entry in read_dir(dir)? {
        let Ok(entry) = maybe_entry else {
            continue;
        };
        let path = entry.path();
        if !path.extension().is_some_and(|ext| "m3u8" == ext) {
            continue;
        }
        let crate_name = String::from("[MixxxKit] ") + &path.file_stem().unwrap().to_string_lossy();
        let Ok(lines) = read_lines(&path) else {
            continue;
        };
        let mut crate_id_maybe: Option<i32> = None;
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
            let crate_id = match crate_id_maybe {
                Some(id) => id,
                None => {
                    let id = get_by_name_or_create(db, &crate_name).await?;
                    clear_crate_tracks(db, id).await?;
                    crate_id_maybe = Some(id);
                    id
                }
            };
            connect_track_by_location(
                db,
                crate_id,
                &line_pathbuf.to_string_lossy().normalize_path(),
            )
            .await?;
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
