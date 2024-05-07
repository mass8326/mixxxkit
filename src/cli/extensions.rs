pub trait NormalizePath {
    fn trim_surround(self, char: char) -> String;
    fn normalize_path(self) -> String;
}

impl NormalizePath for String {
    fn trim_surround(self, pat: char) -> String {
        let mut chars = self.chars();
        let quoted =
            chars.next().is_some_and(|c| c == pat) && chars.last().is_some_and(|c| c == pat);
        if quoted {
            self[1..(self.len() - 1)].to_owned()
        } else {
            self
        }
    }

    fn normalize_path(self) -> String {
        self.trim()
            .replace('\\', "/")
            .trim_surround(char::from_u32(0x22).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_outside_quotes() {
        let result = String::from("\"\"This is a \"test\" string\"\"").normalize_path();
        assert_eq!(result, "\"This is a \"test\" string\"");
    }

    #[test]
    fn replaces_slashes() {
        let result = String::from("C:\\Test\\Dir").normalize_path();
        assert_eq!(result, "C:/Test/Dir");
    }
}
