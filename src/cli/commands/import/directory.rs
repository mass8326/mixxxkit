use crate::cli::traits::{NormalizePath, ResolveBase};
use crate::database::functions::crates;
use crate::database::functions::tracks::get_by_location;
use log::warn;
use sea_orm::ConnectionTrait;
use std::path::{Path, PathBuf};
use tokio::fs::read_dir;

use super::error::Result;

pub async fn import_path(
    db: &impl ConnectionTrait,
    crate_id: i32,
    base: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<()> {
    let mut entries = read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let buf = entry.path();
        if !buf.is_supported_audio_ext() {
            continue;
        }
        let loc = buf.resolve_base(base.as_ref()).normalize_path();
        let Some(track) = get_by_location(db, &loc).await? else {
            warn!(
                r#"Could not find "{loc}" in database! Try rescanning your library and checking for case sensitivity."#,
            );
            continue;
        };
        let track_id = track.id;
        if let Err(err) = crates::connect_track(db, crate_id, track_id).await {
            warn!(
                r#"Could not add "{loc}" with track_id "{track_id}" to crate id "{crate_id}": {err:?}"#,
            );
        };
    }
    Ok(())
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
