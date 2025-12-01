//! Error types for the trace analyzer.

use thiserror::Error;

/// Errors that can occur when working with coverage data.
#[derive(Error, Debug)]
pub enum CoverageError {
    #[error("Failed to open coverage database: {0}")]
    DatabaseOpen(#[from] rusqlite::Error),

    #[error("Invalid coverage database schema: {0}")]
    InvalidSchema(String),

    #[error("Coverage database not found: {path}")]
    NotFound { path: String },
}

/// Errors that can occur when working with scenario metadata.
#[derive(Error, Debug)]
pub enum ScenarioError {
    #[error("Failed to read scenarios file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse scenarios JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Scenarios file not found: {path}")]
    NotFound { path: String },

    #[error("Unsupported scenarios file version: {version}")]
    UnsupportedVersion { version: String },
}
