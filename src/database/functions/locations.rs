use crate::database::schema::track_locations;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel};
use std::{collections::HashMap, hash::BuildHasher};

pub async fn get(db: &DatabaseConnection) -> Result<Vec<track_locations::Model>, DbErr> {
    track_locations::Entity::find().all(db).await
}

pub async fn insert<S: BuildHasher>(
    db: &DatabaseConnection,
    locations: Vec<track_locations::Model>,
    directory_map: Option<&HashMap<String, String, S>>,
) -> Result<HashMap<i32, i32>, DbErr> {
    let mut location_map = HashMap::with_capacity(locations.len());
    for (i, loc) in locations.into_iter().enumerate() {
        println!(
            "Location #{i} '{}'",
            &loc.location.as_deref().unwrap_or("[N/A]")
        );
        let prev_id = loc.id;
        let data = track_locations::ActiveModel {
            id: ActiveValue::NotSet,
            ..match directory_map {
                Some(map) => transform_location(loc, map),
                None => loc.into_active_model(),
            }
        };
        let result = track_locations::Entity::insert(data).exec(db).await?;

        location_map.insert(prev_id, result.last_insert_id);
        println!(
            "Mapping location id '{prev_id}' to '{}'",
            result.last_insert_id
        );
    }
    Ok(location_map)
}

fn transform_location<S: BuildHasher>(
    model: track_locations::Model,
    directory_map: &HashMap<String, String, S>,
) -> track_locations::ActiveModel {
    let (Some(location), Some(directory)) = (&model.location, &model.directory) else {
        return model.into_active_model();
    };
    track_locations::ActiveModel {
        location: replace_location_value(directory_map, location),
        directory: replace_location_value(directory_map, directory),
        ..model.into_active_model()
    }
}

fn replace_location_value<S: BuildHasher>(
    map: &HashMap<String, String, S>,
    subject: &String,
) -> ActiveValue<Option<String>> {
    for (from, to) in map {
        if subject.starts_with(from) {
            return ActiveValue::Set(Some(subject.replace(from, to)));
        }
    }
    ActiveValue::Unchanged(Some(subject.to_owned()))
}
