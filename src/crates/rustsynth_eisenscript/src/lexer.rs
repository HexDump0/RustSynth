//! EisenScript lexer/tokenizer.
//!
//! The implementation mirrors the legacy `Tokenizer` closely so that later
//! parser stages can preserve Structure Synth semantics.

use crate::diagnostics::Diagnostic;

/// Token categories produced by the EisenScript tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Undefined,
    LeftBracket,
    RightBracket,
    MoreThan,
    End,
    Number,
    Multiply,
    UserString,
    Rule,
    Set,
    Operator,
}

/// A lexical token.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub text: String,
    pub float_value: f64,
    pub int_value: i64,
    pub is_integer: bool,
    pub position: usize,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(position: usize, kind: TokenKind, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            float_value: 0.0,
            int_value: 0,
            is_integer: false,
            position,
            kind,
        }
    }

    pub fn numerical_value(&self) -> f64 {
        if self.is_integer {
            self.int_value as f64
        } else {
            self.float_value
        }
    }
}

/// Result of a lexing pass.
#[derive(Debug, Clone)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Tokenize EisenScript source into a token stream.
pub fn lex(source: &str) -> LexResult {
    let normalized = normalize_newlines(source);
    let (chunks, mut diagnostics) = split_chunks(&normalized);
    let mut tokens = Vec::with_capacity(chunks.len() + 1);

    for (chunk, position) in chunks {
        match classify_chunk(&chunk, position) {
            Ok(token) => tokens.push(token),
            Err(message) => diagnostics.push(Diagnostic::error(1, format!("{message} at char {position}"))),
        }
    }

    tokens.push(Token::new(normalized.len(), TokenKind::End, "#END#"));

    LexResult { tokens, diagnostics }
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn split_chunks(source: &str) -> (Vec<(String, usize)>, Vec<Diagnostic>) {
    let chars: Vec<char> = source.chars().collect();
    let mut chunks = Vec::new();
    let mut diagnostics = Vec::new();
    let mut current = String::new();
    let mut current_start = 0usize;
    let mut i = 0usize;
    let mut in_multi_comment = false;
    let mut in_line_comment = false;
    let mut at_line_start = true;

    while i < chars.len() {
        let ch = chars[i];

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                at_line_start = true;
            }
            i += 1;
            continue;
        }

        if in_multi_comment {
            if ch == '*' && chars.get(i + 1) == Some(&'/') {
                in_multi_comment = false;
                i += 2;
                continue;
            }
            if ch == '\n' {
                at_line_start = true;
            }
            i += 1;
            continue;
        }

        if at_line_start && ch == '#' {
            in_line_comment = true;
            i += 1;
            continue;
        }

        if ch == '/' && chars.get(i + 1) == Some(&'/') {
            in_line_comment = true;
            i += 2;
            continue;
        }

        if ch == '/' && chars.get(i + 1) == Some(&'*') {
            in_multi_comment = true;
            i += 2;
            continue;
        }

        if ch == '[' {
            if current.is_empty() {
                current_start = i;
            }
            current.push(ch);
            i += 1;

            let mut found_close = false;
            while i < chars.len() {
                current.push(chars[i]);
                if chars[i] == ']' {
                    found_close = true;
                    i += 1;
                    break;
                }
                i += 1;
            }

            if !found_close {
                diagnostics.push(Diagnostic::error(1, format!("No matching ']' found for '[' at char {current_start}")));
                break;
            }

            chunks.push((std::mem::take(&mut current), current_start));
            at_line_start = false;
            continue;
        }

        if ch == '{' || ch == '}' || ch == ' ' || ch == '\t' || ch == '\n' {
            if !current.is_empty() {
                chunks.push((std::mem::take(&mut current), current_start));
            }
            if ch == '{' || ch == '}' {
                chunks.push((ch.to_string(), i));
            }

            at_line_start = ch == '\n';
            if ch == ' ' || ch == '\t' {
                at_line_start = false;
            }

            i += 1;
            continue;
        }

        if current.is_empty() {
            current_start = i;
        }
        current.push(ch);
        at_line_start = false;
        i += 1;
    }

    if !current.is_empty() {
        chunks.push((current, current_start));
    }

    if in_multi_comment {
        diagnostics.push(Diagnostic::error(1, "Unterminated block comment."));
    }

    (chunks, diagnostics)
}

fn classify_chunk(chunk: &str, position: usize) -> Result<Token, String> {
    let lower = chunk.to_ascii_lowercase();

    let token = match lower.as_str() {
        "rule" => Token::new(position, TokenKind::Rule, chunk),
        "{" => Token::new(position, TokenKind::LeftBracket, chunk),
        "}" => Token::new(position, TokenKind::RightBracket, chunk),
        ">" => Token::new(position, TokenKind::MoreThan, chunk),
        "*" => Token::new(position, TokenKind::Multiply, chunk),
        "set" => Token::new(position, TokenKind::Set, chunk),
        _ if starts_like_number(chunk) => parse_number_token(chunk, position)?,
        _ if is_operator(&lower) => Token::new(position, TokenKind::Operator, canonical_operator(&lower)),
        _ => Token::new(position, TokenKind::UserString, lower),
    };

    Ok(token)
}

fn starts_like_number(chunk: &str) -> bool {
    chunk
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '+' | '-' | '0'..='9'))
}

fn parse_number_token(chunk: &str, position: usize) -> Result<Token, String> {
    if chunk.matches('/').count() == 1 {
        let (numerator, denominator) = chunk.split_once('/').expect("count checked above");
        let numerator = numerator
            .parse::<i64>()
            .map_err(|_| format!("Invalid fraction found: {chunk}"))?;
        let denominator = denominator
            .parse::<i64>()
            .map_err(|_| format!("Invalid fraction found: {chunk}"))?;

        if denominator == 0 {
            return Err(format!("Invalid fraction found: {chunk}"));
        }

        let mut token = Token::new(position, TokenKind::Number, chunk);
        token.float_value = numerator as f64 / denominator as f64;
        token.is_integer = false;
        return Ok(token);
    }

    if let Ok(value) = chunk.parse::<i64>() {
        let mut token = Token::new(position, TokenKind::Number, chunk);
        token.int_value = value;
        token.is_integer = true;
        return Ok(token);
    }

    if let Ok(value) = chunk.parse::<f64>() {
        let mut token = Token::new(position, TokenKind::Number, chunk);
        token.float_value = value;
        token.is_integer = false;
        return Ok(token);
    }

    Err(format!("Invalid symbol found: {chunk}"))
}

fn is_operator(value: &str) -> bool {
    matches!(
        value,
        "c"
            | "reflect"
            | "color"
            | "blend"
            | "a"
            | "alpha"
            | "matrix"
            | "h"
            | "hue"
            | "sat"
            | "b"
            | "brightness"
            | "v"
            | "x"
            | "y"
            | "z"
            | "rx"
            | "ry"
            | "rz"
            | "s"
            | "fx"
            | "fy"
            | "fz"
            | "maxdepth"
            | "weight"
            | "md"
            | "w"
    )
}

fn canonical_operator(value: &str) -> String {
    match value {
        "md" => "maxdepth".to_owned(),
        "w" => "weight".to_owned(),
        "h" => "hue".to_owned(),
        "b" => "brightness".to_owned(),
        "a" => "alpha".to_owned(),
        "c" => "color".to_owned(),
        _ => match value {
            "reflect"
            | "color"
            | "blend"
            | "alpha"
            | "matrix"
            | "hue"
            | "sat"
            | "brightness"
            | "v"
            | "x"
            | "y"
            | "z"
            | "rx"
            | "ry"
            | "rz"
            | "s"
            | "fx"
            | "fy"
            | "fz"
            | "maxdepth"
            | "weight" => value.to_owned(),
            _ => unreachable!("canonical_operator called only for known operators"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BALL_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Ball.es"
    ));
    const MENGER_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Menger.es"
    ));

    #[test]
    fn tokenizes_ball_fixture_with_legacy_kinds() {
        let result = lex(BALL_FIXTURE);

        assert!(result.diagnostics.is_empty());
        assert!(result.tokens.len() > 40);
        assert_eq!(result.tokens[0].kind, TokenKind::Set);
        assert_eq!(result.tokens[1].kind, TokenKind::Operator);
        assert_eq!(result.tokens[1].text, "maxdepth");
        assert_eq!(result.tokens[2].kind, TokenKind::Number);
        assert!(result.tokens[2].is_integer);
        assert_eq!(result.tokens[3].kind, TokenKind::LeftBracket);
        assert_eq!(result.tokens[4].text, "alpha");
        assert_eq!(result.tokens[6].text, "hue");
        assert_eq!(result.tokens[9].kind, TokenKind::UserString);
        assert_eq!(result.tokens[9].text, "r1");
        assert_eq!(result.tokens[10].kind, TokenKind::Rule);
        assert_eq!(result.tokens[11].text, "r1");
        assert_eq!(result.tokens.last().unwrap().kind, TokenKind::End);
    }

    #[test]
    fn parses_fraction_numbers_from_menger() {
        let result = lex(MENGER_FIXTURE);

        assert!(result.diagnostics.is_empty());
        let one_third = result
            .tokens
            .iter()
            .find(|token| token.kind == TokenKind::Number && token.text == "1/3")
            .expect("fraction token");
        assert!(!one_third.is_integer);
        assert!((one_third.float_value - (1.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn strips_comments_and_preprocessor_lines() {
        let source = "#define foo 1\nrule Test { // line comment\n  /* block */ Foo\n}\n";
        let result = lex(source);
        let texts: Vec<_> = result.tokens.iter().map(|token| token.text.as_str()).collect();

        assert!(result.diagnostics.is_empty());
        assert_eq!(texts, vec!["rule", "test", "{", "foo", "}", "#END#"]);
    }

    #[test]
    fn keeps_bracketed_vectors_as_single_tokens() {
        let result = lex("set rotation [1 0 0 0 1 0 0 0 1]");

        assert!(result.diagnostics.is_empty());
        assert_eq!(result.tokens[0].kind, TokenKind::Set);
        assert_eq!(result.tokens[1].text, "rotation");
        assert_eq!(result.tokens[2].kind, TokenKind::UserString);
        assert_eq!(result.tokens[2].text, "[1 0 0 0 1 0 0 0 1]");
    }
}
