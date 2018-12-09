pub mod stage;
pub mod title;

use piston::input::*;

use crate::tt::errors::GameError;
use crate::tt::prefs::PrefManager;
use crate::tt::screen::Screen;
use crate::tt::state::in_game::InGameState;
use crate::tt::state::title::TitleState;
use crate::tt::state::State;
use crate::tt::ActionParams;

use crate::gl;

pub trait Manager {
    fn start(&mut self, seed: u64, params: &mut ActionParams);
    fn draw(&self, params: &mut ActionParams, render_args: &RenderArgs);
    fn draw_front(&self, params: &ActionParams, render_args: &RenderArgs);
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GameState {
    Title,
    InGame,
}

pub struct GameManager<'a> {
    title_state: TitleState,
    in_game_state: InGameState<'a>,
    state: GameState,
    esc_pressed: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum MoveAction {
    None,
    StartTitle(bool),
    StartInGame,
    BreakLoop,
}

impl<'a> GameManager<'a> {
    pub fn new(screen: &Screen) -> Result<Self, GameError> {
        let title_state = TitleState::new(&screen)?;
        let in_game_state = InGameState::new()?;
        Ok(GameManager {
            title_state,
            in_game_state,
            state: GameState::Title,
            esc_pressed: false,
        })
    }

    pub fn quit_last(&self, pref_manager: &PrefManager) -> Result<(), GameError> {
        // TODO
        pref_manager.save()?;
        // TODO
        Ok(())
    }

    pub fn start_title(&mut self, seed: u64, params: &mut ActionParams, _from_game_over: bool) {
        //TODO if (fromGameover)
        //TODO saveLastReplay();
        // TODO titleState.setReplayData(inGameState.replayData);
        self.state = GameState::Title;
        self.start_state(seed, params);
    }

    pub fn start_in_game(&mut self, seed: u64, params: &mut ActionParams) {
        self.state = GameState::InGame;
        self.start_state(seed, params);
    }

    fn start_state(&mut self, seed: u64, params: &mut ActionParams) {
        match self.state {
            GameState::Title => self.title_state.start(seed, params),
            GameState::InGame => self.in_game_state.start(seed, params),
        }
    }

    pub fn mov(&mut self, params: &mut ActionParams) -> MoveAction {
        let mut action = MoveAction::None;
        if params.pad.esc_pressed() {
            if !self.esc_pressed {
                self.esc_pressed = true;
                match self.state {
                    GameState::InGame => action = MoveAction::StartTitle(false),
                    GameState::Title => return MoveAction::BreakLoop,
                }
            }
        } else {
            self.esc_pressed = false;
        }
        if let MoveAction::None = action {
            action = match self.state {
                GameState::Title => self.title_state.mov(params),
                GameState::InGame => self.in_game_state.mov(params),
            };
        }
        action
    }
}

impl<'a> Manager for GameManager<'a> {
    fn start(&mut self, seed: u64, params: &mut ActionParams) {
        // TODO loadLastReplay();
        self.start_title(seed, params, false);
    }

    fn draw(&self, params: &mut ActionParams, render_args: &RenderArgs) {
        /* TODO
        if (screen.startRenderToLuminousScreen()) {
            glPushMatrix();
            ship.setEyepos();
            state.drawLuminous();
            glPopMatrix();
            screen.endRenderToLuminousScreen();
        }
        */
        Screen::clear();
        unsafe {
            gl::PushMatrix();
        }
        params
            .ship
            .set_eye_pos(params.screen, params.camera, params.tunnel);
        match self.state {
            GameState::Title => self.title_state.draw(params, render_args),
            GameState::InGame => self.in_game_state.draw(params, render_args),
        }
        unsafe {
            gl::PopMatrix();
        }
        Screen::view_ortho_fixed();
        match self.state {
            GameState::Title => self.title_state.draw_front(params, render_args),
            GameState::InGame => self.in_game_state.draw_front(params, render_args),
        }
        Screen::view_perspective();
    }

    fn draw_front(&self, _params: &ActionParams, _render_args: &RenderArgs) {}
}
