pub mod functions;
pub mod schema;

use inquire::CustomUserError;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr,
    Statement,
};
use std::path::PathBuf;

pub async fn get_sqlite_connection(path: &str) -> Result<DatabaseConnection, DbErr> {
    let url = String::from("sqlite://") + path;
    Database::connect(ConnectOptions::new(url)).await
}

pub fn get_mixxx_database_path() -> Result<PathBuf, CustomUserError> {
    Ok([get_mixxx_directory()?, "mixxxdb.sqlite".into()]
        .into_iter()
        .collect())
}

#[cfg(target_os = "windows")]
pub fn get_mixxx_directory() -> Result<PathBuf, CustomUserError> {
    use crate::error::MixxxkitExit;
    use log::error;

    let Some(localappdata) = std::env::var_os("LOCALAPPDATA") else {
        error!(r#"Could not find Mixxx database because "%localappdata%" is not set"#);
        return Err(Box::new(MixxxkitExit::Abort));
    };
    Ok([localappdata, "Mixxx".into()].iter().collect())
}

#[cfg(target_os = "macos")]
#[allow(clippy::unnecessary_wraps)]
pub fn get_mixxx_directory() -> Result<PathBuf, CustomUserError> {
    Ok(PathBuf::from(
        "~/Library/Containers/org.mixxx.mixxx/Data/Library/Application Support/Mixxx",
    ))
}

#[cfg(target_os = "linux")]
#[allow(clippy::unnecessary_wraps)]
pub fn get_mixxx_directory() -> Result<PathBuf, CustomUserError> {
    Ok(PathBuf::from("~/.mixxx/"))
}

/// <https://github.com/mixxxdj/mixxx/issues/12328>
pub async fn disable_fk(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF",
    ))
    .await?;
    Ok(())
}

/// <https://github.com/mixxxdj/mixxx/issues/12328>
pub async fn enable_fk(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON",
    ))
    .await?;
    Ok(())
}
