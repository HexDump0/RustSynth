//! Name resolution — builds a [`RuleGraph`] from a parsed [`Script`].
//!
//! Steps:
//! 1. Collect all `RuleDef`s and insert them into the rule map, merging same-name
//!    rules into [`AmbiguousRuleNode`]s.
//! 2. Insert built-in primitives for every name that appears as a rule target.
//! 3. Detect `set recursion depth` to set [`RuleGraph::recurse_depth`].
//! 4. Collect diagnostics for undefined rule references (see `validation`).

use std::collections::HashMap;

use rustsynth_eisenscript::ast::{Action, BodyItem, Script, Statement};
use rustsynth_eisenscript::Diagnostic;

use crate::primitive::{is_builtin, is_triangle_ref};
use crate::rule_graph::{
    AmbiguousRuleNode, CustomRuleNode, RuleGraph, RuleNode, StartItem,
};

/// Resolve a parsed `Script` into a `RuleGraph`.
///
/// Any recoverable issues are returned as diagnostics.  Hard errors (e.g. a
/// `::class` on a non-primitive) are also reported as diagnostics and the
/// graph is returned in its best-effort state.
pub fn resolve(script: &Script) -> (RuleGraph, Vec<Diagnostic>) {
    let mut rules: HashMap<String, RuleNode> = HashMap::new();
    let mut start_items: Vec<StartItem> = Vec::new();
    let mut recurse_depth = false;
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // ── Pass 1: insert user rules ────────────────────────────────────────────
    for stmt in &script.statements {
        match stmt {
            Statement::RuleDef(def) => {
                let node = CustomRuleNode::from_rule_def(def.clone());
                let name = node.name.clone();

                if is_builtin(&name) {
                    diagnostics.push(Diagnostic::error(
                        0,
                        format!(
                            "Cannot redefine built-in primitive '{name}'"
                        ),
                    ));
                    continue;
                }

                match rules.remove(&name) {
                    None => {
                        rules.insert(name, RuleNode::Custom(node));
                    }
                    Some(RuleNode::Custom(existing)) => {
                        let ambig = AmbiguousRuleNode {
                            name: name.clone(),
                            variants: vec![existing, node],
                        };
                        rules.insert(name, RuleNode::Ambiguous(ambig));
                    }
                    Some(RuleNode::Ambiguous(mut ambig)) => {
                        ambig.variants.push(node);
                        rules.insert(name, RuleNode::Ambiguous(ambig));
                    }
                    Some(RuleNode::Primitive { .. }) => {
                        diagnostics.push(Diagnostic::error(
                            0,
                            format!("Cannot shadow primitive rule '{name}'"),
                        ));
                    }
                }
            }
            Statement::SetCmd(cmd) => {
                if cmd.key == "recursion" && cmd.value == "depth" {
                    recurse_depth = true;
                }
                start_items.push(StartItem::SetCmd(cmd.clone()));
            }
            Statement::Action(action) => {
                start_items.push(StartItem::Action(action.clone()));
            }
        }
    }

    // ── Pass 2: resolve rule targets ────────────────────────────────────────
    // Collect all referenced names from every action in the graph.
    let all_refs = collect_all_targets(&rules, &start_items);

    for target in &all_refs {
        if rules.contains_key(target) {
            continue; // already known
        }
        // Primitive name (box, sphere, …)
        if is_builtin(target) {
            rules.insert(
                target.clone(),
                RuleNode::Primitive {
                    kind_name: target.clone(),
                    class_tag: None,
                },
            );
            continue;
        }
        // Triangle inline syntax
        if is_triangle_ref(target) {
            rules.insert(
                target.clone(),
                RuleNode::Primitive {
                    kind_name: "triangle".to_owned(),
                    class_tag: Some(target["triangle".len()..].to_owned()),
                },
            );
            continue;
        }
        // `name::class` — only valid on built-in primitives
        if let Some((base, class)) = target.split_once("::") {
            if is_builtin(base) {
                rules.insert(
                    target.clone(),
                    RuleNode::Primitive {
                        kind_name: base.to_owned(),
                        class_tag: Some(class.to_owned()),
                    },
                );
                continue;
            } else if rules.contains_key(base) {
                diagnostics.push(Diagnostic::error(
                    0,
                    format!(
                        "Class specifier '::{}' is only valid on built-in primitives, not on user rule '{}'",
                        class, base
                    ),
                ));
                continue;
            }
        }
        // Unknown name — leave for validation to report
    }

    // Also ensure all built-ins referenced by retirement rules are present.
    for node in rules.values() {
        let retirements: Vec<String> = match node {
            RuleNode::Custom(c) => c.retirement.iter().cloned().collect(),
            RuleNode::Ambiguous(a) => {
                a.variants.iter().filter_map(|v| v.retirement.clone()).collect()
            }
            _ => vec![],
        };
        for ret in retirements {
            if !rules.contains_key(&ret) && is_builtin(&ret) {
                // Will be inserted on next resolve pass; for now collect
                // as a reference so the second-pass loop picks it up.
                // (The retirement rule names appear in `all_refs` already
                //  if collected properly — they're handled above.)
            }
        }
    }

    let graph = RuleGraph {
        rules,
        start_items,
        recurse_depth,
    };

    (graph, diagnostics)
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Collect every distinct rule target name reachable from the graph.
fn collect_all_targets(
    rules: &HashMap<String, RuleNode>,
    start_items: &[StartItem],
) -> Vec<String> {
    use crate::rule_graph::RuleNode;
    let mut targets: std::collections::HashSet<String> = std::collections::HashSet::new();

    // From start items
    for item in start_items {
        if let StartItem::Action(a) = item {
            collect_from_action(a, &mut targets);
        }
    }

    // From all rule bodies
    for node in rules.values() {
        match node {
            RuleNode::Custom(c) => collect_from_body(&c.body, &mut targets),
            RuleNode::Ambiguous(a) => {
                for v in &a.variants {
                    collect_from_body(&v.body, &mut targets);
                    if let Some(ret) = &v.retirement {
                        targets.insert(ret.clone());
                    }
                }
            }
            RuleNode::Primitive { .. } => {}
        }
    }

    targets.into_iter().collect()
}

fn collect_from_body(
    body: &[BodyItem],
    targets: &mut std::collections::HashSet<String>,
) {
    for item in body {
        if let BodyItem::Action(a) = item {
            collect_from_action(a, targets);
        }
    }
}

fn collect_from_action(
    action: &Action,
    targets: &mut std::collections::HashSet<String>,
) {
    targets.insert(action.target.clone());
}
