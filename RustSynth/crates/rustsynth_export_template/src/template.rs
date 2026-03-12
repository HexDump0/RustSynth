//! Template representation — loaded from a `.rendertemplate` file.
//!
//! Placeholder — full implementation in T11.

/// A render template loaded from disk.
#[derive(Debug, Clone, Default)]
pub struct Template {
    pub name: String,
    pub content: String,
}
