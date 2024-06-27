use super::error::Result;
use crate::cli::traits::{NormalizePath, ResolveBase};
use crate::database::functions::{crates, tracks};
use futures::StreamExt;
use log::{debug, warn};
use sea_orm::ConnectionTrait;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader, Lines};
use tokio_stream::wrappers::LinesStream;

pub async fn import_path<C: ConnectionTrait>(db: &C, crate_id: i32, path: PathBuf) -> Result<()> {
    debug!(
        r#"Connecting tracks from "{}" to crate id [{crate_id}]"#,
        path.to_string_lossy()
    );
    LinesStream::new(read_lines(&path).await?)
        .for_each_concurrent(None, |line_res| async {
            match line_res {
                Ok(line) => import_line_into(line, &path, crate_id, db).await,
                Err(err) => warn!(
                    "Could not import a line from {}, ran into {err:?}",
                    path.to_string_lossy(),
                ),
            }
        })
        .await;
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
        let source = format!(r#"Could not find "{loc}" in database!"#,);
        let tip = "Try rescanning your library and checking for case sensitivity.";
        warn!("{source} {tip}");
        return;
    };
    let track_id = track.id;
    debug!(r#"Connecting "{loc}" with track_id "{track_id}" to crate id "{crate_id}""#);
    let Err(err) = crates::connect_track(db, crate_id, track_id).await else {
        return;
    };
    warn!(r#"Could not add "{loc}" with track_id "{track_id}" to crate id "{crate_id}": {err:?}"#);
}

pub async fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(filename).await?;
    Ok(BufReader::new(file).lines())
}
