use rand::XorShiftRng;
use rand_core::{RngCore, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "game_recorder")]
use crate::game_recorder::{record_event, GameEvent};

pub struct Rand {
    pub rng: XorShiftRng,
    #[cfg(feature = "game_recorder")]
    pub rng_id: u32,
}

impl Rand {
    pub fn new(seed: u64) -> Self {
        #[cfg(not(feature = "game_recorder"))]
        {
            let rng = XorShiftRng::seed_from_u64(seed);
            Rand { rng }
        }
        #[cfg(feature = "game_recorder")]
        {
            let mut rng = XorShiftRng::seed_from_u64(seed);
            let rng_id = rng.next_u32();
            record_event(GameEvent::NewRand { rng_id, seed });
            Rand { rng, rng_id }
        }
    }

    pub fn rand_seed() -> u64 {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH);
        duration
            .map(|d| d.as_secs() * 1000 + u64::from(d.subsec_millis()))
            .unwrap_or_else(|_| 0)
    }

    pub fn set_seed(&mut self, seed: u64) {
        #[cfg(feature = "game_recorder")]
        record_event(GameEvent::SetRandSeed {
            rng_id: self.rng_id,
            seed,
        });
        self.rng = XorShiftRng::seed_from_u64(seed);
    }

    pub fn gen_f32(&mut self, high: f32) -> f32 {
        let next = self.rng.next_u64();
        let value = next as f32 * high / u64::max_value() as f32;
        #[cfg(feature = "game_recorder")]
        record_event(GameEvent::RandF32 {
            rng_id: self.rng_id,
            value,
        });
        value
    }

    pub fn gen_signed_f32(&mut self, high: f32) -> f32 {
        let next = self.rng.next_u64();
        let value = next as f32 * high * 2.0 / u64::max_value() as f32 - high;
        #[cfg(feature = "game_recorder")]
        record_event(GameEvent::RandSignedF32 {
            rng_id: self.rng_id,
            value,
        });
        value
    }

    #[cfg(target_pointer_width = "32")]
    pub fn gen_usize(&mut self, high: usize) -> usize {
        let value = if high == 0 {
            0
        } else {
            self.rng.next_u32() as usize % high
        };
        #[cfg(feature = "game_recorder")]
        record_event(GameEvent::RandUsize {
            rng_id: self.rng_id,
            value,
        });
        value
    }

    #[cfg(target_pointer_width = "64")]
    pub fn gen_usize(&mut self, high: usize) -> usize {
        let value = if high == 0 {
            0
        } else {
            self.rng.next_u64() as usize % high
        };
        #[cfg(feature = "game_recorder")]
        record_event(GameEvent::RandUsize {
            rng_id: self.rng_id,
            value,
        });
        value
    }
}
