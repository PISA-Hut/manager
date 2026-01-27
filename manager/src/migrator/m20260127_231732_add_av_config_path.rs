use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260127_add_config_path_to_av"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AV::Table)
                    .add_column(ColumnDef::new(AV::ConfigPath).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AV::Table)
                    .drop_column(AV::ConfigPath)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AV {
    Table,
    ConfigPath,
}
