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
    Triangle,
}
