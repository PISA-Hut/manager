// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260305_155925_new_db_schema;
mod m20260312_163958_postgrest_permission;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260305_155925_new_db_schema::Migration),
            Box::new(m20260312_163958_postgrest_permission::Migration),
        ]
    }
}
