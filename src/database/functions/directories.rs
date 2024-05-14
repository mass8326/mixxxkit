use crate::database::schema::directories;
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
    for (i, dir) in directories.iter().enumerate() {
        let directory = &dir.directory;
        println!("Directory #{i} '{}'", &directory);
        let data = directories::ActiveModel {
            directory: match directory_map {
                Some(map) => match map.get(directory) {
                    Some(val) => ActiveValue::Set(val.clone()),
                    None => ActiveValue::Unchanged(directory.clone()),
                },
                None => ActiveValue::Unchanged(directory.clone()),
            },
        };
        directories::Entity::insert(data).exec(db).await?;
    }
    Ok(())
}
