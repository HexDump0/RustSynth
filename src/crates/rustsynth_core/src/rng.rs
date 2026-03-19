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
    /// Create a new `Rng` from a seed.
    ///
    /// The seed is hashed through one round of splitmix64 before being used as
    /// the xorshift64 state.  This ensures that even small seeds (including 0,
    /// which is a fixed-point for raw xorshift64) produce well-distributed
    /// first outputs.
    pub fn new(seed: u64) -> Self {
        // splitmix64 finaliser — maps any u64 to a non-zero, well-distributed value.
        let mut s = seed.wrapping_add(0x9E3779B97F4A7C15u64);
        s = (s ^ (s >> 30)).wrapping_mul(0xBF58476D1CE4E5B9u64);
        s = (s ^ (s >> 27)).wrapping_mul(0x94D049BB133111EBu64);
        s = s ^ (s >> 31);
        // xorshift64 is a fixed-point at 0; guard against the astronomically
        // unlikely case that splitmix64 returns 0.
        Self { state: if s == 0 { 1 } else { s } }
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
