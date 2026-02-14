// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

// mod m20260127_225907_create_table;
// mod m20260127_231732_add_av_config_path;
// mod m20260127_232346_add_simulator_sampler;
// mod m20260128_003656_alter_map_path;
// mod m20260128_003656_alter_timestamp_zone;
// mod m20260128_221408_rename_scenario_path_add_param_path;
// mod m20260128_222459_add_simulator_config_path;
// mod m20260128_224021_make_paths_not_null;
// mod m20260128_225731_add_av_moduel_sampler_config;

mod m20260214_163021_new_db_schema;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260214_163021_new_db_schema::Migration)]
    }
}
