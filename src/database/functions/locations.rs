use crate::database::schema::track_locations;
use log::{debug, warn};
use sea_orm::{ActiveValue, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel};
use std::{collections::HashMap, hash::BuildHasher};

pub async fn get<C: ConnectionTrait>(db: &C) -> Result<Vec<track_locations::Model>, DbErr> {
    track_locations::Entity::find().all(db).await
}

pub async fn insert<C: ConnectionTrait, S: BuildHasher>(
    db: &C,
    locations: Vec<track_locations::Model>,
    directory_map: Option<&HashMap<String, String, S>>,
) -> Result<HashMap<i32, i32>, DbErr> {
    let mut location_map = HashMap::with_capacity(locations.len());
    for loc in locations {
        let prev_id = loc.id;
        let prev_path = loc.location.clone().unwrap_or_else(|| "<N/A>".to_owned());
        let data = track_locations::ActiveModel {
            id: ActiveValue::NotSet,
            ..match directory_map {
                Some(map) => transform_location(loc, map),
                None => loc.into_active_model(),
            }
        };
        let path = match data.location.clone() {
            ActiveValue::Set(val) | ActiveValue::Unchanged(val) => val,
            ActiveValue::NotSet => None,
        }
        .unwrap_or_else(|| "<N/A>".to_owned());
        let Ok(result) = track_locations::Entity::insert(data).exec(db).await else {
            warn!(r#"Could not insert location "{path}"! Skipping..."#,);
            continue;
        };
        location_map.insert(prev_id, result.last_insert_id);
        debug!(
            r#"Mapped "{prev_path}" with id "{prev_id}" to "{path}" with id "{}""#,
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
