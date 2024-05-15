use crate::cli::validators;
use crate::database::{get_mixxx_database_path, get_mixxx_directory};
use chrono::Utc;
use inquire::validator::{StringValidator, Validation};
use log::{error, info};
use std::{
    error::Error,
    fs::{copy, create_dir_all},
    path::PathBuf,
    process,
};

pub fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let source = get_mixxx_database_path();
    let validation = validators::Database.validate(&source.to_string_lossy())?;
    if Validation::Valid != validation {
        error!("Could not find Mixxx database");
        process::exit(1);
    }

    let mut target = get_mixxx_directory();
    create_dir_all(&target)?;

    let filename = Utc::now().format("%Y-%m-%d-%s.sqlite").to_string();
    target.push(PathBuf::from("backups"));
    target.push(PathBuf::from(filename));
    copy(&source, &target)?;

    info!(
        r#"Successfully backed up to "{}""#,
        target.to_string_lossy()
    );
    Ok(())
}
