use rand::XorShiftRng;
use rand_core::{RngCore, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rand {
    pub rng: XorShiftRng,
}

impl Rand {
    pub fn new(seed: u64) -> Self {
        Rand {
            rng: XorShiftRng::seed_from_u64(seed),
        }
    }

    pub fn rand_seed() -> u64 {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH);
        duration
            .map(|d| d.as_secs() * 1000 + u64::from(d.subsec_millis()))
            .unwrap_or_else(|_| 0)
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
    #[cfg(target_pointer_width = "32")]
    pub fn gen_usize(&mut self, high: usize) -> usize {
        self.rng.next_u32() as usize % high
    }

    // TODO
    #[cfg(target_pointer_width = "64")]
    pub fn gen_usize(&mut self, high: usize) -> usize {
        self.rng.next_u64() as usize % high
    }
}
