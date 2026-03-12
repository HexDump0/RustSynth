//! Recursion mode — breadth-first vs depth-first rule expansion.
//!
//! Placeholder — full implementation in T08.

/// The recursion traversal strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecursionMode {
    #[default]
    BreadthFirst,
    DepthFirst,
}
