mod extensions;
mod validators;

use crate::cli::{
    extensions::NormalizePath,
    validators::{database, directory},
};
use inquire::Text;
use mixxx_merge::entities::directories;
use sea_orm::DbErr;
use std::collections::HashMap;

pub struct DatabasePaths {
    pub source: String,
    pub target: String,
    pub output: String,
}

pub fn prompt_for_databases() -> DatabasePaths {
    let source = Text::new("Path to source database:")
        .with_validator(database::Validator)
        .with_default("source.sqlite")
        .prompt()
        .unwrap()
        .normalize_path();
    println!("Normalized to \"{source}\"");

    let target = Text::new("Path to target database:")
        .with_validator(database::Validator)
        .with_default("target.sqlite")
        .prompt()
        .unwrap()
        .normalize_path();
    println!("Normalized to \"{target}\"");

    let output = Text::new("Path to output database:")
        .with_default("mixxxdb.sqlite")
        .prompt()
        .unwrap()
        .normalize_path();
    println!("Normalized to \"{output}\"");

    DatabasePaths {
        source,
        target,
        output,
    }
}

pub async fn prompt_for_directories(
    dirs: &[directories::Model],
) -> Result<HashMap<String, String>, DbErr> {
    let mut map = HashMap::<String, String>::with_capacity(dirs.len());
    for dir in dirs {
        let path = Text::new(&format!("Replacement path for \"{}\":", &dir.directory))
            .with_validator(directory::Validator)
            .prompt()
            .unwrap()
            .trim()
            .replace('\\', "/")
            .normalize_path();
        println!("Normalized to \"{path}\"");
        if !path.is_empty() {
            map.insert(dir.directory.clone(), path);
        }
    }
    Ok(map)
}
