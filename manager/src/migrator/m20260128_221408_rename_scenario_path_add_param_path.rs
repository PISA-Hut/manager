use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260128_221408_rename_scenario_path_add_param_path"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .rename_column(Scenario::Path, Scenario::ScenarioPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .add_column(ColumnDef::new(Scenario::ParamPath).string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .drop_column(Scenario::ParamPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .rename_column(Scenario::ScenarioPath, Scenario::Path)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Scenario {
    Table,
    Path,
    ScenarioPath,
    ParamPath,
}
