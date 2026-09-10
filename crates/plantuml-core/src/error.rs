//! Error types for the plantuml library.
//!
//! Ported from: net/sourceforge/plantuml (project-wide error conventions).

use thiserror::Error;

/// Top-level error enum for all plantuml library operations.
#[derive(Debug, Error)]
pub enum PlantumlError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("preprocessor error: {0}")]
    Preproc(String),

    #[error("rendering error: {0}")]
    Render(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("include resolution failed: {0}")]
    Include(String),
}

impl PlantumlError {
    /// Wrap a generic parse failure message.
    #[must_use]
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Wrap an invalid-input failure message.
    #[must_use]
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}
