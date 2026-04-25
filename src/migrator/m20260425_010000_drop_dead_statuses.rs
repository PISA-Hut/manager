use sea_orm_migration::prelude::*;

/// Drop the enum variants nothing writes anymore.
///
/// `task_status.exhausted` was the old permanent-fail label; with the
/// useless-streak rule routed through `invalid`, it's unreachable. Map
/// any straggler rows to `invalid`.
///
/// `task_run_status.invalid` was written by the executor's
/// error-message if-else that invalidated individual runs on
/// "route-not-found" / xml-validation crashes. That path is gone — any
/// crash now becomes `failed` and the task-level `invalid` decision
/// lives on the streak. Map stragglers to `failed`.
///
/// After this migration the state space is:
///   task.task_status     : idle | queued | running | completed | invalid | aborted
///   task_run.task_run_status : running | completed | failed | aborted
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260425_010000_drop_dead_statuses"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                -- Map stragglers onto the surviving labels before the
                -- enum shrink, or the column swap will choke.
                UPDATE task
                   SET task_status = 'invalid'::task_status
                 WHERE task_status = 'exhausted'::task_status;

                UPDATE task_run
                   SET task_run_status = 'failed'::task_run_status
                 WHERE task_run_status = 'invalid'::task_run_status;

                -- Rebuild task_status without `exhausted`.
                CREATE TYPE task_status_new AS ENUM (
                    'idle', 'queued', 'running',
                    'completed', 'invalid', 'aborted'
                );
                ALTER TABLE task ADD COLUMN task_status_new task_status_new;
                UPDATE task SET task_status_new = task_status::text::task_status_new;
                ALTER TABLE task DROP COLUMN task_status;
                ALTER TABLE task RENAME COLUMN task_status_new TO task_status;
                ALTER TABLE task ALTER COLUMN task_status SET NOT NULL;
                DROP TYPE task_status;
                ALTER TYPE task_status_new RENAME TO task_status;

                -- Rebuild task_run_status without `invalid`.
                CREATE TYPE task_run_status_new AS ENUM (
                    'running', 'completed', 'failed', 'aborted'
                );
                ALTER TABLE task_run ADD COLUMN task_run_status_new task_run_status_new;
                UPDATE task_run
                   SET task_run_status_new = task_run_status::text::task_run_status_new;
                ALTER TABLE task_run DROP COLUMN task_run_status;
                ALTER TABLE task_run RENAME COLUMN task_run_status_new TO task_run_status;
                ALTER TABLE task_run ALTER COLUMN task_run_status SET NOT NULL;
                DROP TYPE task_run_status;
                ALTER TYPE task_run_status_new RENAME TO task_run_status;
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "m20260425_010000_drop_dead_statuses cannot be rolled back \
             automatically; the dropped labels have no distinct meaning \
             to restore to. Revert in SQL by hand if truly needed."
                .to_string(),
        ))
    }
}
