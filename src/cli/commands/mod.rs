mod backup;
mod import;
mod merge;

use clap::{Parser, Subcommand};
use inquire::CustomUserError;
use strum::{Display, EnumIter};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Use debug logging and filter by module if provided
    #[arg(short, long, global = true, value_names = ["module"])]
    #[allow(clippy::option_option)]
    pub debug: Option<Option<String>>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

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

pub trait Run {
    async fn run(&self) -> Result<(), CustomUserError>;
}

impl Run for Command {
    async fn run(&self) -> Result<(), CustomUserError> {
        match self {
            Command::Backup => backup::run(),
            Command::Import(args) => import::run(args).await,
            Command::Merge(args) => merge::run(args).await,
        }
    }
}
