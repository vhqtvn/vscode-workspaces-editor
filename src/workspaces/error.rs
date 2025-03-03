use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum WorkspaceError {
    #[error("Failed to determine home directory")]
    HomeDir,
    #[error("Failed to read workspace file: {0}")]
    Read(String),
    #[error("Failed to parse workspace file: {0}")]
    Parse(String),
    #[error("Failed to access database: {0}")]
    Database(String),
    #[error("Failed to write workspace file: {0}")]
    Write(String),
} 