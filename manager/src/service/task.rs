use crate::entity::sea_orm_active_enums::TaskStatus as DbTaskStatus;
use crate::http::dto::task::TaskStatusDto;

impl From<DbTaskStatus> for TaskStatusDto {
    fn from(value: DbTaskStatus) -> Self {
        match value {
            DbTaskStatus::Pending => TaskStatusDto::Pending,
            DbTaskStatus::InProgress => TaskStatusDto::InProgress,
            DbTaskStatus::Completed => TaskStatusDto::Completed,
            DbTaskStatus::Failed => TaskStatusDto::Failed,
        }
    }
}
