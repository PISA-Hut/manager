//! Generic file storage for entities that own `(parent_id,
//! relative_path) → (content, content_sha256)` rows — currently
//! `map_file` and `scenario_file`.
//!
//! Mirrors the `ConfigBearing` trait pattern: the macro stamps out
//! per-entity impls so the HTTP handler layer can be generic, and
//! adding a third file-bearing entity becomes one trait impl rather
//! than ~70 lines of new code.

use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::entity::{map_file, scenario_file};

pub trait FileStore: Sized + Send {
    /// Display name of the parent kind (`"map"`, `"scenario"`, …).
    fn parent_kind() -> &'static str;

    fn find_by_parent(
        db: &DatabaseConnection,
        parent_id: i32,
    ) -> impl Future<Output = Result<Vec<Self>, DbErr>> + Send;

    fn get(
        db: &DatabaseConnection,
        parent_id: i32,
        relative_path: &str,
    ) -> impl Future<Output = Result<Option<Self>, DbErr>> + Send;

    /// Upsert via `INSERT … ON CONFLICT … DO UPDATE` so the
    /// `(parent_id, relative_path)` unique-index path is collision-safe
    /// under concurrent uploads of the same file.
    fn put(
        db: &DatabaseConnection,
        parent_id: i32,
        relative_path: String,
        content: Vec<u8>,
        content_sha256: String,
    ) -> impl Future<Output = Result<Self, DbErr>> + Send;

    /// Returns the number of rows deleted (0 if the file didn't exist).
    fn delete(
        db: &DatabaseConnection,
        parent_id: i32,
        relative_path: &str,
    ) -> impl Future<Output = Result<u64, DbErr>> + Send;

    fn content(&self) -> &[u8];
    fn content_sha256(&self) -> &str;
}

/// Stamp out the impl. Each entity has its own typed `Column` enum and
/// `ActiveModel` constructor so we macro over the differences (parent
/// id field name + the column variants).
macro_rules! impl_file_store {
    (
        entity: $entity:ident,
        parent_kind: $parent_kind:literal,
        parent_id_field: $parent_id_field:ident,
        parent_id_col: $parent_id_col:ident $(,)?
    ) => {
        impl FileStore for $entity::Model {
            fn parent_kind() -> &'static str {
                $parent_kind
            }

            async fn find_by_parent(
                db: &DatabaseConnection,
                parent_id: i32,
            ) -> Result<Vec<Self>, DbErr> {
                use sea_orm::{ColumnTrait, QueryFilter};
                $entity::Entity::find()
                    .filter($entity::Column::$parent_id_col.eq(parent_id))
                    .all(db)
                    .await
            }

            async fn get(
                db: &DatabaseConnection,
                parent_id: i32,
                relative_path: &str,
            ) -> Result<Option<Self>, DbErr> {
                use sea_orm::{ColumnTrait, QueryFilter};
                $entity::Entity::find()
                    .filter($entity::Column::$parent_id_col.eq(parent_id))
                    .filter($entity::Column::RelativePath.eq(relative_path))
                    .one(db)
                    .await
            }

            async fn put(
                db: &DatabaseConnection,
                parent_id: i32,
                relative_path: String,
                content: Vec<u8>,
                content_sha256: String,
            ) -> Result<Self, DbErr> {
                use sea_orm::Set;
                use sea_orm::sea_query::OnConflict;
                let am = $entity::ActiveModel {
                    $parent_id_field: Set(parent_id),
                    relative_path: Set(relative_path),
                    content: Set(content),
                    content_sha256: Set(content_sha256),
                    ..Default::default()
                };
                $entity::Entity::insert(am)
                    .on_conflict(
                        OnConflict::columns([
                            $entity::Column::$parent_id_col,
                            $entity::Column::RelativePath,
                        ])
                        .update_columns([$entity::Column::Content, $entity::Column::ContentSha256])
                        .to_owned(),
                    )
                    .exec_with_returning(db)
                    .await
            }

            async fn delete(
                db: &DatabaseConnection,
                parent_id: i32,
                relative_path: &str,
            ) -> Result<u64, DbErr> {
                use sea_orm::{ColumnTrait, QueryFilter};
                let res = $entity::Entity::delete_many()
                    .filter($entity::Column::$parent_id_col.eq(parent_id))
                    .filter($entity::Column::RelativePath.eq(relative_path))
                    .exec(db)
                    .await?;
                Ok(res.rows_affected)
            }

            fn content(&self) -> &[u8] {
                &self.content
            }

            fn content_sha256(&self) -> &str {
                &self.content_sha256
            }
        }
    };
}

impl_file_store!(
    entity: scenario_file,
    parent_kind: "scenario",
    parent_id_field: scenario_id,
    parent_id_col: ScenarioId,
);

impl_file_store!(
    entity: map_file,
    parent_kind: "map",
    parent_id_field: map_id,
    parent_id_col: MapId,
);
