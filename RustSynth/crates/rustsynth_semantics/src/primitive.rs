//! Built-in primitive rule names.

/// The set of built-in primitive names recognized by the evaluator.
pub const BUILTINS: &[&str] = &[
    "box", "sphere", "cylinder", "mesh", "line", "dot", "grid", "template",
];

/// Returns true if the given name is a built-in primitive.
pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}
