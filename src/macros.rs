macro_rules! record_start {
    ($next_id:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::GAME_RECORDER;
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.start_record($next_id);
        }
    };
}

macro_rules! record_stop {
    () => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::GAME_RECORDER;
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.save_record();
        }
    };
}

macro_rules! record_replay {
    ($next_id:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::GAME_RECORDER;
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.start_replay($next_id);
        }
    };
}

macro_rules! record_compare_replay {
    () => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::GAME_RECORDER;
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.compare_replay();
        }
    };
}

macro_rules! record_event_start {
    () => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.record_event(Start);
        }
    };
}

macro_rules! record_event_end {
    ($from_game_over:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.record_event(End {
                from_game_over: $from_game_over,
            });
        }
    };
}

macro_rules! record_event_new_rand {
    ($rng_id:expr, $seed:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
            let mut guard = GAME_RECORDER.lock().unwrap();
            guard.record_event(NewRand {
                rng_id: $rng_id,
                seed: $seed,
            });
        }
    };
}

macro_rules! record_event_set_rand_seed {
    ($rng_id:expr, $seed:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            if let Some(rng_id) = $rng_id {
                use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
                let mut guard = GAME_RECORDER.lock().unwrap();
                guard.record_event(SetRandSeed {
                    rng_id,
                    seed: $seed,
                });
            }
        }
    };
}

macro_rules! record_event_rand_f32 {
    ($rng_id:expr, $high:expr, $value:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            if let Some(rng_id) = $rng_id {
                use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
                let mut guard = GAME_RECORDER.lock().unwrap();
                guard.record_event(RandF32 {
                    rng_id,
                    high: $high,
                    value: $value,
                });
            }
        }
    };
}

macro_rules! record_event_rand_signed_f32 {
    ($rng_id:expr, $high:expr, $value:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            if let Some(rng_id) = $rng_id {
                use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
                let mut guard = GAME_RECORDER.lock().unwrap();
                guard.record_event(RandSignedF32 {
                    rng_id,
                    high: $high,
                    value: $value,
                });
            }
        }
    };
}

macro_rules! record_event_rand_usize {
    ($rng_id:expr, $high:expr, $value:expr) => {
        #[cfg(feature = "game_recorder")]
        {
            if let Some(rng_id) = $rng_id {
                use crate::game_recorder::{GameEvent::*, GAME_RECORDER};
                let mut guard = GAME_RECORDER.lock().unwrap();
                guard.record_event(RandUsize {
                    rng_id,
                    high: $high,
                    value: $value,
                });
            }
        }
    };
}

#[cfg(feature = "game_recorder")]
macro_rules! record_next_id {
    () => {{
        use crate::game_recorder::GAME_RECORDER;
        let mut guard = GAME_RECORDER.lock().unwrap();
        guard.next_id()
    }};
}
