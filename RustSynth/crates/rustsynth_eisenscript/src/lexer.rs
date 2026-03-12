//! EisenScript lexer/tokenizer.
//!
//! Placeholder skeleton — full implementation in T05.

/// A lexical token.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(f64),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Star,
    Slash,
    /// `//` or `/* */` comment (already stripped by the tokenizer)
    Comment,
    Eof,
}
