use crate::cli::validators;
use crate::database::{get_mixxx_database_path, get_mixxx_directory};
use crate::error::MixxxkitExit;
use chrono::Utc;
use inquire::validator::{StringValidator, Validation};
use inquire::CustomUserError;
use log::{error, info};
use std::fs::{copy, create_dir_all};
use std::path::Path;

pub fn run() -> Result<(), CustomUserError> {
    let source = get_mixxx_database_path()?;
    let validation = validators::Database::Required.validate(&source.to_string_lossy())?;
    if Validation::Valid != validation {
        error!("Could not find Mixxx database");
        return Err(Box::new(MixxxkitExit));
    }

    let mut target = get_mixxx_directory()?;
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
