pub mod av;
pub mod config_bearing;
pub mod executor;
pub mod map;
pub mod map_file;
pub mod plan;
pub mod sampler;
pub mod scenario;
pub mod scenario_file;
pub mod simulator;
pub mod task;
pub mod task_run;

pub use config_bearing::ConfigBearing;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub async fn connect(path: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(path.to_owned());
    opt.max_connections(10)
        .min_connections(5)
        .connect_timeout(std::time::Duration::from_secs(8))
        .sqlx_logging(false);
    Database::connect(opt)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database at {}: {}", path, e))
}

pub async fn migrate(db: &DatabaseConnection) -> Result<(), sea_orm_migration::prelude::DbErr> {
    crate::migrator::Migrator::up(db, None).await?;

    Ok(())
}
