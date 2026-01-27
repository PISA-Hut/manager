use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260127_232346_add_simulator_sampler"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Simulator::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Simulator::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Simulator::Name).string().not_null())
                    .col(ColumnDef::new(Simulator::ModulePath).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sampler::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sampler::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sampler::Name).string().not_null())
                    .col(ColumnDef::new(Sampler::ModulePath).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(ColumnDef::new(Task::SimulatorId).integer().not_null())
                    .add_column(ColumnDef::new(Task::SamplerId).integer().not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-task-simulator-id")
                            .from_tbl(Task::Table)
                            .from_col(Task::SimulatorId)
                            .to_tbl(Simulator::Table)
                            .to_col(Simulator::Id),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-task-sampler-id")
                            .from_tbl(Task::Table)
                            .from_col(Task::SamplerId)
                            .to_tbl(Sampler::Table)
                            .to_col(Sampler::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_foreign_key(Alias::new("fk-task-simulator-id"))
                    .drop_foreign_key(Alias::new("fk-task-sampler-id"))
                    .drop_column(Task::SimulatorId)
                    .drop_column(Task::SamplerId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Simulator::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Sampler::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Simulator {
    Table,
    Id,
    Name,
    ModulePath,
}

#[derive(DeriveIden)]
enum Sampler {
    Table,
    Id,
    Name,
    ModulePath,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    SimulatorId,
    SamplerId,
}
