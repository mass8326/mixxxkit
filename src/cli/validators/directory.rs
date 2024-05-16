use crate::cli::extensions::NormalizePath;
use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};
use std::path::Path;

#[derive(Clone)]
pub enum Directory {
    Required,
    Optional,
}

impl Directory {
    fn is_optional(&self) -> bool {
        matches!(*self, Self::Optional)
    }
}

impl StringValidator for Directory {
    fn validate(&self, path: &str) -> Result<Validation, CustomUserError> {
        if self.is_optional() && path.is_empty() {
            return Ok(Validation::Valid);
        }
        let normalized = path.to_owned().normalize_path();
        match Path::new(&normalized).is_dir() {
            true => Ok(Validation::Valid),
            false => Ok(Validation::Invalid("Directory does not exist!".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fails_missing() {
        let result = Directory::Required.validate(".nonexistent").unwrap();
        let expected = Validation::Invalid("Directory does not exist!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn fails_file() {
        let result = Directory::Required.validate(".gitignore").unwrap();
        let expected = Validation::Invalid("Directory does not exist!".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn passes_directory() {
        let result = Directory::Required.validate("src").unwrap();
        assert_eq!(result, Validation::Valid);
    }

    #[test]
    fn passes_optional() {
        let result = Directory::Optional.validate("").unwrap();
        assert_eq!(result, Validation::Valid);
    }
}
