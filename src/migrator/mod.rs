// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260305_155925_new_db_schema;
mod m20260312_163958_postgrest_permission;
mod m20260422_120000_file_bytes;
mod m20260422_180000_drop_path_columns;
mod m20260422_200000_task_run_log;
mod m20260422_220000_normalize_goal_config;
mod m20260422_230000_drop_goal_config;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260305_155925_new_db_schema::Migration),
            Box::new(m20260312_163958_postgrest_permission::Migration),
            Box::new(m20260422_120000_file_bytes::Migration),
            Box::new(m20260422_180000_drop_path_columns::Migration),
            Box::new(m20260422_200000_task_run_log::Migration),
            Box::new(m20260422_220000_normalize_goal_config::Migration),
            Box::new(m20260422_230000_drop_goal_config::Migration),
        ]
    }
}
