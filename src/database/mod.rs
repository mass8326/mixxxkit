pub mod functions;
pub mod schema;

use log::error;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr,
    Statement,
};
use std::{path::PathBuf, process};

pub async fn get_sqlite_connection(path: &str) -> Result<DatabaseConnection, DbErr> {
    let url = String::from("sqlite://") + path;
    Database::connect(ConnectOptions::new(url)).await
}

#[must_use]
pub fn get_mixxx_database_path() -> PathBuf {
    [get_mixxx_directory(), "mixxxdb.sqlite".into()]
        .into_iter()
        .collect()
}

#[cfg(target_os = "windows")]
#[must_use]
pub fn get_mixxx_directory() -> PathBuf {
    let Some(localappdata) = std::env::var_os("LOCALAPPDATA") else {
        error!(r#"Could not find Mixxx database because "%localappdata%" is not set"#);
        process::exit(1);
    };
    [localappdata, "Mixxx".into()].iter().collect()
}

#[cfg(target_os = "macos")]
#[must_use]
pub fn get_mixxx_directory() -> PathBuf {
    PathBuf::from("~/Library/Containers/org.mixxx.mixxx/Data/Library/Application Support/Mixxx")
}

#[cfg(target_os = "linux")]
#[must_use]
pub fn get_mixxx_directory() -> PathBuf {
    PathBuf::from("~/.mixxx/")
}

/// Turn off foreign key constraints due to Mixxx using a broken schema
/// <https://github.com/mixxxdj/mixxx/issues/12328>
pub async fn disable_fk_constraints(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF",
    ))
    .await?;
    Ok(())
}

/// Undoes the changes done by [`remove_fk_constraints`]
pub async fn enable_fk_constraints(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON",
    ))
    .await?;
    Ok(())
}
