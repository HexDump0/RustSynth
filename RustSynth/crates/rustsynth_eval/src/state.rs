//! Evaluation state — transform matrix, color, alpha, depth, and seed.
//!
//! Placeholder — full implementation in T08/T09.

use rustsynth_core::color::Hsva;
use rustsynth_core::math::Mat4;

/// The per-object execution state threaded through the evaluator.
#[derive(Debug, Clone)]
pub struct State {
    /// Accumulated world transform.
    pub transform: Mat4,
    /// Current HSV color + alpha.
    pub color: Hsva,
    /// Current recursion depth.
    pub depth: u32,
    /// RNG seed for this branch.
    pub seed: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            color: Hsva::default(),
            depth: 0,
            seed: 0,
        }
    }
}
