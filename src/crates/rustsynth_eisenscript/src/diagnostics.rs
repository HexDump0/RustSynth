//! Diagnostic messages produced during preprocessing, lexing, and parsing.

/// Severity of a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// A single diagnostic message with source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub line: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, line, message: message.into() }
    }

    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Self { severity: Severity::Error, line, message: message.into() }
    }
}
