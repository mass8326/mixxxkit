use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr,
    Statement,
};

pub async fn get_sqlite_connection(path: &str) -> Result<DatabaseConnection, DbErr> {
    let mut url = String::from("sqlite://");
    url.push_str(path);
    Database::connect(ConnectOptions::new(url)).await
}

/// Turn off foreign key constraints due to Mixxx using a broken schema
/// <https://github.com/mixxxdj/mixxx/issues/12328>
pub async fn remove_fk_constraints(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF",
    ))
    .await?;
    Ok(())
}

/// Undoes the changes done by [`remove_fk_constraints`]
pub async fn restore_fk_constraints(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON",
    ))
    .await?;
    Ok(())
}
