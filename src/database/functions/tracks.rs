use crate::database::schema::library;
use sea_orm::sea_query::SqliteQueryBuilder;
use sea_orm::{
    ActiveValue, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait,
    FromQueryResult, QueryTrait, Statement,
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
pub async fn get(db: &DatabaseConnection) -> Result<Vec<library::Model>, DbErr> {
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

pub async fn insert<S: BuildHasher>(
    db: &DatabaseConnection,
    tracks: Vec<library::Model>,
    location_map: &HashMap<i32, i32, S>,
) -> Result<(), DbErr> {
    for (i, track) in tracks.into_iter().enumerate() {
        println!(
            "Track #{i} '{} - {}'",
            &track.artist.as_deref().unwrap_or("[N/A]"),
            &track.title.as_deref().unwrap_or("[N/A]"),
        );
        if let Some(mapped_id) = location_map.get(&track.id) {
            let prev_id = track.id;
            let input = library::ActiveModel {
                id: ActiveValue::NotSet,
                location: ActiveValue::Set(Some(*mapped_id)),
                ..track.into()
            };
            library::Entity::insert(input).exec(db).await?;
            println!("Location id mapped from '{prev_id}' to '{mapped_id}'");
        } else {
            println!("Could not find mapped location id! Skipping...");
        };
    }
    Ok(())
}
