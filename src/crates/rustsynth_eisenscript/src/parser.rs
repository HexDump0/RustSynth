//! EisenScript recursive-descent parser.
//!
//! Converts a token stream from the lexer into an AST (`Script`).
//! The parse approach mirrors the legacy `EisenParser.cpp` EBNF:
//!
//! ```text
//! program      = { set | rule | action } ;
//! rule         = 'rule' name { modifier } '{' { set | action } '}' ;
//! modifier     = ('weight' float) | ('maxdepth' int ['>' name]) ;
//! action       = '{' transforms '}' name
//!              | name
//!              | (int '*' '{' transforms '}')+ name ;
//! set          = 'set' name value ;
//! transforms   = { operator … } ;
//! ```

use crate::ast::{
    Action, BodyItem, RuleDef, Script, SetCmd, Statement, TransformLoop, TransformOp,
};
use crate::diagnostics::Diagnostic;
use crate::lexer::{lex, Token, TokenKind};

/// Result of a parse attempt.
pub struct ParseResult {
    pub script: Script,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse EisenScript source text into an AST.
///
/// Internally runs the lexer first, then the recursive-descent parser.
pub fn parse(source: &str) -> ParseResult {
    let lex_result = lex(source);
    let mut p = Parser {
        tokens: lex_result.tokens,
        pos: 0,
        diagnostics: lex_result.diagnostics,
    };
    let script = p.parse_script();
    ParseResult {
        script,
        diagnostics: p.diagnostics,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal parser state
// ─────────────────────────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// Advance and return the consumed token (cloned to avoid lifetime issues).
    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    /// Consume the current token if it matches `kind`; return whether it matched.
    fn accept(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(0, msg));
    }

    // ─── Top-level ────────────────────────────────────────────────────────────

    fn parse_script(&mut self) -> Script {
        let mut statements = Vec::new();
        loop {
            match self.peek().kind {
                TokenKind::Rule => {
                    if let Some(r) = self.parse_rule() {
                        statements.push(Statement::RuleDef(r));
                    }
                }
                TokenKind::Set => {
                    statements.push(Statement::SetCmd(self.parse_set_cmd()));
                }
                TokenKind::LeftBracket | TokenKind::UserString | TokenKind::Number => {
                    if let Some(a) = self.parse_action() {
                        statements.push(Statement::Action(a));
                    }
                }
                TokenKind::End => break,
                _ => {
                    let text = self.peek().text.clone();
                    self.error(format!("Unexpected token at top level: {text}"));
                    self.advance();
                }
            }
        }
        Script { statements }
    }

    // ─── Rule ────────────────────────────────────────────────────────────────

    fn parse_rule(&mut self) -> Option<RuleDef> {
        self.advance(); // consume 'rule'

        // Rule name
        if self.peek().kind != TokenKind::UserString {
            let text = self.peek().text.clone();
            self.error(format!("Expected rule name after 'rule', found: {text}"));
            return None;
        }
        let name = self.advance().text;

        let mut weight = 1.0_f64;
        let mut max_depth: Option<u32> = None;
        let mut retirement: Option<String> = None;

        // Parse optional modifiers: weight / maxdepth [> retirement]
        while self.peek().kind == TokenKind::Operator {
            let op = self.peek().text.clone();
            match op.as_str() {
                "weight" => {
                    self.advance();
                    if self.peek().kind == TokenKind::Number {
                        weight = self.advance().numerical_value();
                    } else {
                        let text = self.peek().text.clone();
                        self.error(format!(
                            "'weight' modifier expected a number, found: {text}"
                        ));
                    }
                }
                "maxdepth" => {
                    self.advance();
                    if self.peek().kind == TokenKind::Number && self.peek().is_integer {
                        max_depth = Some(self.advance().int_value as u32);
                        // Optional retirement: > rulename
                        if self.peek().kind == TokenKind::MoreThan {
                            self.advance();
                            if self.peek().kind == TokenKind::UserString {
                                retirement = Some(self.advance().text);
                            } else {
                                let text = self.peek().text.clone();
                                self.error(format!(
                                    "Expected rule name after '>' in maxdepth, found: {text}"
                                ));
                            }
                        }
                    } else {
                        let text = self.peek().text.clone();
                        self.error(format!(
                            "'maxdepth' modifier expected an integer, found: {text}"
                        ));
                    }
                }
                _ => break, // unknown operator → stop modifier parsing
            }
        }

        // '{'
        if !self.accept(TokenKind::LeftBracket) {
            let text = self.peek().text.clone();
            self.error(format!(
                "Expected '{{' after rule header, found: {text}"
            ));
            return Some(RuleDef { name, weight, max_depth, retirement, body: vec![] });
        }

        // Body items
        let mut body = Vec::new();
        loop {
            match self.peek().kind {
                TokenKind::RightBracket | TokenKind::End => break,
                TokenKind::Set => {
                    body.push(BodyItem::SetCmd(self.parse_set_cmd()));
                }
                TokenKind::LeftBracket | TokenKind::UserString | TokenKind::Number => {
                    if let Some(a) = self.parse_action() {
                        body.push(BodyItem::Action(a));
                    }
                }
                _ => {
                    let text = self.peek().text.clone();
                    self.error(format!("Unexpected token in rule body: {text}"));
                    self.advance();
                }
            }
        }

        if !self.accept(TokenKind::RightBracket) {
            self.error("Expected '}' to close rule definition");
        }

        Some(RuleDef { name, weight, max_depth, retirement, body })
    }

    // ─── Set command ─────────────────────────────────────────────────────────

    fn parse_set_cmd(&mut self) -> SetCmd {
        self.advance(); // consume 'set'

        // Key: either the special 'maxdepth' operator token or a UserString
        let key = if self.peek().kind == TokenKind::Operator
            && self.peek().text == "maxdepth"
        {
            self.advance().text
        } else if self.peek().kind == TokenKind::UserString {
            self.advance().text
        } else {
            let text = self.peek().text.clone();
            self.error(format!("Expected setting name after 'set', found: {text}"));
            String::new()
        };

        // Value: consume one token (any kind except End/RightBracket)
        let value = match self.peek().kind {
            TokenKind::End | TokenKind::RightBracket => String::new(),
            _ => self.advance().text,
        };

        SetCmd { key, value }
    }

    // ─── Action ──────────────────────────────────────────────────────────────

    fn parse_action(&mut self) -> Option<Action> {
        match self.peek().kind {
            // `{ transforms } rulename`
            TokenKind::LeftBracket => {
                let transforms = self.parse_transform_list()?;
                if self.peek().kind != TokenKind::UserString {
                    let text = self.peek().text.clone();
                    self.error(format!(
                        "Expected rule name after transform list, found: {text}"
                    ));
                    return None;
                }
                // Handle triangle[payload] or rule::class as a single target token
                let target = self.collect_target();
                Some(Action {
                    loops: vec![TransformLoop { count: 1, transforms }],
                    target,
                })
            }

            // Bare `rulename` (possibly followed by `[payload]` for triangle)
            TokenKind::UserString => {
                let target = self.collect_target();
                Some(Action { loops: vec![], target })
            }

            // `N * { transforms } [M * { transforms }]* rulename`
            TokenKind::Number => {
                let mut loops = Vec::new();
                while self.peek().kind == TokenKind::Number {
                    if !self.peek().is_integer {
                        let text = self.peek().text.clone();
                        self.error(format!(
                            "Expected integer count in transform loop, found: {text}"
                        ));
                        return None;
                    }
                    let count = self.advance().int_value as u32;
                    if !self.accept(TokenKind::Multiply) {
                        let text = self.peek().text.clone();
                        self.error(format!("Expected '*' after count, found: {text}"));
                        return None;
                    }
                    let transforms = self.parse_transform_list()?;
                    loops.push(TransformLoop { count, transforms });
                }
                if self.peek().kind != TokenKind::UserString {
                    let text = self.peek().text.clone();
                    self.error(format!(
                        "Expected rule name after transform loops, found: {text}"
                    ));
                    return None;
                }
                let target = self.collect_target();
                Some(Action { loops, target })
            }

            _ => {
                let text = self.peek().text.clone();
                self.error(format!(
                    "Expected rule action (rule name, '{{', or count), found: {text}"
                ));
                None
            }
        }
    }

    /// Collect a possibly compound target: `rulename` optionally followed by a
    /// `[payload]` token (for `triangle[…]` syntax) with no whitespace in the
    /// source.  Because the lexer merges `triangle[…]` into one token when there
    /// is no space, a standalone `triangle` followed by a `[…]` UserString is
    /// the only two-token case we need to handle.
    fn collect_target(&mut self) -> String {
        let base = self.advance().text; // consume UserString
        // If the lexer produced a separate `[…]` token immediately after (two-token
        // triangle form with a space in the source), merge them.
        if base == "triangle" && self.peek().kind == TokenKind::UserString {
            let next = self.peek().text.clone();
            if next.starts_with('[') {
                self.advance();
                return format!("{base}{next}");
            }
        }
        base
    }

    // ─── Transform list ──────────────────────────────────────────────────────

    fn parse_transform_list(&mut self) -> Option<Vec<TransformOp>> {
        if !self.accept(TokenKind::LeftBracket) {
            let text = self.peek().text.clone();
            self.error(format!(
                "Expected '{{' to start transform list, found: {text}"
            ));
            return None;
        }
        let mut ops = Vec::new();
        while self.peek().kind == TokenKind::Operator {
            match self.parse_transform_op() {
                Some(op) => ops.push(op),
                None => break,
            }
        }
        if !self.accept(TokenKind::RightBracket) {
            let text = self.peek().text.clone();
            self.error(format!(
                "Expected '}}' to end transform list, found: {text}"
            ));
        }
        Some(ops)
    }

    fn parse_transform_op(&mut self) -> Option<TransformOp> {
        let op = self.advance().text; // consume operator token
        match op.as_str() {
            "x" => Some(TransformOp::X(self.require_number("x")?)),
            "y" => Some(TransformOp::Y(self.require_number("y")?)),
            "z" => Some(TransformOp::Z(self.require_number("z")?)),
            "rx" => Some(TransformOp::Rx(self.require_number("rx")?)),
            "ry" => Some(TransformOp::Ry(self.require_number("ry")?)),
            "rz" => Some(TransformOp::Rz(self.require_number("rz")?)),
            "s" => {
                let x = self.require_number("s")?;
                // `s x` (uniform) or `s x y z` (non-uniform)
                if self.peek().kind == TokenKind::Number {
                    let y = self.advance().numerical_value();
                    let z = self.require_number("s z")?;
                    Some(TransformOp::S { x, y, z })
                } else {
                    Some(TransformOp::S { x, y: x, z: x })
                }
            }
            "fx" => Some(TransformOp::Fx),
            "fy" => Some(TransformOp::Fy),
            "fz" => Some(TransformOp::Fz),
            "reflect" => {
                let nx = self.require_number("reflect nx")?;
                let ny = self.require_number("reflect ny")?;
                let nz = self.require_number("reflect nz")?;
                Some(TransformOp::Reflect { nx, ny, nz })
            }
            "matrix" => {
                let mut vals = [0.0_f64; 9];
                for (i, val) in vals.iter_mut().enumerate() {
                    *val = self.require_number(&format!("matrix[{i}]"))?;
                }
                Some(TransformOp::Matrix(vals))
            }
            "hue" => Some(TransformOp::Hue(self.require_number("hue")?)),
            "sat" => Some(TransformOp::Sat(self.require_number("sat")?)),
            // `brightness` (canonical from `b`) and legacy alias `v`
            "brightness" | "v" => {
                Some(TransformOp::Brightness(self.require_number("brightness")?))
            }
            "alpha" => Some(TransformOp::Alpha(self.require_number("alpha")?)),
            "color" => {
                // Accept a UserString (hex color or "random")
                if self.peek().kind == TokenKind::UserString {
                    Some(TransformOp::Color(self.advance().text))
                } else {
                    self.error("'color' transform expected a color value");
                    None
                }
            }
            "blend" => {
                if self.peek().kind != TokenKind::UserString {
                    self.error("'blend' transform expected a color value as first argument");
                    return None;
                }
                let color = self.advance().text;
                let strength = self.require_number("blend strength")?;
                Some(TransformOp::Blend { color, strength })
            }
            // maxdepth / weight can appear as operator tokens but are not valid
            // inside a transform list — skip with a warning.
            other => {
                self.error(format!("Unknown transform operator: {other}"));
                None
            }
        }
    }

    fn require_number(&mut self, context: &str) -> Option<f64> {
        if self.peek().kind == TokenKind::Number {
            Some(self.advance().numerical_value())
        } else {
            let text = self.peek().text.clone();
            self.error(format!("'{context}' expected a number, found: {text}"));
            None
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BodyItem, Statement, TransformOp};

    // ── fixture helpers ────────────────────────────────────────────────────

    const BALL_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Ball.es"
    ));

    const MENGER_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Menger.es"
    ));

    const DEFAULT_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Default.es"
    ));

    // ── helpers ────────────────────────────────────────────────────────────

    fn rules(script: &Script) -> Vec<&RuleDef> {
        script
            .statements
            .iter()
            .filter_map(|s| if let Statement::RuleDef(r) = s { Some(r) } else { None })
            .collect()
    }

    fn set_cmds(script: &Script) -> Vec<&SetCmd> {
        script
            .statements
            .iter()
            .filter_map(|s| if let Statement::SetCmd(c) = s { Some(c) } else { None })
            .collect()
    }

    fn top_actions(script: &Script) -> Vec<&Action> {
        script
            .statements
            .iter()
            .filter_map(|s| if let Statement::Action(a) = s { Some(a) } else { None })
            .collect()
    }

    // ── Ball ──────────────────────────────────────────────────────────────

    #[test]
    fn ball_parses_without_errors() {
        let result = parse(BALL_FIXTURE);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn ball_has_top_level_action_and_set_maxdepth() {
        let result = parse(BALL_FIXTURE);
        let script = &result.script;

        // set maxdepth 2000
        let sets = set_cmds(script);
        assert!(!sets.is_empty());
        assert!(sets.iter().any(|c| c.key == "maxdepth" && c.value == "2000"));

        // top-level action: { a 0.9 hue 30 } R1
        let actions = top_actions(script);
        assert_eq!(actions.len(), 1);
        let a = actions[0];
        assert_eq!(a.target, "r1");
        assert_eq!(a.loops.len(), 1);
        assert_eq!(a.loops[0].count, 1);
        let ops = &a.loops[0].transforms;
        assert_eq!(ops[0], TransformOp::Alpha(0.9));
        assert_eq!(ops[1], TransformOp::Hue(30.0));
    }

    #[test]
    fn ball_has_two_ambiguous_r1_rules_with_weight() {
        let result = parse(BALL_FIXTURE);
        let rs: Vec<_> = rules(&result.script)
            .into_iter()
            .filter(|r| r.name == "r1")
            .collect();
        assert_eq!(rs.len(), 2, "Ball should have two R1 rules");
        for r in &rs {
            assert!((r.weight - 10.0).abs() < 1e-9, "Both R1 rules should have weight 10");
        }
    }

    // ── Menger ────────────────────────────────────────────────────────────

    #[test]
    fn menger_parses_without_errors() {
        let result = parse(MENGER_FIXTURE);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn menger_r1_has_maxdepth_and_retirement() {
        let result = parse(MENGER_FIXTURE);
        let rs = rules(&result.script);
        let r1 = rs.iter().find(|r| r.name == "r1").expect("R1 rule");
        assert_eq!(r1.max_depth, Some(3));
        assert_eq!(r1.retirement.as_deref(), Some("c2"));
    }

    #[test]
    fn menger_uses_fraction_scale() {
        let result = parse(MENGER_FIXTURE);
        // Find a body action with s 1/3
        let rs = rules(&result.script);
        let r1 = rs.iter().find(|r| r.name == "r1").expect("R1 rule");
        let found = r1.body.iter().any(|item| {
            if let BodyItem::Action(a) = item {
                a.loops.iter().any(|l| {
                    l.transforms.iter().any(|op| {
                        if let TransformOp::S { x, y, z } = op {
                            (*x - 1.0 / 3.0).abs() < 1e-9
                                && (*y - 1.0 / 3.0).abs() < 1e-9
                                && (*z - 1.0 / 3.0).abs() < 1e-9
                        } else {
                            false
                        }
                    })
                })
            } else {
                false
            }
        });
        assert!(found, "R1 should contain s 1/3 uniform scale");
    }

    // ── Default ───────────────────────────────────────────────────────────

    #[test]
    fn default_parses_without_errors() {
        let result = parse(DEFAULT_FIXTURE);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn default_camera_set_commands() {
        let result = parse(DEFAULT_FIXTURE);
        let sets = set_cmds(&result.script);
        assert!(sets.iter().any(|c| c.key == "translation"), "missing translation");
        assert!(sets.iter().any(|c| c.key == "rotation"), "missing rotation");
        assert!(sets.iter().any(|c| c.key == "pivot"), "missing pivot");
        assert!(sets.iter().any(|c| c.key == "scale"), "missing scale");
        assert!(sets.iter().any(|c| c.key == "background"), "missing background");
        assert!(sets.iter().any(|c| c.key == "maxdepth"), "missing maxdepth");
    }

    #[test]
    fn default_r0_uses_multiply_syntax() {
        let result = parse(DEFAULT_FIXTURE);
        let rs = rules(&result.script);
        let r0 = rs.iter().find(|r| r.name == "r0").expect("r0 rule");
        // rule r0 { 3 * { rz 120 } R1   3 * { rz 120 } R2 }
        let actions: Vec<_> = r0
            .body
            .iter()
            .filter_map(|b| if let BodyItem::Action(a) = b { Some(a) } else { None })
            .collect();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].loops[0].count, 3);
        assert_eq!(actions[0].loops[0].transforms[0], TransformOp::Rz(120.0));
        assert_eq!(actions[0].target, "r1");
    }

    #[test]
    fn default_r1_has_sphere_shiny_target() {
        let result = parse(DEFAULT_FIXTURE);
        let rs = rules(&result.script);
        let r1 = rs.iter().find(|r| r.name == "r1").expect("R1 rule");
        let shiny = r1.body.iter().any(|b| {
            if let BodyItem::Action(a) = b {
                a.target.contains("sphere")
            } else {
                false
            }
        });
        assert!(shiny, "R1 should reference sphere or sphere::shiny");
    }

    // ── Edge cases ────────────────────────────────────────────────────────

    #[test]
    fn bare_rule_name_action() {
        let result = parse("rule T { box }");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let rs = rules(&result.script);
        assert_eq!(rs.len(), 1);
        let body = &rs[0].body;
        assert_eq!(body.len(), 1);
        if let BodyItem::Action(a) = &body[0] {
            assert!(a.loops.is_empty());
            assert_eq!(a.target, "box");
        } else {
            panic!("expected action");
        }
    }

    #[test]
    fn non_uniform_scale_parses() {
        let result = parse("rule T { { s 1 2 3 } box }");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let rs = rules(&result.script);
        if let BodyItem::Action(a) = &rs[0].body[0] {
            assert_eq!(
                a.loops[0].transforms[0],
                TransformOp::S { x: 1.0, y: 2.0, z: 3.0 }
            );
        }
    }

    #[test]
    fn color_and_blend_parse() {
        let result = parse("rule T { { color #ff0000 blend #00ff00 0.5 } sphere }");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let rs = rules(&result.script);
        if let BodyItem::Action(a) = &rs[0].body[0] {
            assert_eq!(a.loops[0].transforms[0], TransformOp::Color("#ff0000".to_owned()));
            assert_eq!(
                a.loops[0].transforms[1],
                TransformOp::Blend {
                    color: "#00ff00".to_owned(),
                    strength: 0.5
                }
            );
        }
    }

    #[test]
    fn set_inside_rule_parses() {
        let result = parse("rule T { set background #000 box }");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let rs = rules(&result.script);
        assert!(rs[0].body.iter().any(|b| matches!(b, BodyItem::SetCmd(_))));
    }
}

