mod cli;

use clap::Parser;
use cli::{
    commands::{backup, import, merge},
    parser::{Cli, Commands},
};
use inquire::Select;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let cli = Cli::parse();
    match cli.command.unwrap_or_else(prompt) {
        Commands::Merge => merge::run().await,
        Commands::Backup => backup::run(),
        Commands::Import => import::run().await,
    }
}

fn prompt() -> Commands {
    let result = Select::new("What would you like to do?", vec!["merge", "backup"])
        .prompt()
        .unwrap();
    match result {
        "merge" => Commands::Merge,
        "backup" => Commands::Backup,
        _ => panic!("Unexpected command selected!"),
    }
}
