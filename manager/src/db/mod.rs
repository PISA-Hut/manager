pub mod av;
pub mod map;
pub mod plan;
pub mod sampler;
pub mod scenario;
pub mod simulator;
pub mod task;
pub mod worker;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub async fn connect(path: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(path.to_owned());
    opt.max_connections(10)
        .min_connections(5)
        .connect_timeout(std::time::Duration::from_secs(8))
        .sqlx_logging(false);

    Database::connect(opt).await.unwrap()
}

pub async fn migrate(db: &DatabaseConnection) -> Result<(), sea_orm_migration::prelude::DbErr> {
    crate::migrator::Migrator::up(db, None).await?;

    Ok(())
}
