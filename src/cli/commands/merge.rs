use crate::cli::{extensions::NormalizePath, validators};
use crate::database::get_mixxx_database_path;
use crate::database::{
    disable_fk, enable_fk, functions, get_sqlite_connection, schema::directories,
};
use crate::error::MixxxkitExit;
use clap::Parser;
use inquire::validator::{StringValidator, Validation};
use inquire::{Confirm, CustomUserError, Text};
use log::{debug, error, info};
use sea_orm::TransactionTrait;
use std::{collections::HashMap, fs::copy};

#[derive(Parser, Debug, Default)]
pub struct Args {
    /// Source database to pull tracks from. If omitted, you will be prompted for paths.
    pub source: Option<String>,
    /// Target database to merge into. If omitted, your installation database is targeted.
    pub target: Option<String>,
    /// Output database as new file to this location. If omitted, target is edited in place.
    pub output: Option<String>,
    /// Skip all prompts and force execution
    #[arg(short, long)]
    pub force: bool,
}

pub async fn run(args: &Args) -> Result<(), CustomUserError> {
    let paths = prompt_for_databases(args)?;
    if args.target.is_none() && get_mixxx_database_path()?.to_string_lossy() == paths.output {
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
    let dir_map = match args.force {
        false => Some(prompt_for_directories(&dirs)),
        true => None,
    };

    if paths.target != paths.output {
        copy(&paths.target, &paths.output)?;
    }

    let output = &get_sqlite_connection(&paths.output).await?;
    disable_fk(output).await?;
    let txn = output.begin().await?;
    functions::directories::insert(&txn, &dirs, dir_map.as_ref()).await?;

    let locs = functions::locations::get(&source).await?;
    let loc_map = functions::locations::insert(&txn, locs, dir_map.as_ref()).await?;

    let tracks = functions::tracks::get(&source).await?;
    functions::tracks::insert(&txn, tracks, &loc_map).await?;

    txn.commit().await?;
    enable_fk(output).await?;
    info!("Successfully merged libraries");
    Ok(())
}

struct DatabasePaths {
    pub source: String,
    pub target: String,
    pub output: String,
}

fn prompt_for_databases(args: &Args) -> Result<DatabasePaths, CustomUserError> {
    let Some(source) = args.source.as_ref().map_or_else(
        || {
            Some(
                Text::new("Path to source database:")
                    .with_validator(validators::Database::Required)
                    .with_help_message("Enter the database that you want to pull songs from")
                    .prompt()
                    .unwrap()
                    .normalize_path(),
            )
        },
        |path| match validators::Database::Required.validate(path) {
            Ok(Validation::Valid) => Some(path.to_owned()),
            _ => None,
        },
    ) else {
        error!("Source database invalid!");
        return Err(Box::new(MixxxkitExit));
    };
    debug!(r#"Source path set to "{source}""#);

    let Some(target) = match args.source {
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
    .or_else(|| {
        Some(
            get_mixxx_database_path()
                .ok()?
                .to_string_lossy()
                .to_string(),
        )
    }) else {
        return Err(Box::new(MixxxkitExit));
    };
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

    Ok(DatabasePaths {
        source,
        target,
        output,
    })
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
