//! `rustsynth_eisenscript` — EisenScript preprocessor, lexer, parser, AST, and diagnostics.
//!
//! The pipeline within this crate mirrors the legacy processing order:
//! `source text → preprocessor → tokenizer → parser → AST`

pub mod ast;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod preprocessor;

pub use diagnostics::Diagnostic;
