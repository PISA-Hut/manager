// src/migrator/m20260214_163021_new_db_schema.rs (create new file)

use sea_orm::ActiveEnum;
use sea_orm::{DbBackend, Schema};
use sea_orm_migration::prelude::*;

use crate::migrator::sea_orm::{DeriveActiveEnum, EnumIter};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260305_155925_new_db_schema"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<ScenarioType>())
            .await?;

        manager
            .create_type(schema.create_enum_from_active_enum::<TaskStatus>())
            .await?;

        manager
            .create_type(schema.create_enum_from_active_enum::<TaskRunStatus>())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Executor::Table)
                    .col(
                        ColumnDef::new(Executor::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Executor::SlurmJobId).integer().not_null())
                    .col(ColumnDef::new(Executor::SlurmArrayId).integer().not_null())
                    .col(ColumnDef::new(Executor::SlurmNodeList).string().not_null())
                    .col(ColumnDef::new(Executor::Hostname).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AV::Table)
                    .col(
                        ColumnDef::new(AV::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AV::Name).string().not_null())
                    .col(ColumnDef::new(AV::ImagePath).json().not_null())
                    .col(ColumnDef::new(AV::ConfigPath).string().not_null())
                    .col(
                        ColumnDef::new(AV::NvRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AV::RosRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AV::CarlaRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Scenario::Table)
                    .col(
                        ColumnDef::new(Scenario::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Scenario::Type)
                            .custom(ScenarioType::name())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Scenario::Title).string().null())
                    .col(ColumnDef::new(Scenario::ScenarioPath).string().not_null())
                    .col(ColumnDef::new(Scenario::GoalConfig).json().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Map::Table)
                    .col(
                        ColumnDef::new(Map::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Map::Name).string().not_null())
                    .col(ColumnDef::new(Map::XodrPath).string().null())
                    .col(ColumnDef::new(Map::OsmPath).string().null())
                    .to_owned(),
            )
            .await?;

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
                    .col(ColumnDef::new(Simulator::ImagePath).json().not_null())
                    .col(ColumnDef::new(Simulator::ConfigPath).string().not_null())
                    .col(
                        ColumnDef::new(Simulator::NvRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Simulator::RosRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Simulator::CarlaRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
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
                    .col(ColumnDef::new(Sampler::ConfigPath).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Plan::Table)
                    .col(
                        ColumnDef::new(Plan::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Plan::Name).string().not_null())
                    .col(ColumnDef::new(Plan::MapId).integer().not_null())
                    .col(ColumnDef::new(Plan::ScenarioId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Plan::Table, Plan::MapId)
                            .to(Map::Table, Map::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Plan::Table, Plan::ScenarioId)
                            .to(Scenario::Table, Scenario::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .col(
                        ColumnDef::new(Task::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Task::PlanId).integer().not_null())
                    .col(ColumnDef::new(Task::AvId).integer().not_null())
                    .col(ColumnDef::new(Task::SimulatorId).integer().not_null())
                    .col(ColumnDef::new(Task::SamplerId).integer().not_null())
                    .col(
                        ColumnDef::new(Task::TaskStatus)
                            .custom(TaskStatus::name())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Task::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Task::RetryCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::PlanId)
                            .to(Plan::Table, Plan::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::AvId)
                            .to(AV::Table, AV::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::SimulatorId)
                            .to(Simulator::Table, Simulator::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::SamplerId)
                            .to(Sampler::Table, Sampler::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskRun::Table)
                    .col(
                        ColumnDef::new(TaskRun::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskRun::TaskId).integer().not_null())
                    .col(ColumnDef::new(TaskRun::ExecutorId).integer().not_null())
                    .col(ColumnDef::new(TaskRun::Attempt).integer().not_null())
                    .col(ColumnDef::new(TaskRun::RunTimeEnv).json().null())
                    .col(
                        ColumnDef::new(TaskRun::TaskRunStatus)
                            .custom(TaskRunStatus::name())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskRun::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TaskRun::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(TaskRun::ErrorMessage).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskRun::Table, TaskRun::TaskId)
                            .to(Task::Table, Task::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskRun::Table, TaskRun::ExecutorId)
                            .to(Executor::Table, Executor::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx_task_run_task_id_attempt")
                            .col(TaskRun::TaskId)
                            .col(TaskRun::Attempt)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_status")
                    .table(Task::Table)
                    .col(Task::TaskStatus)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Executor::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AV::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scenario::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Map::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Simulator::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sampler::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Plan::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TaskRun::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AV {
    Table,
    Id,
    Name,
    ImagePath,
    ConfigPath,
    NvRuntime,
    CarlaRuntime,
    RosRuntime,
}

#[derive(DeriveIden)]
enum Simulator {
    Table,
    Id,
    Name,
    ImagePath,
    ConfigPath,
    NvRuntime,
    CarlaRuntime,
    RosRuntime,
}

#[derive(DeriveIden)]
enum Sampler {
    Table,
    Id,
    Name,
    ModulePath,
    ConfigPath,
}

#[derive(DeriveIden)]
enum Map {
    Table,
    Id,
    Name,
    XodrPath,
    OsmPath,
}

#[derive(DeriveIden)]
enum Scenario {
    Table,
    Id,
    Type,
    Title,
    ScenarioPath,
    GoalConfig,
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "scenario_type")]
enum ScenarioType {
    #[sea_orm(string_value = "OpenScenario1.0")]
    OpenScenario1,
    #[sea_orm(string_value = "OpenScenario2.0")]
    OpenScenario2,
    #[sea_orm(string_value = "CarlaLBRoute")]
    CarlaLBRoute,
}

#[derive(DeriveIden)]
enum Plan {
    Table,
    Id,
    Name,
    MapId,
    ScenarioId,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    PlanId,
    AvId,
    SimulatorId,
    SamplerId,
    TaskStatus,
    CreatedAt,
    RetryCount,
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
enum TaskStatus {
    #[sea_orm(string_value = "created")]
    Created,
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "invalid")]
    Invalid,
}

#[derive(DeriveIden)]
enum TaskRun {
    Table,
    Id,
    TaskId,
    ExecutorId,
    RunTimeEnv,
    Attempt,
    TaskRunStatus,
    StartedAt,
    FinishedAt,
    ErrorMessage,
}

#[derive(DeriveIden)]
enum Executor {
    Table,
    Id,
    SlurmJobId,
    SlurmArrayId,
    SlurmNodeList,
    Hostname,
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_run_status")]
enum TaskRunStatus {
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "aborted")]
    Aborted,
}
