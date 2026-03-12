//! EisenScript AST node types.
//!
//! This is a placeholder skeleton. The full AST will be built in T06.

/// Top-level script.
#[derive(Debug, Clone)]
pub struct Script {
    pub statements: Vec<Statement>,
}

/// A top-level statement in a script.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A rule definition: `rule <name> [modifiers] { ... }`
    RuleDef(RuleDef),
    /// A set command: `set <key> <value>`
    SetCmd(SetCmd),
    /// A top-level rule invocation with optional transform: `[transform] rulename`
    Invocation(Invocation),
}

/// A rule definition.
#[derive(Debug, Clone)]
pub struct RuleDef {
    pub name: String,
    pub weight: Option<f64>,
    pub max_depth: Option<u32>,
    pub retirement: Option<String>,
    pub body: Vec<Action>,
}

/// An action inside a rule body.
#[derive(Debug, Clone)]
pub struct Action {
    pub transforms: Vec<Transform>,
    pub count: Option<u32>,
    pub target: String,
}

/// A set command.
#[derive(Debug, Clone)]
pub struct SetCmd {
    pub key: String,
    pub value: String,
}

/// A top-level invocation.
#[derive(Debug, Clone)]
pub struct Invocation {
    pub transforms: Vec<Transform>,
    pub count: Option<u32>,
    pub target: String,
}

/// A single transform step (placeholder — full enum comes in T05/T06).
#[derive(Debug, Clone)]
pub struct Transform {
    pub op: String,
    pub args: Vec<f64>,
}
