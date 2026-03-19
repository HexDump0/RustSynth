//! Validation pass — checks the resolved [`RuleGraph`] for semantic errors.
//!
//! Currently checks:
//! - every action target has a corresponding rule in the graph
//! - retirement rule references are defined

use rustsynth_eisenscript::ast::BodyItem;
use rustsynth_eisenscript::Diagnostic;

use crate::rule_graph::{RuleGraph, RuleNode};

/// Validate a resolved [`RuleGraph`] and return any semantic diagnostics.
pub fn validate(graph: &RuleGraph) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check start-item action targets
    for item in &graph.start_items {
        if let crate::rule_graph::StartItem::Action(a) = item {
            check_target(&a.target, graph, &mut diagnostics);
        }
    }

    // Check each rule body
    for node in graph.rules.values() {
        match node {
            RuleNode::Custom(c) => {
                check_body(&c.body, graph, &mut diagnostics);
                if let Some(ret) = &c.retirement {
                    check_target(ret, graph, &mut diagnostics);
                }
            }
            RuleNode::Ambiguous(a) => {
                for v in &a.variants {
                    check_body(&v.body, graph, &mut diagnostics);
                    if let Some(ret) = &v.retirement {
                        check_target(ret, graph, &mut diagnostics);
                    }
                }
            }
            RuleNode::Primitive { .. } => {}
        }
    }

    diagnostics
}

fn check_body(
    body: &[BodyItem],
    graph: &RuleGraph,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for item in body {
        if let BodyItem::Action(a) = item {
            check_target(&a.target, graph, diagnostics);
        }
    }
}

fn check_target(
    target: &str,
    graph: &RuleGraph,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !graph.rules.contains_key(target) {
        diagnostics.push(Diagnostic::error(
            0,
            format!("Undefined rule reference: '{target}'"),
        ));
    }
}
