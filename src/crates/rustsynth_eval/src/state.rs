//! Evaluation state — transform matrix, HSV colour, alpha, per-rule depth, and seed.
//!
//! The `State` is threaded through every rule invocation and cloned into each
//! new branch.  It matches the semantics of the legacy `State.cpp`.

use std::collections::HashMap;

use rustsynth_core::color::Hsva;
use rustsynth_core::math::Mat4;

/// The per-object execution state threaded through the evaluator.
///
/// Matches legacy `StructureSynth::Model::State`:
/// - `transform` corresponds to `State::matrix`
/// - `color` corresponds to `State::hsv` + `State::alpha`
/// - `max_depths` corresponds to `State::maxDepths`
/// - `seed` corresponds to `State::seed`
#[derive(Debug, Clone)]
pub struct State {
    /// Accumulated world transform (column-major, f32).
    pub transform: Mat4,
    /// Current HSVA colour.  Default is fully-saturated red (h=0°, s=1, v=1, a=1),
    /// matching the legacy initial state.
    pub color: Hsva,
    /// Per-rule remaining depth budget.  `None` means the rule has not been entered
    /// on this branch yet.  `0` means exhausted.
    pub max_depths: HashMap<String, i32>,
    /// Branch RNG seed (0 = inherit global seed).
    pub seed: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            // Legacy default: hue=0°, sat=1.0, val=1.0, alpha=1.0 (fully-saturated red)
            color: Hsva::new(0.0, 1.0, 1.0, 1.0),
            max_depths: HashMap::new(),
            seed: 0,
        }
    }
}
