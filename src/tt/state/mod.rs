pub mod in_game;
pub mod shared;
pub mod title;

use piston::input::RenderArgs;
use rle_vec::RleVec;

use crate::tt::manager::MoveAction;
use crate::tt::pad::PadState;
use crate::tt::ActionParams;

pub trait State {
    fn mov(&mut self, params: &mut ActionParams) -> MoveAction;
    fn draw(&self, params: &mut ActionParams, render_args: &RenderArgs);
    fn draw_front(&self, params: &ActionParams, render_args: &RenderArgs);
}

#[derive(Debug, Clone)]
pub struct ReplayData {
    grade: u32,
    level: f32,
    seed: u64,
    pad_record: RleVec<PadState>,
}

impl ReplayData {
    pub fn new() -> Self {
        ReplayData {
            grade: 0,
            level: 1.,
            seed: 0,
            pad_record: RleVec::new(),
        }
    }

    pub fn grade(mut self, grade: u32) -> Self {
        self.grade = grade;
        self
    }

    pub fn level(mut self, level: f32) -> Self {
        self.level = level;
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn pad_record(mut self, pad_record: RleVec<PadState>) -> Self {
        self.pad_record = pad_record;
        self
    }
}
