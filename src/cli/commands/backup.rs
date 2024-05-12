use crate::cli::validators::database;
use chrono::Utc;
use inquire::validator::{StringValidator, Validation};
use mixxxkit::database::{get_mixxx_database_path, get_mixxx_directory};
use std::{
    error::Error,
    fs::{copy, create_dir_all},
    path::PathBuf,
};

pub fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let source = get_mixxx_database_path();
    let validation = database::Validator.validate(&source.to_string_lossy())?;
    assert!(
        Validation::Valid == validation,
        "Could not find Mixxx database!"
    );

    let mut target = get_mixxx_directory();
    create_dir_all(&target)?;

    let filename = Utc::now().format("%Y-%m-%d-%s.sqlite").to_string();
    target.push(PathBuf::from("backups"));
    target.push(PathBuf::from(filename));
    copy(&source, &target)?;

    println!("Successfully backed up to \"{}\"", target.to_string_lossy());
    Ok(())
}
