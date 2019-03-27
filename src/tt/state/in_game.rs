use piston::input::RenderArgs;

use crate::gl;

use crate::tt::manager::MoveAction;
use crate::tt::pad::PadButtons;
use crate::tt::state::ReplayData;
use crate::tt::{GeneralParams, MoreParams};

use super::State;

pub struct InGameState {
    game_over_cnt: u32,
    btn_pressed: bool,
    pause_cnt: u32,
    pause_pressed: bool,
    replay_data: ReplayData,
}

impl InGameState {
    pub fn new() -> Self {
        InGameState {
            game_over_cnt: 0,
            btn_pressed: false,
            pause_cnt: 0,
            pause_pressed: false,
            replay_data: ReplayData::new(),
        }
    }

    pub fn start(
        &mut self,
        grade: u32,
        level: u32,
        seed: u64,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
    ) {
        record_start!(params.next_recorder_id);
        record_event_start!();
        more_params.shots.clear();
        more_params.bullets.clear();
        more_params.enemies.clear_shallow();
        more_params.particles.clear();
        more_params.float_letters.clear();
        params.pad.start_record();
        self.replay_data = ReplayData::new()
            .grade(grade)
            .level(level as f32)
            .seed(seed);
        more_params.bullets.set_seed(seed);
        more_params.enemies.set_seed(seed);
        more_params.float_letters.set_seed(seed);
        more_params.particles.set_seed(seed);
        more_params.shots.set_seed(seed);
        params.sound_manager.set_rand_seed(seed);
        more_params
            .ship
            .start(false, grade, seed, params.camera, more_params.shots);
        params.stage_manager.start(
            level as f32,
            grade,
            seed,
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
        params.sound_manager.play_bgm();
        params.shared_state.start_bgm_clear();
        more_params.ship.set_screen_shake(0, 0.);
        self.game_over_cnt = 0;
        self.pause_cnt = 0;
        params.tunnel.set_ship_pos(0., 0.);
        params.tunnel.set_slices();
        params.sound_manager.enable_se();
    }

    pub fn replay_data(&mut self, params: &mut GeneralParams) -> ReplayData {
        self.replay_data.clone().pad_record(params.pad.get_record())
    }
}

impl State for InGameState {
    fn mov(&mut self, params: &mut GeneralParams, more_params: &mut MoreParams) -> MoveAction {
        if params.pad.pause_pressed() {
            if !self.pause_pressed {
                if self.pause_cnt <= 0 && !more_params.ship.is_game_over() {
                    self.pause_cnt = 1;
                } else {
                    self.pause_cnt = 0;
                }
            }
            self.pause_pressed = true;
        } else {
            self.pause_pressed = false;
        }
        if self.pause_cnt > 0 {
            self.pause_cnt += 1;
            return MoveAction::None;
        }
        params.shared_state.start_bgm_tick(params.sound_manager);
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
        params.shared_state.decrement_time(more_params.ship);
        let mut action = MoveAction::None;
        if params.shared_state.check_time_overflow() {
            if !more_params.ship.is_game_over() {
                more_params.ship.game_over();
                self.btn_pressed = true;
                params.sound_manager.fade_bgm();
                params.sound_manager.disable_se();
                params.pref_manager.record_result(
                    params.stage_manager.level() as u32,
                    params.shared_state.score(),
                );
            }
            self.game_over_cnt += 1;
            let btn = params.pad.get_buttons();
            if btn & PadButtons::A != PadButtons::NONE {
                if self.game_over_cnt > 60 && !self.btn_pressed {
                    action = MoveAction::StartTitle(true);
                }
                self.btn_pressed = true;
            } else {
                self.btn_pressed = false;
            }
            if self.game_over_cnt > 1200 {
                action = MoveAction::StartTitle(false);
            }
        } else if params.shared_state.check_beep_time() {
            params.sound_manager.play_se("timeup_beep.wav");
        }
        action
    }

    fn draw(
        &self,
        params: &mut GeneralParams,
        more_params: &mut MoreParams,
        _render_args: &RenderArgs,
    ) {
        unsafe {
            gl::Enable(gl::GL_CULL_FACE);
        }
        params
            .tunnel
            .draw(&params.stage_manager.slice_draw_state(), params.screen);
        unsafe {
            gl::Disable(gl::GL_CULL_FACE);
        }
        more_params.particles.draw(params.screen);
        more_params.enemies.draw(params.tunnel, more_params.bullets);
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

    fn draw_front(
        &self,
        params: &GeneralParams,
        more_params: &MoreParams,
        render_args: &RenderArgs,
    ) {
        params
            .shared_state
            .draw_front(params, more_params, render_args);
        if self.pause_cnt > 0 && (self.pause_cnt % 64) < 32 {
            params.letter.draw_string("PAUSE", 240., 185., 17.);
        }
    }
}
