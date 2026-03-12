//! Built-in primitive rule names.

/// The set of built-in primitive names recognised by the evaluator.
pub const BUILTINS: &[&str] = &[
    "box", "sphere", "cylinder", "mesh", "line", "dot", "grid", "template", "triangle",
];

/// Returns `true` if `name` is a built-in primitive.
pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}

/// Returns `true` if `name` looks like an inline triangle rule reference
/// (`triangle[…]`).
pub fn is_triangle_ref(name: &str) -> bool {
    name.starts_with("triangle[") && name.ends_with(']')
}
