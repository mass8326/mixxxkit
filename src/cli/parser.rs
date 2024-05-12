use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Merge two libraries together")]
    Merge,
    #[command(about = "Create a backup of your current library")]
    Backup,
    #[command(about = "Import m3u8 files as crates into your library")]
    Import,
}
