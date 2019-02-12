use std::borrow::Cow;
use std::sync::Mutex;

#[derive(Default)]
pub struct GameRecorder<'a> {
    replay: bool,
    saved: Vec<GameEvent<'a>>,
    running: Vec<GameEvent<'a>>,
}

#[derive(PartialEq, Debug)]
pub enum GameEvent<'a> {
    Start,
    End { from_game_over: bool },
    NewRand { rng_id: u32, seed: u64 },
    SetRandSeed { rng_id: u32, seed: u64 },
    RandF32 { rng_id: u32, value: f32 },
    RandSignedF32 { rng_id: u32, value: f32 },
    RandUsize { rng_id: u32, value: usize },
    Custom(Cow<'a, str>),
}

impl<'a> GameRecorder<'a> {
    pub fn start_record(&mut self) {
        self.replay = false;
        self.saved.clear();
        self.running.clear();
    }
    pub fn save_record(&mut self) {
        std::mem::swap(&mut self.saved, &mut self.running);
    }
    pub fn start_replay(&mut self) {
        self.replay = true;
        self.running.clear();
    }
    pub fn record_event(&mut self, event: GameEvent<'a>) {
        self.running.push(event);
        if self.replay {
            let index = self.running.len() - 1;
            let saved = self.saved.get(index);
            let running = self.running.get(index);
            if saved != running {
                dbg!(saved);
                dbg!(running);
            }
        }
    }
}

lazy_static! {
    static ref GAME_RECORDER: Mutex<GameRecorder<'static>> = Mutex::new(GameRecorder::default());
}

pub fn record_start() {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.start_record();
}

pub fn record_stop() {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.save_record();
}

pub fn record_replay() {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.start_replay();
}

pub fn record_event(event: GameEvent<'static>) {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.record_event(event);
}
