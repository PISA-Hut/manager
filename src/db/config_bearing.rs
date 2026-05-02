//! Generic config-blob storage for entities that own a `(config,
//! config_sha256)` pair — currently AV, Simulator, Sampler.
//!
//! Each entity hand-rolled its own near-identical `set_config` /
//! `clear_config` / `get_by_id` trio (3 × ~30 LOC). Collapse those into
//! a `ConfigBearing` trait so the HTTP handler layer can be generic
//! over the parent kind, and adding a fourth config-bearing entity
//! becomes one trait impl rather than 90 lines of new code.

use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::entity::{av, sampler, simulator};

/// Anything with a nullable bytes-config column we serve via the
/// `/{kind}/{id}/config` endpoints.
///
/// Implementors decide how to fetch / mutate the row; the trait gives
/// us a uniform vocabulary so handlers don't need to know whether
/// they're talking to AV, Simulator or Sampler.
pub trait ConfigBearing: Sized + Send {
    /// Display name used in error messages (`"av"`, `"simulator"`, …).
    fn kind() -> &'static str;

    /// Fetch the row (without dragging the config bytes out of it
    /// — that's a separate accessor so the handler can decide whether
    /// to read or just check existence).
    fn get_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> impl Future<Output = Result<Option<Self>, DbErr>> + Send;

    /// Replace the config bytes + sha on a row, returning `RecordNotFound`
    /// when the parent doesn't exist.
    fn set_config(
        db: &DatabaseConnection,
        id: i32,
        content: Vec<u8>,
        content_sha256: String,
    ) -> impl Future<Output = Result<(), DbErr>> + Send;

    /// Clear the config bytes + sha (set both to `NULL`), returning
    /// `RecordNotFound` when the parent doesn't exist.
    fn clear_config(
        db: &DatabaseConnection,
        id: i32,
    ) -> impl Future<Output = Result<(), DbErr>> + Send;

    fn config_bytes(&self) -> Option<&[u8]>;
    fn config_sha256(&self) -> Option<&str>;
}

/// Stamp out the trait impl for a `(entity_module, kind_name)` pair.
/// The bodies are identical except for the entity path and kind
/// string — collapsing them via macro is the cleanest dedup we can do
/// while still respecting sea-orm's typed `ActiveModel` field access.
macro_rules! impl_config_bearing {
    ($entity:ident, $kind:literal) => {
        impl ConfigBearing for $entity::Model {
            fn kind() -> &'static str {
                $kind
            }

            async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Self>, DbErr> {
                $entity::Entity::find_by_id(id).one(db).await
            }

            async fn set_config(
                db: &DatabaseConnection,
                id: i32,
                content: Vec<u8>,
                content_sha256: String,
            ) -> Result<(), DbErr> {
                use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
                let existing = $entity::Entity::find_by_id(id)
                    .one(db)
                    .await?
                    .ok_or_else(|| DbErr::RecordNotFound(format!("{} {} not found", $kind, id)))?;
                let mut am = existing.into_active_model();
                am.config = Set(Some(content));
                am.config_sha256 = Set(Some(content_sha256));
                am.update(db).await?;
                Ok(())
            }

            async fn clear_config(db: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
                use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
                let existing = $entity::Entity::find_by_id(id)
                    .one(db)
                    .await?
                    .ok_or_else(|| DbErr::RecordNotFound(format!("{} {} not found", $kind, id)))?;
                let mut am = existing.into_active_model();
                am.config = Set(None);
                am.config_sha256 = Set(None);
                am.update(db).await?;
                Ok(())
            }

            fn config_bytes(&self) -> Option<&[u8]> {
                self.config.as_deref()
            }

            fn config_sha256(&self) -> Option<&str> {
                self.config_sha256.as_deref()
            }
        }
    };
}

impl_config_bearing!(av, "av");
impl_config_bearing!(simulator, "simulator");
impl_config_bearing!(sampler, "sampler");
