//! Primitive types that appear in the canonical scene.

/// The kind of geometric primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveKind {
    Box,
    Sphere,
    Cylinder,
    Mesh,
    Line,
    Dot,
    Grid,
    Template,
    /// Inline triangle. The payload is the raw vertex string from the
    /// `triangle[…]` syntax, e.g. `"[0 0 0; 1 0 0; 0 1 0]"`.
    Triangle(String),
}
