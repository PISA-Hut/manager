use sea_orm_migration::prelude::*;

/// spec.yaml (stored under scenario_file) already carries the ego
/// destination; duplicating it into a dedicated column meant every
/// shape mismatch (`goal` vs `position`) needed its own backfill. Drop
/// the column — the executor parses spec.yaml from the staged files
/// instead.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260422_230000_drop_goal_config"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Scenario::Table)
                    .drop_column(Scenario::GoalConfig)
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
                    .add_column(
                        ColumnDef::new(Scenario::GoalConfig)
                            .json()
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Scenario {
    Table,
    GoalConfig,
}
