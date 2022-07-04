//! Reproducible Random Number Generator
//!
//! There is probabilistic fall back mechanism in offer decoder, but the behavior is highly related to the random number generator which is not reproducible.
//! Here we try to use a specific random number generator to reproduce the result given the random seed, to better debug the algorithm
//! We use Xoroshiro128StarStar from <https://docs.rs/crate/rand_xoshiro/0.6.0/source/src/xoroshiro128starstar.rs>
//! The code is mostly copied here so that I can do the same in JavaScript and test it.

use rand_core::le::read_u64_into;
use rand_core::impls::fill_bytes_via_next;
use rand_core::{RngCore, SeedableRng};
use super::serde::{Serialize, Deserialize};
use super::rand::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Xoroshiro128StarStar {
    s0: u64,
    s1: u64,
}

impl Xoroshiro128StarStar {
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        f64::from_bits(0x3FF << 52 | self.next_u64() >> 12) - 1.
    }

    #[allow(dead_code)]
    pub fn get_s0_i64(&self) -> i64 {
        i64::from_le_bytes(self.s0.to_le_bytes())
    }

    #[allow(dead_code)]
    pub fn get_s1_i64(&self) -> i64 {
        i64::from_le_bytes(self.s1.to_le_bytes())
    }

    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self::seed_from_u64(rng.gen::<u64>())
    }
}

impl RngCore for Xoroshiro128StarStar {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let r = self.s0.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        self.s1 ^= self.s0;
        self.s0 = self.s0.rotate_left(24) ^ self.s1 ^ (self.s1 << 16);
        self.s1 = self.s1.rotate_left(37);
        r
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl SeedableRng for Xoroshiro128StarStar {
    type Seed = [u8; 16];

    /// Create a new `Xoroshiro128StarStar`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    fn from_seed(seed: [u8; 16]) -> Xoroshiro128StarStar {
        if seed.iter().all(|&x| x == 0) {
            return Self::seed_from_u64(0);
        }
        let mut s = [0; 2];
        read_u64_into(&seed, &mut s);
        Self {
            s0: s[0],
            s1: s[1],
        }
    }

    /// Seed a `Xoroshiro128StarStar` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoroshiro128StarStar {
        let mut rng = SplitMix64::seed_from_u64(seed);
        let s0 = rng.next_u64();
        let s1 = rng.next_u64();
        Self {
            s0: s0,
            s1: s1,
        }
    }
}

pub struct SplitMix64 {
    x: u64,
}

impl SplitMix64 {
    #[allow(dead_code)]
    pub fn get_x_i64(&self) -> i64 {
        i64::from_le_bytes(self.x.to_le_bytes())
    }
}

const PHI: u64 = 0x9e3779b97f4a7c15;

impl SeedableRng for SplitMix64 {
    type Seed = [u8; 8];

    /// Create a new `SplitMix64`.
    fn from_seed(seed: [u8; 8]) -> SplitMix64 {
        let mut state = [0; 1];
        read_u64_into(&seed, &mut state);
        SplitMix64 {
            x: state[0],
        }
    }

    /// Seed a `SplitMix64` from a `u64`.
    fn seed_from_u64(seed: u64) -> SplitMix64 {
        SplitMix64::from_seed(seed.to_le_bytes())
    }
}

impl RngCore for SplitMix64 {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.x = self.x.wrapping_add(PHI);
        let mut z = self.x;
        // David Stafford's
        // (http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html)
        // "Mix4" variant of the 64-bit finalizer in Austin Appleby's
        // MurmurHash3 algorithm.
        z = (z ^ (z >> 33)).wrapping_mul(0x62A9D9ED799705F5);
        z = (z ^ (z >> 28)).wrapping_mul(0xCB24D0A5C88C35B3);
        (z >> 32) as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.x = self.x.wrapping_add(PHI);
        let mut z = self.x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl SplitMix64 {
    #[inline]
    #[allow(dead_code)]
    pub fn next_f64(&mut self) -> f64 {
        f64::from_bits(0x3FF << 52 | self.next_u64() >> 12) - 1.
    }

}
