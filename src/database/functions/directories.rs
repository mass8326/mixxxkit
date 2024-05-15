use crate::database::schema::directories;
use log::debug;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use std::{collections::HashMap, hash::BuildHasher};

pub async fn get(db: &DatabaseConnection) -> Result<Vec<directories::Model>, DbErr> {
    directories::Entity::find().all(db).await
}

pub async fn insert<S: BuildHasher>(
    db: &DatabaseConnection,
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
        directories::Entity::insert(data).exec(db).await?;
    }
    Ok(())
}
