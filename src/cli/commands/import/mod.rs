mod directory;
mod error;
mod playlist;

use crate::cli::traits::ResolveBase;
use crate::cli::{traits::NormalizePath, validators};
use crate::database::functions::crates;
use crate::database::{disable_fk, enable_fk, get_mixxx_directory, get_sqlite_connection};
use clap::Parser;
use error::Error;
use futures::future::try_join_all;
use inquire::error::InquireResult;
use inquire::{CustomUserError, Text};
use log::{debug, info, trace, warn};
use sea_orm::{ConnectionTrait, TransactionTrait};
use std::env::current_dir;
use std::{
    collections::HashMap,
    fs::read_to_string,
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
    trace!("Import command started on {dir}");

    let url: PathBuf = [get_mixxx_directory()?, "mixxxdb.sqlite".into()]
        .into_iter()
        .collect();

    let db = &get_sqlite_connection(&url.to_string_lossy()).await?;
    disable_fk(db).await?;
    let txn = db.begin().await?;

    let crate_map = get_crate_map(&dir)?;
    let base = dir.resolve_base(current_dir()?);
    trace!("Crate map acquired!");
    try_join_all(
        crate_map
            .into_iter()
            .map(|(name, paths)| import_paths(&txn, name, &base, paths)),
    )
    .await?;

    txn.commit().await?;
    enable_fk(db).await?;

    info!("Successfully imported crates");
    Ok(())
}

async fn import_paths<C: ConnectionTrait>(
    db: &C,
    name: String,
    base: impl AsRef<Path>,
    paths: Vec<String>,
) -> Result<(), CustomUserError> {
    let crate_id = crates::get_by_name_or_create(db, &name).await?;
    trace!(r#"Clearing crate "{name}""#);
    clear_crate(db, crate_id).await;
    for path in paths {
        let buf = PathBuf::from(path);
        if buf.is_dir() {
            directory::import(db, crate_id, base.as_ref(), buf).await?;
        } else {
            playlist::import_path(db, crate_id, buf).await?;
        }
    }
    Ok(())
}

fn prompt() -> InquireResult<Option<String>> {
    Text::new("Path to playlists folder:")
        .with_validator(validators::Directory::Required)
        .prompt_skippable()
        .map(|maybe| maybe.map(NormalizePath::normalize_path))
}

async fn clear_crate<C: ConnectionTrait>(db: &C, id: i32) {
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

fn get_crate_map<P: AsRef<Path>>(input: P) -> Result<HashMap<String, Vec<String>>, Error> {
    let buf: PathBuf;
    let path = if input.as_ref().is_dir() {
        buf = input.as_ref().join("mixxxkit.crates.yaml");
        buf.as_ref()
    } else {
        input.as_ref()
    };
    let contents = read_to_string(path)?;
    parse_crate_map(&contents)
}

fn parse_crate_map(str: &str) -> Result<HashMap<String, Vec<String>>, Error> {
    let Ok(docs) = YamlLoader::load_from_str(str) else {
        return Err(Error::ParsingFailed);
    };
    let doc = &docs[0];
    let Some(source) = doc["mappings"].as_hash() else {
        return Err(Error::ParsingFailed);
    };
    let prefix = doc["prefix"].as_str().unwrap_or("");
    let payload = source
        .iter()
        .filter_map(|(key_raw, paths_raw)| {
            let (Some(key), Some(paths)) = (key_raw.as_str(), paths_raw.as_vec()) else {
                return None;
            };
            let vec = paths
                .iter()
                .filter_map(|raw| raw.as_str())
                .map(ToOwned::to_owned)
                .collect();
            let crate_name = prefix.to_owned() + key;
            Some((crate_name, vec))
        })
        .collect();
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn parses_map() {
        let str = indoc! {r#"
            prefix: "[my] "
            mappings:
                fruit:
                    - apple
                    - tomato
                vegetable:
                    - leek
                    - tomato
        "#};
        let map = parse_crate_map(str).unwrap();
        let vec = map.get("[my] fruit").unwrap();
        assert!(["apple", "tomato"]
            .into_iter()
            .all(|subject| vec.contains(&subject.to_string())));
    }
}
