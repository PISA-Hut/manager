// src/migrator/m20260214_163021_new_db_schema.rs (create new file)

use sea_orm::ActiveEnum;
use sea_orm::{DbBackend, Schema};
use sea_orm_migration::prelude::*;

use crate::migrator::sea_orm::{DeriveActiveEnum, EnumIter};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260214_163021_new_db_schema"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Worker::Table)
                    .col(
                        ColumnDef::new(Worker::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Worker::JobId).integer().not_null())
                    .col(ColumnDef::new(Worker::NodeList).string().not_null())
                    .col(ColumnDef::new(Worker::Hostname).string().not_null())
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
                    .col(ColumnDef::new(AV::ImagePath).string().not_null())
                    .col(ColumnDef::new(AV::ConfigPath).string().not_null())
                    .col(
                        ColumnDef::new(AV::NvRuntime)
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
                    .col(ColumnDef::new(Scenario::Title).string().null())
                    .col(ColumnDef::new(Scenario::Description).string().null())
                    .col(ColumnDef::new(Scenario::ScenarioPath).string().not_null())
                    .col(ColumnDef::new(Scenario::ParamPath).string().null())
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
                    .col(ColumnDef::new(Simulator::ImagePath).string().not_null())
                    .col(ColumnDef::new(Simulator::ConfigPath).string().not_null())
                    .col(
                        ColumnDef::new(Simulator::NvRuntime)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Simulator::ExtraPorts).json().null())
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

        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<TaskStatus>())
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
                    .col(ColumnDef::new(Task::WorkerId).integer().null())
                    .col(
                        ColumnDef::new(Task::Status)
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
                        ColumnDef::new(Task::ExecutedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Task::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
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
                            .from(Task::Table, Task::WorkerId)
                            .to(Worker::Table, Worker::Id),
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Worker::Table).to_owned())
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
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Worker {
    Table,
    Id,
    JobId,
    NodeList,
    Hostname,
}

#[derive(DeriveIden)]
enum AV {
    Table,
    Id,
    Name,
    ImagePath,
    ConfigPath,
    NvRuntime,
}

#[derive(DeriveIden)]
enum Scenario {
    Table,
    Id,
    Title,
    Description,
    ScenarioPath,
    ParamPath,
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
enum Simulator {
    Table,
    Id,
    Name,
    ImagePath,
    ConfigPath,
    NvRuntime,
    ExtraPorts,
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
    WorkerId,
    Status,
    CreatedAt,
    ExecutedAt,
    FinishedAt,
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
enum TaskStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "in_progress")]
    InProgress,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}
