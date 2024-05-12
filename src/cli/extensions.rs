pub trait NormalizePath {
    fn trim_surround(self, char: char) -> Self;
    fn normalize_path(self) -> String;
}

impl<'a> NormalizePath for &'a str {
    fn trim_surround(self, pat: char) -> &'a str {
        let mut chars = self.chars();
        let quoted =
            chars.next().is_some_and(|c| c == pat) && chars.last().is_some_and(|c| c == pat);
        if quoted {
            &self[1..(self.len() - 1)]
        } else {
            self
        }
    }

    fn normalize_path(self) -> String {
        self.trim().trim_surround('"').replace('\\', "/")
    }
}

impl NormalizePath for String {
    fn trim_surround(self, pat: char) -> String {
        self[..].trim_surround(pat).to_owned()
    }

    fn normalize_path(self) -> String {
        self[..].normalize_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_trim_surround() {
        assert_eq!("xxxxx".trim_surround('x'), "xxx");
    }

    #[test]
    fn str_normalize_path() {
        let result = " \"C:\\Test\\Dir\" ".normalize_path();
        assert_eq!(result, "C:/Test/Dir");
    }
}
