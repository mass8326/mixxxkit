#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database ran into error {0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("Could not read mixxxkit.crates.yaml {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Unable to parse mixxxkit.crates.yaml")]
    ParsingFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
