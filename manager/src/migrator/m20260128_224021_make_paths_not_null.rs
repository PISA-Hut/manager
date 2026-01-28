use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260128_224021_add_simulator_config_path"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Simulator::Table)
                    .modify_column(ColumnDef::new(Simulator::ConfigPath).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .modify_column(ColumnDef::new(Scenario::ParamPath).string().not_null())
                    .modify_column(ColumnDef::new(Scenario::ScenarioPath).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Simulator::Table)
                    .modify_column(ColumnDef::new(Simulator::ConfigPath).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .modify_column(ColumnDef::new(Scenario::ParamPath).string().null())
                    .modify_column(ColumnDef::new(Scenario::ScenarioPath).string().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Simulator {
    Table,
    ConfigPath,
}

#[derive(Iden)]
enum Scenario {
    Table,
    ScenarioPath,
    ParamPath,
}
