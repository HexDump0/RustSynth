//! EisenScript AST node types.

/// Top-level script.
#[derive(Debug, Clone)]
pub struct Script {
    pub statements: Vec<Statement>,
}

/// A top-level statement.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A rule definition: `rule <name> [modifiers] { ... }`
    RuleDef(RuleDef),
    /// A set command: `set <key> <value>`
    SetCmd(SetCmd),
    /// A top-level action (rule invocation with optional loops).
    Action(Action),
}

/// A rule definition.
#[derive(Debug, Clone)]
pub struct RuleDef {
    pub name: String,
    /// Rule selection weight (default 1.0).
    pub weight: f64,
    /// Optional per-invocation max depth.
    pub max_depth: Option<u32>,
    /// Retirement rule name — invoked when max depth is exhausted.
    pub retirement: Option<String>,
    pub body: Vec<BodyItem>,
}

/// An item inside a rule body.
#[derive(Debug, Clone)]
pub enum BodyItem {
    Action(Action),
    SetCmd(SetCmd),
}

/// A rule action: zero or more `N * { transforms }` loops followed by a target name.
///
/// - Bare `rulename` → `loops: []`
/// - `{ transforms } rulename` → one loop with count 1
/// - `N * { transforms } rulename` → one loop with count N
/// - Multiple loops are cartesian-expanded: all (c₀, c₁, …) combos are pushed.
#[derive(Debug, Clone)]
pub struct Action {
    pub loops: Vec<TransformLoop>,
    /// Target rule or primitive name (may contain `::class` suffix or `triangle[…]` payload).
    pub target: String,
}

/// A single transform loop: `N * { transforms }`.
#[derive(Debug, Clone)]
pub struct TransformLoop {
    pub count: u32,
    pub transforms: Vec<TransformOp>,
}

/// A set command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCmd {
    pub key: String,
    pub value: String,
}

/// A single EisenScript transform operation.
///
/// Rotations are applied around the center of the unit cube (0.5, 0.5, 0.5),
/// matching the legacy Structure Synth pivot convention.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformOp {
    /// Translate along X.
    X(f64),
    /// Translate along Y.
    Y(f64),
    /// Translate along Z.
    Z(f64),
    /// Rotate about X axis, degrees. Pivot at (0, 0.5, 0.5).
    Rx(f64),
    /// Rotate about Y axis, degrees. Pivot at (0.5, 0, 0.5).
    Ry(f64),
    /// Rotate about Z axis, degrees. Pivot at (0.5, 0.5, 0).
    Rz(f64),
    /// Non-uniform (or uniform) scale. `s x` → `S { x, y: x, z: x }`.
    /// Scale is about the center (0.5, 0.5, 0.5).
    S { x: f64, y: f64, z: f64 },
    /// Flip X: `S { x: -1, y: 1, z: 1 }`.
    Fx,
    /// Flip Y: `S { x: 1, y: -1, z: 1 }`.
    Fy,
    /// Flip Z: `S { x: 1, y: 1, z: -1 }`.
    Fz,
    /// Plane reflection about an arbitrary normal.
    Reflect { nx: f64, ny: f64, nz: f64 },
    /// Arbitrary 3×3 matrix (row-major, 9 values). Applied about (0.5, 0.5, 0.5).
    Matrix([f64; 9]),
    /// Additive hue shift in degrees.
    Hue(f64),
    /// Multiplicative saturation scale factor.
    Sat(f64),
    /// Multiplicative brightness (value) scale factor.
    Brightness(f64),
    /// Multiplicative alpha scale factor.
    Alpha(f64),
    /// Set absolute color. Value is a CSS hex string (e.g. `#ff0000`) or `"random"`.
    Color(String),
    /// Blend current color towards `color` by `strength`.
    Blend { color: String, strength: f64 },
}
