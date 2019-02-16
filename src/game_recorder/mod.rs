use std::borrow::Cow;
use std::sync::Mutex;

#[derive(Default)]
pub struct GameRecorder<'a> {
    replay: bool,
    saved: Vec<GameEvent<'a>>,
    running: Vec<GameEvent<'a>>,
    next_id: usize,
}

#[derive(PartialEq, Serialize, Debug)]
pub enum GameEvent<'a> {
    Start,
    End {
        from_game_over: bool,
    },
    NewRand {
        rng_id: usize,
        seed: u64,
    },
    SetRandSeed {
        rng_id: usize,
        seed: u64,
    },
    RandF32 {
        rng_id: usize,
        high: f32,
        value: f32,
    },
    RandSignedF32 {
        rng_id: usize,
        high: f32,
        value: f32,
    },
    RandUsize {
        rng_id: usize,
        high: usize,
        value: usize,
    },
    Custom(Cow<'a, str>),
}

impl<'a> GameRecorder<'a> {
    pub fn start_record(&mut self, next_id: usize) {
        self.replay = false;
        self.saved.clear();
        self.running.clear();
        self.next_id = next_id;
    }
    pub fn save_record(&mut self) {
        std::mem::swap(&mut self.saved, &mut self.running);
    }
    pub fn start_replay(&mut self, next_id: usize) {
        self.replay = true;
        self.running.clear();
        self.next_id = next_id;
    }
    pub fn compare_replay(&mut self) {
        let left = &self
            .saved
            .iter()
            .map(|event| serde_json::to_string(event).unwrap())
            .collect::<Vec<String>>();
        let right = &self
            .running
            .iter()
            .map(|event| serde_json::to_string(event).unwrap())
            .collect::<Vec<String>>();
        let diff = diff::slice(left, right);
        println!("COMPARE !!!!!!!!!!!!!!!!!!!!!!!!");
        for res in diff {
            match res {
                diff::Result::Both(l, _r) => {
                    println!(" {}", l);
                }
                diff::Result::Left(l) => {
                    println!("-{}", l);
                }
                diff::Result::Right(r) => {
                    println!("+{}", r);
                }
            }
        }
        println!("END COMPARE !!!!!!!!!!!!!!!!!!!!!!!!");
        self.running.clear();
    }
    pub fn record_event(&mut self, event: GameEvent<'a>) {
        self.running.push(event);
    }
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

lazy_static! {
    static ref GAME_RECORDER: Mutex<GameRecorder<'static>> = Mutex::new(GameRecorder::default());
}

pub fn record_start(next_id: usize) {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.start_record(next_id);
}

pub fn record_stop() {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.save_record();
}

pub fn record_replay(next_id: usize) {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.start_replay(next_id);
}

pub fn record_compare_replay() {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.compare_replay();
}

pub fn record_event(event: GameEvent<'static>) {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.record_event(event);
}

pub fn record_next_id() -> usize {
    let mut guard = GAME_RECORDER.lock().unwrap();
    guard.next_id()
}
