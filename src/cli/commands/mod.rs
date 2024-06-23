mod backup;
mod import;
mod merge;

use clap::Subcommand;
use inquire::CustomUserError;
use strum::{Display, EnumIter};

#[derive(EnumIter, Debug, Subcommand, Display)]
pub enum Command {
    /// Create a backup of your installation database
    #[command()]
    Backup,
    /// Import m3u8 files as crates into your library
    #[command()]
    Import(import::Args),
    /// Merge two libraries together
    #[command()]
    Merge(merge::Args),
}

impl Command {
    pub async fn run(&self) -> Result<(), CustomUserError> {
        match self {
            Command::Backup => backup::run(),
            Command::Import(args) => import::run(args).await,
            Command::Merge(args) => merge::run(args).await,
        }
    }
}
