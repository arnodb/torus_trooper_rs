use piston::input::RenderArgs;

use crate::gl;

use crate::tt::errors::GameError;
use crate::tt::manager::title::TitleManager;
use crate::tt::manager::{Manager, MoveAction};
use crate::tt::screen::Screen;
use crate::tt::state::ReplayData;
use crate::tt::{GeneralParams, MoreParams};

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

    pub fn start(
        &mut self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
    ) -> Result<(), GameError> {
        params.sound_manager.halt_bgm();
        params.sound_manager.disable_se();
        self.manager.start(params, more_params)?;
        self.clear_all(more_params);
        self.start_replay(params, more_params);
        Ok(())
    }

    fn clear_all(&mut self, more_params: &mut MoreParams) {
        more_params.shots.clear();
        more_params.bullets.clear();
        more_params.enemies.clear_shallow();
        more_params.particles.clear();
        more_params.float_letters.clear();
    }

    pub fn set_replay_data(&mut self, replay_data: ReplayData) {
        self.replay_data = Some(replay_data);
    }

    fn start_replay(&mut self, params: &mut GeneralParams, more_params: &mut MoreParams) {
        if let Some(replay_data) = &self.replay_data {
            record_replay!(params.next_recorder_id);
            record_event_start!();
            params.pad.start_replay(replay_data.pad_record.clone());
            more_params.bullets.set_seed(replay_data.seed);
            more_params.enemies.set_seed(replay_data.seed);
            more_params.float_letters.set_seed(replay_data.seed);
            more_params.particles.set_seed(replay_data.seed);
            more_params.shots.set_seed(replay_data.seed);
            params.sound_manager.set_rand_seed(replay_data.seed);
            more_params.ship.start(
                true,
                replay_data.grade,
                replay_data.seed,
                params.camera,
                more_params.shots,
            );
            params.stage_manager.start(
                replay_data.level,
                replay_data.grade,
                replay_data.seed,
                params.screen,
                params.tunnel,
                params.barrage_manager,
                more_params,
            );
            params.shared_state.init_game_state(
                params.stage_manager,
                params.sound_manager,
                more_params.bullets,
            );
            more_params.ship.set_screen_shake(0, 0.);
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
    fn mov(&mut self, params: &mut GeneralParams, more_params: &mut MoreParams) -> MoveAction {
        if more_params.ship.is_game_over() {
            self.game_over_cnt += 1;
            if self.game_over_cnt > 120 {
                record_compare_replay!();
                self.clear_all(more_params);
                self.start_replay(params, more_params);
                return MoveAction::None;
            }
        }
        if self.replay_data.is_some() {
            more_params.ship.mov(
                params,
                more_params.shots,
                more_params.bullets,
                more_params.particles,
            );
            params.stage_manager.mov(
                params.screen,
                params.tunnel,
                params.barrage_manager,
                more_params,
            );
            if more_params.enemies.mov(
                params.tunnel,
                more_params.ship,
                more_params.bullets,
                more_params.particles,
            ) {
                params.shared_state.goto_next_zone(
                    false,
                    params.stage_manager,
                    params.sound_manager,
                    more_params.bullets,
                );
            }
            more_params.shots.mov(
                params,
                more_params.ship,
                more_params.bullets,
                more_params.enemies,
                more_params.particles,
                more_params.float_letters,
            );
            more_params.bullets.mov(
                params,
                more_params.ship,
                more_params.shots,
                more_params.particles,
            );
            more_params
                .particles
                .mov(more_params.ship.speed(), params.tunnel);
            more_params.float_letters.mov();
            more_params.enemies.mov_passed(
                params.tunnel,
                more_params.ship,
                more_params.bullets,
                more_params.particles,
            );
            params.shared_state.decrement_time(more_params.ship);
            self.manager.mov(true, params, more_params)
        } else {
            self.manager.mov(false, params, more_params)
        }
    }

    fn draw(
        &self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
        render_args: &RenderArgs,
    ) {
        if self.replay_data.is_some() {
            let rcr = f32::min(self.manager.replay_change_ratio() * 2.4, 1.);
            unsafe {
                let screen = &params.screen;
                let (p_width, p_height) = screen.physical_size();
                gl::Viewport(
                    0,
                    0,
                    (p_width / 4. * (3. + f64::from(rcr))) as i32,
                    p_height as i32,
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
            more_params.particles.draw(params.screen);
            more_params.enemies.draw(params.tunnel, more_params.bullets);
            more_params
                .enemies
                .draw_passed(params.tunnel, more_params.bullets);
            more_params.ship.draw();
            unsafe {
                gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
            }
            more_params.float_letters.draw(params);
            unsafe {
                gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
                gl::Disable(gl::GL_BLEND);
            }
            more_params.bullets.draw(params.tunnel);
            unsafe {
                gl::Enable(gl::GL_BLEND);
            }
            more_params.shots.draw(params.tunnel);
        }
        unsafe {
            let screen = &params.screen;
            let (p_width, p_height) = screen.physical_size();
            gl::Viewport(0, 0, p_width as i32, p_height as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            let ratio_threshold = 480. / 640.;
            let screen_ratio = p_height / p_width;
            if screen_ratio >= ratio_threshold {
                gl::Frustum(
                    -f64::from(screen.near_plane()),
                    f64::from(screen.near_plane()),
                    -f64::from(screen.near_plane()) * screen_ratio,
                    f64::from(screen.near_plane()) * screen_ratio,
                    0.1,
                    f64::from(screen.far_plane()),
                );
            } else {
                // This allows to see at least what can be seen horizontally and vertically
                // with the default ratio -- arnodb
                gl::Frustum(
                    -f64::from(screen.near_plane()) * ratio_threshold / screen_ratio,
                    f64::from(screen.near_plane()) * ratio_threshold / screen_ratio,
                    -f64::from(screen.near_plane()) * ratio_threshold,
                    f64::from(screen.near_plane()) * ratio_threshold,
                    0.1,
                    f64::from(screen.far_plane()),
                );
            }
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
        self.manager.draw(params, more_params, render_args)
    }

    fn draw_luminous(
        &self,
        _params: &mut GeneralParams,
        _more_params: &mut MoreParams,
        _render_args: &RenderArgs,
    ) {
    }

    fn draw_front(
        &self,
        params: &GeneralParams,
        _more_params: &MoreParams,
        render_args: &RenderArgs,
    ) {
        self.manager.draw_front(params, render_args);
    }
}
