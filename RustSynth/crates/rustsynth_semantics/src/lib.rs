//! `rustsynth_semantics` — rule graph, name resolution, primitive lookup, and validation.
//!
//! Takes the AST from `rustsynth_eisenscript` and produces a fully resolved
//! `RuleGraph` ready for evaluation.

pub mod primitive;
pub mod resolution;
pub mod rule_graph;
pub mod validation;
