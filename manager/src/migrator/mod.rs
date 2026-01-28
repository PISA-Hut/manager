// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260127_225907_create_table;
mod m20260127_231732_add_av_config_path;
mod m20260127_232346_add_simulator_sampler;
mod m20260128_003656_alter_map_path;
mod m20260128_003656_alter_timestamp_zone;
mod m20260128_221408_rename_scenario_path_add_param_path;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260127_225907_create_table::Migration),
            Box::new(m20260127_231732_add_av_config_path::Migration),
            Box::new(m20260127_232346_add_simulator_sampler::Migration),
            Box::new(m20260128_003656_alter_map_path::Migration),
            Box::new(m20260128_003656_alter_timestamp_zone::Migration),
            Box::new(m20260128_221408_rename_scenario_path_add_param_path::Migration),
        ]
    }
}
