pub mod commands;
mod extensions;
mod validators;

use clap::Parser;
use commands::Command;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Use debug logging and optionally trace by module (e.g. `mixxxkit::import::commands`)
    #[arg(short, long, global = true, value_names = ["module"])]
    #[allow(clippy::option_option)]
    pub debug: Option<Option<String>>,

    #[command(subcommand)]
    pub command: Option<Command>,
}
