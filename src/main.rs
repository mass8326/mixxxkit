mod cli;
mod database;

use clap::Parser;
use cli::commands::{Cli, Command, Run};
use flexi_logger::Logger;
use inquire::Select;
use std::error::Error;
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let cli = Cli::parse();

    let specification = match &cli.debug {
        None => "mixxxkit=info".to_owned(),
        Some(debug) => match debug {
            None => "mixxxkit=debug".to_owned(),
            Some(module) => format!("{module}=trace"),
        },
    };
    Logger::try_with_str(specification)?
        .log_to_stdout()
        .start()?;

    cli.command.unwrap_or_else(prompt).run().await
}

fn prompt() -> Command {
    Select::new("What would you like to do?", Command::iter().collect())
        .prompt()
        .unwrap()
}
