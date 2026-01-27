use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260128_003656_alter_timestamp_zone"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(
                        ColumnDef::new(Task::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .modify_column(
                        ColumnDef::new(Task::ExecutedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .modify_column(
                        ColumnDef::new(Task::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
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
                    .modify_column(ColumnDef::new(Task::CreatedAt).timestamp().not_null())
                    .modify_column(ColumnDef::new(Task::ExecutedAt).timestamp().null())
                    .modify_column(ColumnDef::new(Task::FinishedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

enum Task {
    Table,
    CreatedAt,
    ExecutedAt,
    FinishedAt,
}
impl Iden for Task {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        match self {
            Task::Table => write!(s, "task").unwrap(),
            Task::CreatedAt => write!(s, "created_at").unwrap(),
            Task::ExecutedAt => write!(s, "executed_at").unwrap(),
            Task::FinishedAt => write!(s, "finished_at").unwrap(),
        }
    }
}
