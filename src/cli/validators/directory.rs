use crate::cli::extensions::NormalizePath;
use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};
use std::path::Path;

#[derive(Clone)]
pub struct Directory;

impl StringValidator for Directory {
    fn validate(&self, path: &str) -> Result<Validation, CustomUserError> {
        let normalized = path.to_owned().normalize_path();
        Ok(if Path::new(&normalized).is_dir() {
            Validation::Valid
        } else {
            Validation::Invalid("Directory does not exist!".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fails_missing() {
        let result = Directory.validate(".nonexistent").unwrap();
        let expected = Validation::Invalid("Directory does not exist!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn fails_file() {
        let result = Directory.validate(".gitignore").unwrap();
        let expected = Validation::Invalid("Directory does not exist!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn passes_directory() {
        let result = Directory.validate("src").unwrap();
        assert_eq!(result, Validation::Valid);
    }
}
