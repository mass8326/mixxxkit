mod backup;
mod import;
mod merge;

use std::error::Error;

use clap::{Parser, Subcommand};
use strum::{Display, EnumIter};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, global = true, value_names = ["module"])]
    #[allow(clippy::option_option)]
    pub debug: Option<Option<String>>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(EnumIter, Debug, Subcommand, Display)]
pub enum Command {
    #[command(about = "Create a backup of your current library")]
    Backup,
    #[command(about = "Import m3u8 files as crates into your library")]
    Import,
    #[command(about = "Merge two libraries together")]
    Merge,
}

pub trait Run {
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;
}

impl Run for Command {
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        match self {
            Command::Backup => backup::run(),
            Command::Import => import::run().await,
            Command::Merge => merge::run().await,
        }
    }
}
