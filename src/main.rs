mod cli;
mod database;
mod error;

use clap::Parser;
use cli::commands::Command;
use cli::Cli;
use flexi_logger::Logger;
use inquire::{error::InquireResult, CustomUserError, Select};
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() -> Result<(), CustomUserError> {
    let cli = Cli::parse();

    let specification = match &cli.debug {
        None => "mixxxkit=info".to_owned(),
        Some(debug) => match debug {
            None => "mixxxkit=debug".to_owned(),
            Some(module) => format!("mixxxkit=info,{module}=trace"),
        },
    };
    Logger::try_with_str(specification)?
        .log_to_stdout()
        .start()?;

    match cli.command {
        Some(cmd) => cmd.run().await,
        None => match prompt()? {
            Some(cmd) => cmd.run().await,
            _ => Ok(()),
        },
    }
}

fn prompt() -> InquireResult<Option<Command>> {
    Select::new("What would you like to do?", Command::iter().collect()).prompt_skippable()
}
