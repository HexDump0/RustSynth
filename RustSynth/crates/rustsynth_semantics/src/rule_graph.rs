//! Rule graph — the validated, resolved semantic representation of an EisenScript program.
//!
//! The rule graph is produced by [`crate::resolution::resolve`] from the parsed AST.
//! It stores:
//! - named rules (custom, ambiguous, or primitive)
//! - the top-level body items (start actions + set commands)
//! - settings gathered from top-level set commands
//! - whether depth-first recursion mode was requested

use std::collections::HashMap;
use rustsynth_eisenscript::ast::{Action, BodyItem, RuleDef, SetCmd};

/// A fully resolved and validated rule graph, ready for evaluation.
#[derive(Debug, Default, Clone)]
pub struct RuleGraph {
    /// All named rules (user-defined, ambiguous, and primitives with class tags).
    pub rules: HashMap<String, RuleNode>,
    /// Top-level items (start actions and set commands) in source order.
    pub start_items: Vec<StartItem>,
    /// Whether `set recursion depth` was encountered.
    pub recurse_depth: bool,
}

impl RuleGraph {
    /// Look up a rule by name.
    pub fn get(&self, name: &str) -> Option<&RuleNode> {
        self.rules.get(name)
    }
}

/// A top-level item from the parsed script.
#[derive(Debug, Clone)]
pub enum StartItem {
    Action(Action),
    SetCmd(SetCmd),
}

/// A resolved rule — either a user-defined custom rule, a set of ambiguous
/// weighted alternatives, or a built-in primitive.
#[derive(Debug, Clone)]
pub enum RuleNode {
    /// A single user-defined rule.
    Custom(CustomRuleNode),
    /// Multiple same-name user rules merged into weighted alternatives.
    Ambiguous(AmbiguousRuleNode),
    /// A built-in primitive (box, sphere, …) with an optional class tag.
    Primitive {
        /// Canonical primitive name: "box", "sphere", etc.
        kind_name: String,
        /// Optional class tag from `name::class` syntax (e.g. `"shiny"`).
        class_tag: Option<String>,
    },
}

/// A single user-defined rule variant.
#[derive(Debug, Clone)]
pub struct CustomRuleNode {
    pub name: String,
    pub weight: f64,
    pub max_depth: Option<u32>,
    pub retirement: Option<String>,
    pub body: Vec<BodyItem>,
}

impl CustomRuleNode {
    pub fn from_rule_def(def: RuleDef) -> Self {
        Self {
            name: def.name,
            weight: def.weight,
            max_depth: def.max_depth,
            retirement: def.retirement,
            body: def.body,
        }
    }
}

/// Multiple weighted alternatives for the same rule name.
#[derive(Debug, Clone)]
pub struct AmbiguousRuleNode {
    pub name: String,
    pub variants: Vec<CustomRuleNode>,
}

impl AmbiguousRuleNode {
    /// Total weight across all variants.
    pub fn total_weight(&self) -> f64 {
        self.variants.iter().map(|v| v.weight).sum()
    }
}
