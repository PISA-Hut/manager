use sea_orm_migration::prelude::*;

/// Rename `scenario.goal_config.goal` → `scenario.goal_config.position` on
/// every existing row so simcore stops bailing out with
/// `ValueError: ego.position not defined`. The current spec.yaml bundles
/// use `goal`, but simcore reads `position`.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260422_220000_normalize_goal_config"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE scenario
                SET goal_config = jsonb_set(
                    (goal_config::jsonb) - 'goal',
                    '{position}',
                    (goal_config::jsonb) -> 'goal'
                )::json
                WHERE (goal_config::jsonb) ? 'goal'
                  AND NOT ((goal_config::jsonb) ? 'position');
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Not reversed — data-only normalisation.
        Ok(())
    }
}
