use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260422_180000_drop_path_columns"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Map::Table)
                    .drop_column(Map::XodrPath)
                    .drop_column(Map::OsmPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .drop_column(Scenario::ScenarioPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AV::Table)
                    .drop_column(AV::ConfigPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Simulator::Table)
                    .drop_column(Simulator::ConfigPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .drop_column(Sampler::ConfigPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .add_column(ColumnDef::new(Sampler::ConfigPath).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Simulator::Table)
                    .add_column(ColumnDef::new(Simulator::ConfigPath).string().not_null().default(""))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(AV::Table)
                    .add_column(ColumnDef::new(AV::ConfigPath).string().not_null().default(""))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .add_column(ColumnDef::new(Scenario::ScenarioPath).string().not_null().default(""))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Map::Table)
                    .add_column(ColumnDef::new(Map::XodrPath).string().null())
                    .add_column(ColumnDef::new(Map::OsmPath).string().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Map {
    Table,
    XodrPath,
    OsmPath,
}

#[derive(DeriveIden)]
enum Scenario {
    Table,
    ScenarioPath,
}

#[derive(DeriveIden)]
enum AV {
    Table,
    ConfigPath,
}

#[derive(DeriveIden)]
enum Simulator {
    Table,
    ConfigPath,
}

#[derive(DeriveIden)]
enum Sampler {
    Table,
    ConfigPath,
}
