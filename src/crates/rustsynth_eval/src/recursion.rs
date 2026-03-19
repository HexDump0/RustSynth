//! Recursion mode — breadth-first vs depth-first rule expansion.
//!
//! Placeholder — full implementation in T08.

use serde::{Deserialize, Serialize};

/// The recursion traversal strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecursionMode {
    #[default]
    BreadthFirst,
    DepthFirst,
}
