use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug, Clone)]
pub struct MixxxkitExit;

impl Display for MixxxkitExit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Mixxxkit error")
    }
}

impl Error for MixxxkitExit {}
