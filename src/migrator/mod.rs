// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260305_155925_new_db_schema;
mod m20260312_163958_postgrest_permission;
mod m20260422_120000_file_bytes;
mod m20260422_180000_drop_path_columns;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260305_155925_new_db_schema::Migration),
            Box::new(m20260312_163958_postgrest_permission::Migration),
            Box::new(m20260422_120000_file_bytes::Migration),
            Box::new(m20260422_180000_drop_path_columns::Migration),
        ]
    }
}
