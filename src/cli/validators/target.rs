use crate::cli::extensions::NormalizePath;
use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};
use std::{env::current_dir, path::Path};

#[derive(Clone)]
pub enum Target {
    OptionalDirExists,
}

impl StringValidator for Target {
    fn validate(&self, path: &str) -> Result<Validation, CustomUserError> {
        if path.is_empty() {
            return Ok(Validation::Valid);
        }
        let normalized = path.to_owned().normalize_path();
        let path = Path::new(&normalized);
        let raw = path.parent().unwrap();
        let dir = if raw.to_string_lossy().is_empty() {
            current_dir()?
        } else {
            raw.to_owned()
        };
        let result = match (dir.is_dir(), path.exists()) {
            (true, false) => Validation::Valid,
            (true, true) => Validation::Invalid("File already exists at path!".into()),
            (false, _) => Validation::Invalid("Parent directory does not exist!".into()),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_value() {
        let result = Target::OptionalDirExists
            .validate(".nonexistent.txt")
            .unwrap();
        assert_eq!(result, Validation::Valid);
    }
    #[test]
    fn passes_empty() {
        let result = Target::OptionalDirExists.validate("").unwrap();
        assert_eq!(result, Validation::Valid);
    }

    #[test]
    fn fails_missing_directory() {
        let result = Target::OptionalDirExists
            .validate(".nonexistent/file.txt")
            .unwrap();
        let expected = Validation::Invalid("Parent directory does not exist!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn fails_existing_file() {
        let result = Target::OptionalDirExists.validate(".gitignore").unwrap();
        let expected = Validation::Invalid("File already exists at path!".into());
        assert_eq!(result, expected);
    }
}
