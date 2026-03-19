//! `rustsynth_core` — shared types, errors, math helpers, deterministic RNG,
//! and common utilities used by every other RustSynth crate.

pub mod color;
pub mod error;
pub mod id;
pub mod math;
pub mod rng;

pub use error::{Error, Result};
