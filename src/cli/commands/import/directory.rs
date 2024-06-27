use crate::cli::traits::{NormalizePath, ResolveBase};
use crate::database::functions::crates;
use crate::database::functions::tracks::get_by_location;
use futures::StreamExt;
use log::warn;
use sea_orm::ConnectionTrait;
use std::path::{Path, PathBuf};
use tokio::fs::{read_dir, DirEntry};
use tokio::io;
use tokio_stream::wrappers::ReadDirStream;

use super::error::Result;

pub async fn import(
    db: &impl ConnectionTrait,
    crate_id: i32,
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<()> {
    let stream = ReadDirStream::new(read_dir(&path).await?);
    stream
        .for_each_concurrent(None, |entry| {
            import_entry(db, entry, crate_id, base.as_ref(), path.as_ref())
        })
        .await;
    Ok(())
}

async fn import_entry(
    db: &impl ConnectionTrait,
    entry: io::Result<DirEntry>,
    crate_id: i32,
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) {
    if entry.is_err() {
        let err = entry.unwrap_err();
        warn!(
            "Could not read an item in {}: {err:?}",
            path.as_ref().to_string_lossy(),
        );
        return;
    }
    let buf = entry.unwrap().path();
    if !buf.is_supported_audio_ext() {
        return;
    }
    let loc = buf.resolve_base(base.as_ref()).normalize_path();
    let track_res = get_by_location(db, &loc).await;
    let Ok(track_opt) = track_res else {
        warn!(r#"Could not retrieve "{}" from database!"#, loc);
        return;
    };
    let Some(track) = track_opt else {
        let source = format!(r#"Could not find "{loc}" in database!"#);
        let tip = "Try rescanning your library and checking for case sensitivity.";
        warn!("{source} {tip}");
        return;
    };
    let track_id = track.id;
    let Err(err) = crates::connect_track(db, crate_id, track_id).await else {
        return;
    };
    warn!(r#"Could not add "{loc}" with track_id "{track_id}" to crate id "{crate_id}": {err:?}"#,);
}

const SUPPORTED_AUDIO_EXTS: [&str; 8] = ["wav", "aiff", "aif", "mp3", "ogg", "flac", "aac", "m4a"];

trait IsSupportedAudioExt {
    fn is_supported_audio_ext(&self) -> bool;
}

impl IsSupportedAudioExt for PathBuf {
    fn is_supported_audio_ext(&self) -> bool {
        let Some(ext) = self.extension() else {
            return false;
        };
        SUPPORTED_AUDIO_EXTS
            .into_iter()
            .any(|supported| ext == supported)
    }
}
