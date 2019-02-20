use piston::input::RenderArgs;

use crate::gl;

use crate::tt::errors::GameError;
use crate::tt::manager::title::TitleManager;
use crate::tt::manager::{Manager, MoveAction};
use crate::tt::screen::Screen;
use crate::tt::state::in_game::ScoreAccumulator;
use crate::tt::state::ReplayData;
use crate::tt::ActionParams;

use super::State;

pub struct TitleState {
    manager: TitleManager,
    replay_data: Option<ReplayData>,
    game_over_cnt: u32,
}

impl TitleState {
    pub fn new(screen: &Screen) -> Result<Self, GameError> {
        Ok(TitleState {
            manager: TitleManager::new(screen)?,
            replay_data: None,
            game_over_cnt: 0,
        })
    }

    pub fn start(&mut self, params: &mut ActionParams) {
        // TODO SoundManager.haltBgm();
        // TODO SoundManager.disableSe();
        self.manager.start(params);
        self.clear_all(params);
        self.start_replay(params);
    }

    fn clear_all(&mut self, params: &mut ActionParams) {
        params.shots.clear();
        params.bullets.clear();
        params.enemies.clear_shallow();
        params.particles.clear();
        params.float_letters.clear();
        /*TODO
        passedEnemies.clear();
        */
    }

    pub fn set_replay_data(&mut self, replay_data: ReplayData) {
        self.replay_data = Some(replay_data);
    }

    fn start_replay(&mut self, params: &mut ActionParams) {
        if let Some(replay_data) = &self.replay_data {
            record_replay!(params.next_recorder_id);
            record_event_start!();
            params.pad.start_replay(replay_data.pad_record.clone());
            params.bullets.set_seed(replay_data.seed);
            params.particles.set_seed(replay_data.seed);
            params.enemies.set_seed(replay_data.seed);
            /* TODO REPLAY
            FloatLetter.setRandSeed(_seed);
            Shot.setRandSeed(_seed);
            SoundManager.setRandSeed(_seed);
            */
            params
                .ship
                .start(true, replay_data.grade, replay_data.seed, params.camera);
            params.stage_manager.start(
                replay_data.level,
                replay_data.grade,
                replay_data.seed,
                params.screen,
                params.tunnel,
                params.ship,
                params.bullets,
                params.enemies,
                params.barrage_manager,
            );
            params.ship.set_screen_shake(0, 0.);
            self.game_over_cnt = 0;
            params.tunnel.set_ship_pos(0., 0.);
            params.tunnel.set_slices();
            params.tunnel.set_slices_backward();
        }
    }

    pub fn replay_change_ratio(&self) -> f32 {
        self.manager.replay_change_ratio()
    }
}

impl State for TitleState {
    fn mov(&mut self, params: &mut ActionParams) -> MoveAction {
        if params.ship.is_game_over() {
            self.game_over_cnt += 1;
            if self.game_over_cnt > 120 {
                record_compare_replay!();
                self.clear_all(params);
                self.start_replay(params);
                return MoveAction::StartReplay;
            }
        }
        let action = if self.replay_data.is_some() {
            let mut score_accumulator = ScoreAccumulator { score: 0 };
            params.ship.mov(
                params.pad,
                params.camera,
                params.tunnel,
                params.shots,
                params.bullets,
                params.particles,
                &mut score_accumulator,
            );
            params.stage_manager.mov(
                params.screen,
                params.tunnel,
                params.ship,
                params.bullets,
                params.enemies,
                params.barrage_manager,
            );
            params
                .enemies
                .mov(params.tunnel, params.ship, params.bullets, params.particles);
            params.shots.mov(
                params.tunnel,
                params.ship,
                params.bullets,
                params.enemies,
                params.particles,
                params.float_letters,
                &mut score_accumulator,
            );
            params
                .bullets
                .mov(params.tunnel, params.ship, params.particles);
            params.particles.mov(params.ship.speed(), params.tunnel);
            params.float_letters.mov();
            // TODO REPLAY params.passedEnemies.mov();
            self.manager.mov(true, params)
        } else {
            self.manager.mov(false, params)
        };
        action
    }

    fn draw(&self, params: &mut ActionParams, render_args: &RenderArgs) {
        if self.replay_data.is_some() {
            let rcr = f32::max(self.manager.replay_change_ratio() * 2.4, 1.);
            unsafe {
                let screen = &params.screen;
                let p_size = screen.physical_size();
                gl::Viewport(
                    0,
                    0,
                    (p_size.0 as f32 / 4. * (3. + rcr)) as i32,
                    p_size.1 as i32,
                );
                gl::Enable(gl::GL_CULL_FACE);
            }
            params
                .tunnel
                .draw(params.stage_manager.slice_draw_state(), params.screen);
            params
                .tunnel
                .draw_backward(params.stage_manager.slice_draw_state(), params.screen);
            unsafe {
                gl::Disable(gl::GL_CULL_FACE);
            }
            params.particles.draw(params.screen);
            params.enemies.draw(params.tunnel, params.bullets);
            // TODO REPLAY params.passedEnemies.draw();
            params.ship.draw();
            unsafe {
                gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
            }
            params
                .float_letters
                .draw(params.screen, params.letter, params.tunnel);
            unsafe {
                gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
                gl::Disable(gl::GL_BLEND);
            }
            params.bullets.draw(params.tunnel);
            unsafe {
                gl::Enable(gl::GL_BLEND);
            }
            params.shots.draw(params.tunnel);
        }
        unsafe {
            let screen = &params.screen;
            let p_size = screen.physical_size();
            gl::Viewport(0, 0, p_size.0 as i32, p_size.1 as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            let ratio_threshold = 480. / 640.;
            let screen_ratio = p_size.1 as f32 / p_size.0 as f32;
            if screen_ratio >= ratio_threshold {
                gl::Frustum(
                    -screen.near_plane() as f64,
                    screen.near_plane() as f64,
                    (-screen.near_plane() * screen_ratio) as f64,
                    (screen.near_plane() * screen_ratio) as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
            } else {
                // This allows to see at least what can be seen horizontally and vertically
                // with the default ratio -- arnodb
                gl::Frustum(
                    (-screen.near_plane() * ratio_threshold / screen_ratio) as f64,
                    (screen.near_plane() * ratio_threshold / screen_ratio) as f64,
                    (-screen.near_plane() * ratio_threshold) as f64,
                    (screen.near_plane() * ratio_threshold) as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
            }
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
        self.manager.draw(params, render_args)
    }

    fn draw_front(&self, params: &ActionParams, render_args: &RenderArgs) {
        self.manager.draw_front(params, render_args);
    }
}
