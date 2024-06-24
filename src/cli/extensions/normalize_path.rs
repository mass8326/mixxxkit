use std::path::PathBuf;

use super::TrimSurround;

pub trait NormalizePath {
    fn normalize_path(self) -> String;
}

impl<'a> NormalizePath for &'a str {
    fn normalize_path(self) -> String {
        self.trim().trim_surround('"').replace('\\', "/")
    }
}

impl NormalizePath for String {
    fn normalize_path(self) -> String {
        self[..].normalize_path()
    }
}

impl NormalizePath for PathBuf {
    fn normalize_path(self) -> String {
        self.to_string_lossy().replace('\\', "/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_normalize_path() {
        let result = r#" "C:\Test\Dir" "#.normalize_path();
        assert_eq!(result, "C:/Test/Dir");
    }
}
