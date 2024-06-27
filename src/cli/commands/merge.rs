use crate::cli::{traits::NormalizePath, validators};
use crate::database::get_mixxx_database_path;
use crate::database::{
    disable_fk, enable_fk, functions, get_sqlite_connection, schema::directories,
};
use crate::error::MixxxkitExit;
use clap::Parser;
use inquire::validator::StringValidator;
use inquire::{Confirm, CustomUserError, Text};
use log::{debug, info};
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

    let source_db = get_sqlite_connection(&paths.source).await?;
    let dirs = functions::directories::get(&source_db).await?;
    let dir_map = match args.force {
        false => Some(prompt_for_directories(&dirs)),
        true => None,
    };

    let output_path = match (paths.target, paths.output) {
        (Some(target), Some(output)) => {
            if target != output {
                copy(target, &output)?;
            }
            output
        }
        (Some(target), None) => target,
        _ => get_mixxx_database_path()?.to_string_lossy().to_string(),
    };
    let output_db = &get_sqlite_connection(&output_path).await?;

    disable_fk(output_db).await?;
    let txn = output_db.begin().await?;

    functions::directories::insert(&txn, &dirs, dir_map.as_ref()).await?;

    let locs = functions::locations::get(&source_db).await?;
    let loc_map = functions::locations::insert(&txn, locs, dir_map.as_ref()).await?;

    let tracks = functions::tracks::get(&source_db).await?;
    functions::tracks::insert(&txn, tracks, &loc_map).await?;

    txn.commit().await?;
    enable_fk(output_db).await?;

    info!("Successfully merged libraries");
    Ok(())
}

struct DatabasePaths {
    pub source: String,
    pub target: Option<String>,
    pub output: Option<String>,
}

fn prompt_for_databases(args: &Args) -> Result<DatabasePaths, CustomUserError> {
    let source = match &args.source {
        Some(path) => {
            validators::Database::Required.validate(path)?;
            path.clone()
        }
        None => Text::new("Path to source database:")
            .with_validator(validators::Database::Required)
            .with_help_message("Enter the database that you want to pull songs from")
            .prompt()
            .unwrap()
            .normalize_path(),
    };

    if args.source.is_some() && args.target.is_none() {
        return prompt_for_confirmation(DatabasePaths {
            source,
            target: None,
            output: None,
        });
    }

    let target_raw = Text::new("Path to target database:")
        .with_help_message("Leave blank to target your current Mixxx database")
        .with_validator(validators::Database::Optional)
        .prompt()
        .unwrap()
        .normalize_path();
    let target = match target_raw.is_empty() {
        false => Some(target_raw),
        true => None,
    };

    let output_raw = Text::new("Path to output database:")
        .with_help_message(
            "Leave blank to edit the target in place, enter a path to output a new file",
        )
        .with_validator(validators::Target::OptionalDirExists)
        .prompt()
        .unwrap()
        .normalize_path();
    let output = match output_raw.is_empty() {
        false => Some(output_raw),
        true => None,
    };

    let payload = DatabasePaths {
        source,
        target,
        output,
    };
    match payload.target.is_none() && payload.output.is_none() {
        true => prompt_for_confirmation(payload),
        false => Ok(payload),
    }
}

fn prompt_for_confirmation(paths: DatabasePaths) -> Result<DatabasePaths, CustomUserError> {
    let check =
        Confirm::new("You are going to edit your Mixxx database in-place. Are you sure? (y/n)")
            .with_help_message("Please make a backup of your database before continuing!")
            .prompt_skippable()
            .unwrap();
    match check.is_some_and(|b| b) {
        true => Ok(paths),
        false => Err(Box::new(MixxxkitExit::Abort)),
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
