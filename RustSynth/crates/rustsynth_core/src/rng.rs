//! Deterministic pseudo-random number generator wrapper.
//!
//! The legacy code uses a Mersenne Twister seeded from the user-visible seed
//! value. This module wraps a compatible LCG/MT-style generator so that seed
//! behavior is explicit and reproducible.
//!
//! The actual MT implementation will be confirmed against the legacy source in T04.

/// A simple, seedable PRNG placeholder.
/// Will be replaced with a validated MT-compatible implementation in T04/T09.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the generator and return the next u64.
    pub fn next_u64(&mut self) -> u64 {
        // Xorshift64 as placeholder — will be replaced with MT in T04.
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Return a float in `[0.0, 1.0)`.
    pub fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }

    /// Return a float in `[lo, hi)`.
    pub fn next_range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut r1 = Rng::new(42);
        let mut r2 = Rng::new(42);
        assert_eq!(r1.next_u64(), r2.next_u64());
    }

    #[test]
    fn range_bounds() {
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            let v = rng.next_range_f64(-5.0, 5.0);
            assert!((-5.0..5.0).contains(&v));
        }
    }
}
