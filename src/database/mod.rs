pub mod functions;
pub mod schema;

use inquire::CustomUserError;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, DbErr, Statement, TransactionTrait,
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
        return Err(Box::new(MixxxkitExit));
    };
    Ok([localappdata, "Mixxx".into()].iter().collect())
}

#[cfg(target_os = "macos")]
#[must_use]
pub fn get_mixxx_directory() -> Result<PathBuf, CustomUserError> {
    Ok(PathBuf::from(
        "~/Library/Containers/org.mixxx.mixxx/Data/Library/Application Support/Mixxx",
    ))
}

#[cfg(target_os = "linux")]
#[must_use]
pub fn get_mixxx_directory() -> Result<PathBuf, CustomUserError> {
    Ok(PathBuf::from("~/.mixxx/"))
}

/// Additionally makes changes due to Mixxx using a broken schema
/// <https://github.com/mixxxdj/mixxx/issues/12328>
pub async fn begin_transaction(db: &DatabaseConnection) -> Result<DatabaseTransaction, DbErr> {
    let txn = db.begin().await?;
    txn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF",
    ))
    .await?;
    txn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE library_old (id INTEGER PRIMARY KEY) WITHOUT ROWID",
    ))
    .await?;
    Ok(txn)
}

/// Undoes the changes done by [`remove_fk_constraints`] and commits the transaction
pub async fn commit_transaction(txn: DatabaseTransaction) -> Result<(), DbErr> {
    txn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "DROP TABLE library_old",
    ))
    .await?;
    txn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON",
    ))
    .await?;
    txn.commit().await?;
    Ok(())
}
