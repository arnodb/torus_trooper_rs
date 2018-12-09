pub mod in_game;
pub mod title;

use piston::input::RenderArgs;

use crate::tt::manager::MoveAction;
use crate::tt::ActionParams;

pub trait State {
    fn start(&mut self, seed: u64, params: &mut ActionParams);
    fn mov(&mut self, params: &mut ActionParams) -> MoveAction;
    fn draw(&self, params: &mut ActionParams, render_args: &RenderArgs);
    fn draw_front(&self, params: &ActionParams, render_args: &RenderArgs);
}
