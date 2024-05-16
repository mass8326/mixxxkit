use crate::cli::validators;
use crate::database::{get_mixxx_database_path, get_mixxx_directory};
use chrono::Utc;
use inquire::validator::{StringValidator, Validation};
use log::{error, info};
use std::path::Path;
use std::{
    error::Error,
    fs::{copy, create_dir_all},
};

pub fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let source = get_mixxx_database_path();
    let validation = validators::Database::Required.validate(&source.to_string_lossy())?;
    if Validation::Valid != validation {
        error!("Could not find Mixxx database");
        std::process::exit(1);
    }

    let mut target = get_mixxx_directory();
    create_dir_all(&target)?;

    let filename = Utc::now().format("%Y-%m-%d-%s.sqlite").to_string();
    target.push(Path::new("backups"));
    target.push(Path::new(&filename));
    copy(&source, &target)?;

    info!(
        r#"Successfully backed up to "{}""#,
        target.to_string_lossy()
    );
    Ok(())
}
