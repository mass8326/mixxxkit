//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "PlaylistTracks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub playlist_id: Option<i32>,
    pub track_id: Option<i32>,
    pub position: Option<i32>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub pl_datetime_added: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::playlists::Entity",
        from = "Column::PlaylistId",
        to = "super::playlists::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Playlists,
}

impl Related<super::playlists::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlists.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
