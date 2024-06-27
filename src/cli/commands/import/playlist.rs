use super::error::Result;
use crate::cli::traits::{NormalizePath, ResolveBase};
use crate::database::functions::{crates, tracks};
use log::{debug, warn};
use sea_orm::ConnectionTrait;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader, Lines};

pub async fn import_path<C: ConnectionTrait>(db: &C, crate_id: i32, path: PathBuf) -> Result<()> {
    debug!(
        r#"Connecting tracks from "{}" to crate id [{crate_id}]"#,
        path.to_string_lossy()
    );
    let mut lines = read_lines(&path).await?;
    while let Some(line) = lines.next_line().await? {
        import_line_into(line, &path, crate_id, db).await;
    }
    Ok(())
}

pub async fn import_line_into<C: ConnectionTrait>(
    line: String,
    path: &PathBuf,
    crate_id: i32,
    db: &C,
) {
    let buf = PathBuf::from(&line).resolve_base(path);
    if !buf.exists() {
        return;
    }
    let loc = &buf.normalize_path();
    let Ok(Some(track)) = tracks::get_by_location(db, loc).await else {
        warn!(
            r#"Could not find "{loc}" in database! Try rescanning your library and checking for case sensitivity."#,
        );
        return;
    };
    let track_id = track.id;
    debug!(
        r#"Connecting "{loc}" with track_id "{}" to crate id "{crate_id}""#,
        track_id,
    );
    if let Err(err) = crates::connect_track(db, crate_id, track_id).await {
        warn!(
            r#"Could not add "{loc}" with track_id "{track_id}" to crate id "{crate_id}": {:?}"#,
            err
        );
    };
}

pub async fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(filename).await?;
    Ok(BufReader::new(file).lines())
}
