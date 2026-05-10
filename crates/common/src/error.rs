use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("authentication failed")]
    Unauthorized,
    #[error("permission denied: missing {0}")]
    Forbidden(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("external provider error: {0}")]
    External(String),
    #[error("internal error: {0}")]
    Internal(String),
}
