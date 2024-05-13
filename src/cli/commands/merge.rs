use crate::cli::{
    extensions::NormalizePath,
    validators::{database, directory},
};
use inquire::Text;
use mixxxkit::{
    database::{disable_fk_constraints, enable_fk_constraints, get_sqlite_connection},
    entities::directories,
    queries,
};
use std::{collections::HashMap, error::Error, fs::copy};
use tokio::try_join;

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let paths = prompt_for_databases();

    copy(&paths.target, &paths.output)?;

    let (source, output) = try_join!(
        get_sqlite_connection(&paths.source),
        get_sqlite_connection(&paths.output),
    )?;

    let dirs = queries::directories::get(&source).await?;
    let dir_map = prompt_for_directories(&dirs);
    queries::directories::insert(&output, &dirs, Some(&dir_map)).await?;

    disable_fk_constraints(&output).await?;

    let locs = queries::locations::get(&source).await?;
    let loc_map = queries::locations::insert(&output, locs, Some(&dir_map)).await?;

    let tracks = queries::tracks::get(&source).await?;
    queries::tracks::insert(&output, tracks, &loc_map).await?;

    enable_fk_constraints(&output).await?;

    Ok(())
}

struct DatabasePaths {
    pub source: String,
    pub target: String,
    pub output: String,
}

fn prompt_for_databases() -> DatabasePaths {
    let source = Text::new("Path to source database:")
        .with_validator(database::Validator)
        .with_default("source.sqlite")
        .prompt()
        .unwrap()
        .normalize_path();
    println!(r#"Normalized to "{source}""#);

    let target = Text::new("Path to target database:")
        .with_validator(database::Validator)
        .with_default("target.sqlite")
        .prompt()
        .unwrap()
        .normalize_path();
    println!(r#"Normalized to "{target}""#);

    let output = Text::new("Path to output database:")
        .with_default("mixxxdb.sqlite")
        .prompt()
        .unwrap()
        .normalize_path();
    println!(r#"Normalized to "{output}""#);

    DatabasePaths {
        source,
        target,
        output,
    }
}

fn prompt_for_directories(dirs: &[directories::Model]) -> HashMap<String, String> {
    let mut map = HashMap::<String, String>::with_capacity(dirs.len());
    for dir in dirs {
        let path = Text::new(&format!(r#"Replacement path for "{}":"#, &dir.directory))
            .with_validator(directory::Validator)
            .prompt()
            .unwrap()
            .trim()
            .replace('\\', "/")
            .normalize_path();
        println!(r#"Normalized to "{path}""#);
        if !path.is_empty() {
            map.insert(dir.directory.clone(), path);
        }
    }
    map
}
