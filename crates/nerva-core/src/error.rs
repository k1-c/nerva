use thiserror::Error;

#[derive(Debug, Error)]
pub enum NervaError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Policy denied: {0}")]
    PolicyDenied(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("OS error: {0}")]
    OsError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
