use thiserror::Error;

#[derive(Debug, Error)]
pub enum MixxxkitExit {
    #[error("Program aborted")]
    Abort,
}
