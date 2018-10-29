pub mod in_game;
pub mod title;

use piston::input::RenderArgs;

use crate::tt::manager::MoveAction;
use crate::tt::{DrawParams, MoveParams, StartParams};

pub trait State {
    fn start(&mut self, params: &mut StartParams);
    fn mov(&mut self, params: &mut MoveParams) -> MoveAction;
    fn draw(&self, params: &mut DrawParams, render_args: &RenderArgs);
    fn draw_front(&self, params: &DrawParams, render_args: &RenderArgs);
}
