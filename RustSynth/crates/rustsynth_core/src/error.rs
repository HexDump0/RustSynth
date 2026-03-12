//! Shared error types for RustSynth.

use thiserror::Error;

/// Top-level error type shared across all RustSynth crates.
#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error: {0}")]
    Parse(String),

    #[error("evaluation error: {0}")]
    Eval(String),

    #[error("export error: {0}")]
    Export(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, Error>;
