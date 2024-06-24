pub trait TrimSurround {
    fn trim_surround(self, char: char) -> Self;
}

impl<'a> TrimSurround for &'a str {
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
}

impl TrimSurround for String {
    fn trim_surround(self, pat: char) -> String {
        self[..].trim_surround(pat).to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_trim_surround() {
        assert_eq!("xxxxx".trim_surround('x'), "xxx");
    }
}
