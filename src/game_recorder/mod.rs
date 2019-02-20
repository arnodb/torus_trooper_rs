use std::borrow::Cow;
use std::sync::Mutex;

#[derive(Default)]
pub struct GameRecorder<'a> {
    recording: bool,
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
        self.recording = true;
        self.saved.clear();
        self.running.clear();
        self.next_id = next_id;
    }
    pub fn save_record(&mut self) {
        self.recording = false;
        std::mem::swap(&mut self.saved, &mut self.running);
    }
    pub fn start_replay(&mut self, next_id: usize) {
        self.recording = true;
        self.running.clear();
        self.next_id = next_id;
    }
    pub fn compare_replay(&mut self) {
        let until_end = || {
            let mut reached_end = false;
            move |e: &&GameEvent| {
                if reached_end {
                    false
                } else {
                    if let GameEvent::End { .. } = e {
                        reached_end = true;
                    }
                    true
                }
            }
        };
        let left = &self
            .saved
            .iter()
            .take_while(until_end())
            .map(|event| serde_json::to_string(event).unwrap())
            .collect::<Vec<String>>();
        let right = &self
            .running
            .iter()
            .take_while(until_end())
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
        self.recording = false;
        self.running.clear();
    }
    pub fn record_event(&mut self, event: GameEvent<'a>) {
        if self.recording {
            self.running.push(event);
        }
    }
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

lazy_static! {
    pub static ref GAME_RECORDER: Mutex<GameRecorder<'static>> =
        Mutex::new(GameRecorder::default());
}
