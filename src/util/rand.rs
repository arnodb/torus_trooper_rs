use rand::XorShiftRng;
use rand_core::{RngCore, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rand {
    pub rng: XorShiftRng,
}

impl Rand {
    pub fn new() -> Self {
        Rand {
            rng: XorShiftRng::seed_from_u64(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or_else(|_| 0),
            ),
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rng = XorShiftRng::seed_from_u64(seed);
    }

    // TODO
    pub fn gen_f32(&mut self, high: f32) -> f32 {
        let next = self.rng.next_u64();
        next as f32 * high / u64::max_value() as f32
    }

    // TODO
    pub fn gen_signed_f32(&mut self, high: f32) -> f32 {
        let next = self.rng.next_u64();
        next as f32 * high * 2.0 / u64::max_value() as f32 - high
    }

    // TODO
    pub fn gen_usize(&mut self, high: usize) -> usize {
        let next = self.rng.next_u32();
        next as usize * high / u32::max_value() as usize
    }
}
