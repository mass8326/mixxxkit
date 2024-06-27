use crate::cli::traits::NormalizePath;
use crate::database::get_sqlite_connection;
use inquire::validator::{StringValidator, Validation};
use inquire::CustomUserError;
use sea_orm::{ConnectionTrait, DatabaseBackend, DbErr, Statement};
use std::path::Path;
use tokio::{runtime, task};

#[derive(Clone)]
pub enum Database {
    Required,
    Optional,
}

impl Database {
    fn is_optional(&self) -> bool {
        matches!(*self, Self::Optional)
    }
}

async fn can_open_database(path: &str) -> Result<(), DbErr> {
    let db = get_sqlite_connection(path).await?;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA integrity_check",
    ))
    .await?;
    Ok(())
}

impl StringValidator for Database {
    fn validate(&self, path: &str) -> Result<Validation, CustomUserError> {
        if self.is_optional() && path.is_empty() {
            return Ok(Validation::Valid);
        }

        let normalized = path.normalize_path();
        if normalized.is_empty() {
            return Ok(Validation::Invalid("Path is required!".into()));
        }
        if !Path::new(&normalized).exists() {
            return Ok(Validation::Invalid(
                "Could not find database at path!".into(),
            ));
        }
        let result = task::block_in_place(|| {
            let rt = runtime::Runtime::new().unwrap();
            rt.block_on(can_open_database(&normalized))
        });
        match result {
            Ok(()) => Ok(Validation::Valid),
            Err(_) => Ok(Validation::Invalid("File is not a valid database!".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_passes_empty_string() {
        let result = Database::Optional.validate("").unwrap();
        let expected = Validation::Valid;
        assert_eq!(result, expected);
    }
    #[test]
    fn required_fails_empty_string() {
        let result = Database::Required.validate("").unwrap();
        let expected = Validation::Invalid("Path is required!".into());
        assert_eq!(result, expected);
    }
    #[test]
    fn fails_missing_path() {
        let result = Database::Required.validate(".nonexistent").unwrap();
        let expected = Validation::Invalid("Could not find database at path!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn fails_invalid_file() {
        let result = Database::Required.validate(".gitignore").unwrap();
        let expected = Validation::Invalid("File is not a valid database!".into());
        assert_eq!(result, expected);
    }
}
