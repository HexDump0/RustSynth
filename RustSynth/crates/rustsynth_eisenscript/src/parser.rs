//! EisenScript parser.
//!
//! Placeholder skeleton — full recursive-descent parser in T06.

use crate::ast::Script;
use crate::diagnostics::Diagnostic;

/// Result of a parse attempt.
pub struct ParseResult {
    pub script: Script,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse EisenScript source text into an AST.
///
/// This is a stub that returns an empty script. The real implementation
/// will be built in T06 after the preprocessor (T04) and lexer (T05) land.
pub fn parse(_source: &str) -> ParseResult {
    ParseResult {
        script: Script { statements: vec![] },
        diagnostics: vec![],
    }
}
