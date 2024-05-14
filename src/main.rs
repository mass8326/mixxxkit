mod cli;

use clap::Parser;
use cli::commands::{Cli, Command, Run};
use inquire::Select;
use std::error::Error;
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Cli::parse().command.unwrap_or_else(prompt).run().await
}

fn prompt() -> Command {
    Select::new("What would you like to do?", Command::iter().collect())
        .prompt()
        .unwrap()
}
