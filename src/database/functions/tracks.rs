use crate::database::schema::{library, track_locations};
use log::{debug, warn};
use sea_orm::sea_query::SqliteQueryBuilder;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseBackend, DbErr, EntityTrait,
    FromQueryResult, QueryFilter, QueryTrait, Statement,
};
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Get tracks from database, accounting for the fact that `cuepoint` is set to
/// `Integer` but Mixxx may have inserted values that are `Real`
///
/// Using `#[sea_orm(select_as = "Text")]` is bugged and so we must manually
/// grab cuepoints from the `QueryResult` ourselves
///
/// <https://github.com/SeaQL/sea-orm/issues/1558>
pub async fn get<C: ConnectionTrait>(db: &C) -> Result<Vec<library::Model>, DbErr> {
    let tracks_sql = library::Entity::find()
        .into_query()
        .to_string(SqliteQueryBuilder);
    let tracks: Vec<_> = db
        .query_all(Statement::from_string(DatabaseBackend::Sqlite, tracks_sql))
        .await?
        .into_iter()
        .map(|v| library::Model {
            cuepoint: v.try_get_by_index(13).unwrap(),
            ..library::Model::from_query_result(&v, "").unwrap()
        })
        .collect();
    Ok(tracks)
}

pub async fn get_by_location<C: ConnectionTrait>(
    db: &C,
    path: &str,
) -> Result<Option<library::Model>, DbErr> {
    library::Entity::find()
        .inner_join(track_locations::Entity)
        .filter(track_locations::Column::Location.eq(path))
        .one(db)
        .await
}

pub async fn insert<C: ConnectionTrait, S: BuildHasher>(
    db: &C,
    tracks: Vec<library::Model>,
    location_map: &HashMap<i32, i32, S>,
) -> Result<(), DbErr> {
    for track in tracks {
        let display = format!(
            r#""{} - {}""#,
            &track.artist.as_deref().unwrap_or("<N/A>"),
            &track.title.as_deref().unwrap_or("<N/A>")
        );
        let Some(prev_loc_id) = track.location else {
            warn!(r#"Track {display} has no original location! Skipping..."#);
            continue;
        };
        let Some(mapped_loc_id) = location_map.get(&prev_loc_id) else {
            warn!(
                r#"Could not find new location of {display} with id "{prev_loc_id}"! Skipping..."#,
            );
            continue;
        };
        let input = library::ActiveModel {
            id: ActiveValue::NotSet,
            location: ActiveValue::Set(Some(*mapped_loc_id)),
            ..track.into()
        };
        let result = library::Entity::insert(input).exec(db).await?;
        debug!(
            r#"Created {display} with track id {}, mapping location id from "{prev_loc_id}" to "{mapped_loc_id}""#,
            result.last_insert_id
        );
    }
    Ok(())
}
