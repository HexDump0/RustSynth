//! `rustsynth_semantics` — rule graph, name resolution, primitive lookup, and validation.
//!
//! Takes the AST from `rustsynth_eisenscript` and produces a fully resolved
//! [`RuleGraph`] ready for evaluation.
//!
//! ## Pipeline
//! ```text
//! Script  →  resolution::resolve  →  RuleGraph  →  validation::validate  →  diagnostics
//! ```

pub mod primitive;
pub mod resolution;
pub mod rule_graph;
pub mod validation;

pub use rule_graph::{AmbiguousRuleNode, CustomRuleNode, RuleGraph, RuleNode, StartItem};
pub use resolution::resolve;
pub use validation::validate;
