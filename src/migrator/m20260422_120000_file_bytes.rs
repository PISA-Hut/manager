use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260422_120000_file_bytes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MapFile::Table)
                    .col(
                        ColumnDef::new(MapFile::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MapFile::MapId).integer().not_null())
                    .col(ColumnDef::new(MapFile::RelativePath).string().not_null())
                    .col(ColumnDef::new(MapFile::Content).binary().not_null())
                    .col(ColumnDef::new(MapFile::ContentSha256).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(MapFile::Table, MapFile::MapId)
                            .to(Map::Table, Map::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_map_file_unique")
                            .col(MapFile::MapId)
                            .col(MapFile::RelativePath)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ScenarioFile::Table)
                    .col(
                        ColumnDef::new(ScenarioFile::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ScenarioFile::ScenarioId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScenarioFile::RelativePath)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScenarioFile::Content).binary().not_null())
                    .col(
                        ColumnDef::new(ScenarioFile::ContentSha256)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ScenarioFile::Table, ScenarioFile::ScenarioId)
                            .to(Scenario::Table, Scenario::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_scenario_file_unique")
                            .col(ScenarioFile::ScenarioId)
                            .col(ScenarioFile::RelativePath)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AV::Table)
                    .add_column(ColumnDef::new(AV::Config).binary().null())
                    .add_column(ColumnDef::new(AV::ConfigSha256).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Simulator::Table)
                    .add_column(ColumnDef::new(Simulator::Config).binary().null())
                    .add_column(ColumnDef::new(Simulator::ConfigSha256).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .add_column(ColumnDef::new(Sampler::Config).binary().null())
                    .add_column(ColumnDef::new(Sampler::ConfigSha256).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE map_file TO web_anon;
                GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE scenario_file TO web_anon;
                GRANT USAGE, SELECT ON SEQUENCE map_file_id_seq TO web_anon;
                GRANT USAGE, SELECT ON SEQUENCE scenario_file_id_seq TO web_anon;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .drop_column(Sampler::ConfigSha256)
                    .drop_column(Sampler::Config)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Simulator::Table)
                    .drop_column(Simulator::ConfigSha256)
                    .drop_column(Simulator::Config)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AV::Table)
                    .drop_column(AV::ConfigSha256)
                    .drop_column(AV::Config)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ScenarioFile::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(MapFile::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum MapFile {
    Table,
    Id,
    MapId,
    RelativePath,
    Content,
    ContentSha256,
}

#[derive(DeriveIden)]
enum ScenarioFile {
    Table,
    Id,
    ScenarioId,
    RelativePath,
    Content,
    ContentSha256,
}

#[derive(DeriveIden)]
enum Map {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Scenario {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum AV {
    Table,
    Config,
    ConfigSha256,
}

#[derive(DeriveIden)]
enum Simulator {
    Table,
    Config,
    ConfigSha256,
}

#[derive(DeriveIden)]
enum Sampler {
    Table,
    Config,
    ConfigSha256,
}
