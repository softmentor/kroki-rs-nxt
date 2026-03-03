//! Typed error hierarchy for diagram operations.

use thiserror::Error;

/// Result type alias for diagram operations.
pub type DiagramResult<T> = Result<T, DiagramError>;

/// Errors that can occur during diagram generation.
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Execution timeout: {tool} exceeded {timeout_ms}ms")]
    ExecutionTimeout { tool: String, timeout_ms: u64 },

    #[error("Process failed: {0}")]
    ProcessFailed(String),

    #[error("Unsupported format '{format}' for provider '{provider}'")]
    UnsupportedFormat { format: String, provider: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
