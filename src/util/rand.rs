use rand::XorShiftRng;
use rand_core::{RngCore, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rand {
    pub rng: XorShiftRng,
    #[cfg(feature = "game_recorder")]
    pub rng_id: Option<usize>,
}

impl Rand {
    pub fn new(seed: u64) -> Self {
        {
            let rng = XorShiftRng::seed_from_u64(seed);
            #[cfg(feature = "game_recorder")]
            let rng_id = record_next_id!();
            record_event_new_rand!(rng_id, seed);
            Rand {
                rng,
                #[cfg(feature = "game_recorder")]
                rng_id: Some(rng_id),
            }
        }
    }

    pub fn new_not_recorded(seed: u64) -> Self {
        let rng = XorShiftRng::seed_from_u64(seed);
        Rand {
            rng,
            #[cfg(feature = "game_recorder")]
            rng_id: None,
        }
    }

    pub fn rand_seed() -> u64 {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH);
        duration
            .map(|d| d.as_secs() * 1000 + u64::from(d.subsec_millis()))
            .unwrap_or_else(|_| 0)
    }

    pub fn set_seed(&mut self, seed: u64) {
        record_event_set_rand_seed!(self.rng_id, seed);
        self.rng = XorShiftRng::seed_from_u64(seed);
    }

    pub fn gen_f32(&mut self, high: f32) -> f32 {
        let next = self.rng.next_u64();
        let value = next as f32 * high / u64::max_value() as f32;
        record_event_rand_f32!(self.rng_id, high, value);
        value
    }

    pub fn gen_signed_f32(&mut self, high: f32) -> f32 {
        let next = self.rng.next_u64();
        let value = next as f32 * high * 2.0 / u64::max_value() as f32 - high;
        record_event_rand_signed_f32!(self.rng_id, high, value);
        value
    }

    #[cfg(target_pointer_width = "32")]
    pub fn gen_usize(&mut self, high: usize) -> usize {
        let value = if high == 0 {
            0
        } else {
            self.rng.next_u32() as usize % high
        };
        record_event_rand_usize!(self.rng_id, high, value);
        value
    }

    #[cfg(target_pointer_width = "64")]
    pub fn gen_usize(&mut self, high: usize) -> usize {
        let value = if high == 0 {
            0
        } else {
            self.rng.next_u64() as usize % high
        };
        record_event_rand_usize!(self.rng_id, high, value);
        value
    }
}
