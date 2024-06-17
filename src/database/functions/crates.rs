use crate::database::schema::{crate_tracks, crates};
use log::debug;
use sea_orm::{
    sea_query::OnConflict, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    InsertResult, QueryFilter, TryInsertResult,
};

pub async fn get_by_id<C: ConnectionTrait>(
    db: &C,
    id: i32,
) -> Result<Option<crates::Model>, DbErr> {
    crates::Entity::find_by_id(id).one(db).await
}

pub async fn get_by_name_or_create<C: ConnectionTrait>(db: &C, name: &str) -> Result<i32, DbErr> {
    let crate_maybe = crates::Entity::find()
        .filter(crates::Column::Name.eq(name))
        .one(db)
        .await?;
    if let Some(track_crate) = crate_maybe {
        debug!(r#"Found crate "{name}" with id "{}""#, track_crate.id);
        return Ok(track_crate.id);
    }

    let result = crates::Entity::insert(crates::ActiveModel {
        name: ActiveValue::Set(name.to_owned()),
        ..crates::ActiveModel::default()
    })
    .exec(db)
    .await?;

    debug!(
        r#"Created crate "{name}" with id "{}""#,
        result.last_insert_id
    );
    Ok(result.last_insert_id)
}

pub async fn clear_tracks<C: ConnectionTrait>(db: &C, crate_id: i32) -> Result<(), DbErr> {
    crate_tracks::Entity::delete_many()
        .filter(crate_tracks::Column::CrateId.eq(crate_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn connect_track<C: ConnectionTrait>(
    db: &C,
    crate_id: i32,
    track_id: i32,
) -> Result<TryInsertResult<InsertResult<crate_tracks::ActiveModel>>, DbErr> {
    let data = crate_tracks::ActiveModel {
        crate_id: ActiveValue::Set(crate_id),
        track_id: ActiveValue::Set(track_id),
    };
    crate_tracks::Entity::insert(data)
        .on_conflict(
            OnConflict::columns([crate_tracks::Column::CrateId, crate_tracks::Column::TrackId])
                .do_nothing()
                .to_owned(),
        )
        .do_nothing()
        .exec(db)
        .await
}
