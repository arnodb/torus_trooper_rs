pub mod stage;
pub mod title;

use piston::input::*;

use crate::tt::errors::GameError;
use crate::tt::prefs::{load_prefs_file, save_prefs_file, PrefManager};
use crate::tt::screen::Screen;
use crate::tt::state::in_game::InGameState;
use crate::tt::state::title::TitleState;
use crate::tt::state::{ReplayData, State};
use crate::tt::{GeneralParams, MoreParams};

use crate::gl;

pub trait Manager {
    fn start(
        &mut self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
    ) -> Result<(), GameError>;
    fn draw(
        &self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
        render_args: &RenderArgs,
    );
    fn draw_front(&self, params: &GeneralParams, render_args: &RenderArgs);
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GameState {
    Title,
    InGame,
}

pub struct GameManager {
    title_state: TitleState,
    in_game_state: InGameState,
    state: GameState,
    esc_pressed: bool,
}

#[derive(Debug)]
pub enum MoveAction {
    None,
    StartTitle(bool),
    StartInGame,
    BreakLoop,
}

impl GameManager {
    pub fn new(screen: &Screen) -> Result<Self, GameError> {
        let title_state = TitleState::new(&screen)?;
        let in_game_state = InGameState::new();
        Ok(GameManager {
            title_state,
            in_game_state,
            state: GameState::Title,
            esc_pressed: false,
        })
    }

    pub fn quit_last(&self, pref_manager: &PrefManager) -> Result<(), GameError> {
        pref_manager.save()?;
        Ok(())
    }

    pub fn start_title(
        &mut self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
        load_last_state: bool,
        from_game_over: bool,
    ) -> Result<(), GameError> {
        let replay_data = if load_last_state {
            load_prefs_file::<ReplayData, _>("last_replay")
        } else {
            self.in_game_state.replay_data(params)
        };
        if from_game_over {
            save_prefs_file(&replay_data, "last_replay")?;
        }
        self.title_state.set_replay_data(replay_data);
        self.state = GameState::Title;
        self.start_state(0, params, more_params)?;
        Ok(())
    }

    pub fn start_in_game(
        &mut self,
        seed: u64,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
    ) -> Result<(), GameError> {
        self.state = GameState::InGame;
        self.start_state(seed, params, more_params)?;
        Ok(())
    }

    fn start_state(
        &mut self,
        seed: u64,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
    ) -> Result<(), GameError> {
        match self.state {
            GameState::Title => {
                self.title_state.start(params, more_params)?;
            }
            GameState::InGame => {
                let grade = params.pref_manager.selected_grade();
                let level = params.pref_manager.selected_level();
                self.in_game_state
                    .start(grade, level, seed, params, more_params)
            }
        }
        Ok(())
    }

    pub fn mov(&mut self, params: &mut GeneralParams, more_params: &mut MoreParams) -> MoveAction {
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
                GameState::Title => self.title_state.mov(params, more_params),
                GameState::InGame => self.in_game_state.mov(params, more_params),
            };
        }
        action
    }
}

impl Manager for GameManager {
    fn start(
        &mut self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
    ) -> Result<(), GameError> {
        self.start_title(params, more_params, true, false)?;
        Ok(())
    }

    fn draw(
        &self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
        render_args: &RenderArgs,
    ) {
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
        more_params
            .ship
            .set_eye_pos(params.screen, params.camera, params.tunnel);
        match self.state {
            GameState::Title => self.title_state.draw(params, more_params, render_args),
            GameState::InGame => self.in_game_state.draw(params, more_params, render_args),
        }
        unsafe {
            gl::PopMatrix();
        }
        Screen::view_ortho_fixed();
        match self.state {
            GameState::Title => {
                self.title_state
                    .draw_front(params, more_params, render_args);
                if more_params.ship.is_draw_front_mode()
                    && self.title_state.replay_change_ratio() >= 1.
                {
                    params
                        .shared_state
                        .draw_front(params, more_params, render_args);
                }
            }
            GameState::InGame => self
                .in_game_state
                .draw_front(params, more_params, render_args),
        }
        Screen::view_perspective();
    }

    fn draw_front(&self, _params: &GeneralParams, _render_args: &RenderArgs) {}
}
