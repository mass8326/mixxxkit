mod cli;

use cli::{prompt_for_databases, prompt_for_directories};
use mixxx_merge::{
    database::{get_sqlite_connection, remove_fk_constraints, restore_fk_constraints},
    queries,
};
use std::fs;
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = prompt_for_databases();

    fs::copy(&paths.target, &paths.output)?;

    let (source, output) = try_join!(
        get_sqlite_connection(&paths.source),
        get_sqlite_connection(&paths.output),
    )?;

    let dirs = queries::directories::get(&source).await?;
    let dir_map = prompt_for_directories(&dirs).await?;
    queries::directories::insert(&output, &dirs, Some(&dir_map)).await?;

    remove_fk_constraints(&output).await?;

    let locs = queries::locations::get(&source).await?;
    let loc_map = queries::locations::insert(&output, locs, Some(&dir_map)).await?;

    let tracks = queries::tracks::get(&source).await?;
    queries::tracks::insert(&output, tracks, &loc_map).await?;

    restore_fk_constraints(&output).await?;

    Ok(())
}
