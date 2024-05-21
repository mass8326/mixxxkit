use crate::database::schema::{crate_tracks, crates, library, track_locations};
use log::debug;
use sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};

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

pub async fn clear_crate_tracks<C: ConnectionTrait>(db: &C, crate_id: i32) -> Result<(), DbErr> {
    crate_tracks::Entity::delete_many()
        .filter(crate_tracks::Column::CrateId.eq(crate_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn connect_track_by_location<C: ConnectionTrait>(
    db: &C,
    crate_id: i32,
    path: &str,
) -> Result<Option<()>, DbErr> {
    let Some((location, Some(track))) = track_locations::Entity::find()
        .filter(track_locations::Column::Location.eq(path))
        .find_also_related(library::Entity)
        .one(db)
        .await?
    else {
        return Ok(None);
    };

    debug!(
        r#"Connecting "{path}" with location id "{}" and track id "{}" to crate id "{crate_id}""#,
        location.id, track.id,
    );
    let data = crate_tracks::ActiveModel {
        crate_id: ActiveValue::Set(crate_id),
        track_id: ActiveValue::Set(track.id),
    };
    crate_tracks::Entity::insert(data).exec(db).await?;
    Ok(Some(()))
}
