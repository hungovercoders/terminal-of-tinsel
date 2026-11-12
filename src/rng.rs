//! A tiny, stable random number generator.
//!
//! SplitMix64 is enough for picking facts, and unlike a library RNG its
//! sequence will never change between versions, so the Fact of the Night is
//! the same for everyone on the same release and the next.

use std::time::{SystemTime, UNIX_EPOCH};

/// SplitMix64: fast, tiny, and good enough for a Christmas special.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// A seed from `TERMINAL_OF_TINSEL_SEED` if set (for repeatable tests and
    /// demos), otherwise from the clock.
    pub fn from_env_or_clock() -> Self {
        let seed = std::env::var("TERMINAL_OF_TINSEL_SEED")
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0)
            });
        Self::new(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. Panics if `n` is zero.
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "Rng::below(0)");
        (self.next_u64() % n as u64) as usize
    }

    /// A random element of a non-empty slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(20261224);
        let mut b = Rng::new(20261224);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn sequence_is_pinned() {
        // If this changes, every Fact of the Night changes with it.
        assert_eq!(Rng::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..1000 {
            assert!(r.below(13) < 13);
        }
    }
}
