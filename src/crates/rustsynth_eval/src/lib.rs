//! `rustsynth_eval` — execution engine.
//!
//! Expands a resolved [`rustsynth_semantics::RuleGraph`] into a stream of
//! canonical [`rustsynth_scene::Scene`] objects.
//!
//! ## Pipeline
//! ```text
//! RuleGraph + BuildConfig  →  builder::build  →  Scene
//! ```

pub mod builder;
pub mod recursion;
pub mod state;
pub mod transform;

pub use builder::{build, BuildConfig};
pub use recursion::RecursionMode;
pub use state::State;
