use crate::database::schema::directories;
use log::{debug, warn};
use sea_orm::{ActiveValue, ConnectionTrait, DbErr, EntityTrait};
use std::{collections::HashMap, hash::BuildHasher};

pub async fn get<C: ConnectionTrait>(db: &C) -> Result<Vec<directories::Model>, DbErr> {
    directories::Entity::find().all(db).await
}

pub async fn insert<C: ConnectionTrait, S: BuildHasher>(
    db: &C,
    directories: &[directories::Model],
    directory_map: Option<&HashMap<String, String, S>>,
) -> Result<(), DbErr> {
    for dir in directories {
        let directory = &dir.directory;
        let data = directories::ActiveModel {
            directory: directory_map
                .and_then(|map| map.get(directory))
                .map_or_else(
                    || {
                        debug!(r#"Merging directory "{directory}" unchanged"#);
                        ActiveValue::Unchanged(directory.to_string())
                    },
                    |val| {
                        debug!(r#"Merging directory "{directory}" as "{val}""#);
                        ActiveValue::Set(val.clone())
                    },
                ),
        };
        let Ok(_) = directories::Entity::insert(data).exec(db).await else {
            warn!(r#"Could not insert directory "{directory}"! Skipping..."#,);
            continue;
        };
    }
    Ok(())
}
