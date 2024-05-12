use crate::cli::extensions::NormalizePath;
use inquire::validator::{StringValidator, Validation};
use inquire::CustomUserError;
use mixxxkit::database::get_sqlite_connection;
use sea_orm::{ConnectionTrait, DatabaseBackend, DbErr, Statement};
use std::path::Path;
use tokio::{runtime, task};

#[derive(Clone)]
pub struct Validator;

async fn can_open_database(path: &str) -> Result<(), DbErr> {
    let db = get_sqlite_connection(path).await?;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA integrity_check",
    ))
    .await?;
    Ok(())
}

impl StringValidator for Validator {
    fn validate(&self, path: &str) -> Result<Validation, CustomUserError> {
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
    fn fails_empty_string() {
        let result = Validator.validate("").unwrap();
        let expected = Validation::Invalid("Path is required!".into());
        assert_eq!(result, expected);
    }
    #[test]
    fn fails_missing_path() {
        let result = Validator.validate(".nonexistent").unwrap();
        let expected = Validation::Invalid("Could not find database at path!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn fails_invalid_file() {
        let result = Validator.validate(".gitignore").unwrap();
        let expected = Validation::Invalid("File is not a valid database!".into());
        assert_eq!(result, expected);
    }
}
