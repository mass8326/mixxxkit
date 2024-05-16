use crate::cli::{extensions::NormalizePath, validators};
use crate::database::get_mixxx_database_path;
use crate::database::{
    disable_fk_constraints, enable_fk_constraints, functions, get_sqlite_connection,
    schema::directories,
};
use clap::Parser;
use inquire::validator::{StringValidator, Validation};
use inquire::{Confirm, Text};
use log::{debug, error, info};
use std::{collections::HashMap, error::Error, fs::copy};

#[derive(Parser, Debug, Default)]
pub struct Args {
    pub source: Option<String>,
    pub target: Option<String>,
    pub output: Option<String>,
}

pub async fn run(args: &Args) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let paths = prompt_for_databases(args);
    if args.target.is_none() && get_mixxx_database_path().to_string_lossy() == paths.output {
        let check =
            Confirm::new("You are going to edit your Mixxx database in-place. Are you sure? (y/n)")
                .with_help_message("Please make a backup of your database before continuing!")
                .prompt_skippable()
                .unwrap();
        if !check.is_some_and(|b| b) {
            return Ok(());
        };
    }

    let source = get_sqlite_connection(&paths.source).await?;
    let dirs = functions::directories::get(&source).await?;
    let dir_map = prompt_for_directories(&dirs);

    if paths.target != paths.output {
        copy(&paths.target, &paths.output)?;
    }

    let output = get_sqlite_connection(&paths.output).await?;
    disable_fk_constraints(&output).await?;
    functions::directories::insert(&output, &dirs, Some(&dir_map)).await?;

    let locs = functions::locations::get(&source).await?;
    let loc_map = functions::locations::insert(&output, locs, Some(&dir_map)).await?;

    let tracks = functions::tracks::get(&source).await?;
    functions::tracks::insert(&output, tracks, &loc_map).await?;

    enable_fk_constraints(&output).await?;
    info!("Successfully merged libraries");
    Ok(())
}

struct DatabasePaths {
    pub source: String,
    pub target: String,
    pub output: String,
}

fn prompt_for_databases(args: &Args) -> DatabasePaths {
    let source = args.source.as_ref().map_or_else(
        || {
            Text::new("Path to source database:")
                .with_validator(validators::Database::Required)
                .with_help_message("Enter the database that you want to pull songs from")
                .prompt()
                .unwrap()
                .normalize_path()
        },
        |path| {
            let Ok(Validation::Valid) = validators::Database::Required.validate(path) else {
                error!("Source database invalid!");
                std::process::exit(1);
            };
            path.to_owned()
        },
    );
    debug!(r#"Source path set to "{source}""#);

    let target = match args.source {
        None => {
            let result = Text::new("Path to target database:")
                .with_help_message("Leave blank to target your current Mixxx database")
                .with_validator(validators::Database::Optional)
                .prompt()
                .unwrap()
                .normalize_path();
            match result.is_empty() {
                false => Some(result),
                true => None,
            }
        }
        Some(_) => None,
    }
    .unwrap_or_else(|| get_mixxx_database_path().to_string_lossy().to_string());
    debug!(r#"Target path set to "{target}""#);

    let output = match args.source {
        None => {
            let result = Text::new("Path to output database:")
                .with_help_message(
                    "Leave blank to edit the target in place, enter a path to output a new file",
                )
                .with_validator(validators::Target::OptionalDirExists)
                .prompt()
                .unwrap()
                .normalize_path();
            match result.is_empty() {
                false => Some(result),
                true => None,
            }
        }
        Some(_) => None,
    }
    .unwrap_or_else(|| target.clone());
    debug!(r#"Output path set to "{output}""#);

    DatabasePaths {
        source,
        target,
        output,
    }
}

fn prompt_for_directories(dirs: &[directories::Model]) -> HashMap<String, String> {
    let mut map = HashMap::<String, String>::with_capacity(dirs.len());
    for dir in dirs {
        let original = &dir.directory;
        let replacement = Text::new(&format!(r#"New location of "{original}":"#))
            .with_validator(validators::Directory::Optional)
            .with_help_message("Leave blank to keep tracks in original location")
            .prompt()
            .unwrap()
            .trim()
            .replace('\\', "/")
            .normalize_path();
        if !replacement.is_empty() {
            debug!(r#"Mapping {original} to "{replacement}""#);
            map.insert(dir.directory.clone(), replacement);
        }
    }
    map
}
