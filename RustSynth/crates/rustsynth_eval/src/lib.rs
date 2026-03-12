//! `rustsynth_eval` — execution engine.
//!
//! Expands a resolved rule graph into a stream of canonical scene objects.
//! Supports breadth-first and depth-first recursion, max depth/object limits,
//! weighted ambiguous rule selection, and deterministic seed behavior.

pub mod builder;
pub mod recursion;
pub mod state;
pub mod transform;
