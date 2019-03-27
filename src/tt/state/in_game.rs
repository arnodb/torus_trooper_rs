use piston::input::RenderArgs;

use crate::gl;

use crate::tt::manager::MoveAction;
use crate::tt::pad::PadButtons;
use crate::tt::state::ReplayData;
use crate::tt::ActionParams;

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

    pub fn start(&mut self, grade: u32, level: u32, seed: u64, params: &mut ActionParams) {
        record_start!(params.next_recorder_id);
        record_event_start!();
        params.shots.clear();
        params.bullets.clear();
        params.enemies.clear_shallow();
        params.particles.clear();
        params.float_letters.clear();
        params.pad.start_record();
        self.replay_data = ReplayData::new()
            .grade(grade)
            .level(level as f32)
            .seed(seed);
        params.bullets.set_seed(seed);
        params.enemies.set_seed(seed);
        params.float_letters.set_seed(seed);
        params.particles.set_seed(seed);
        params.shots.set_seed(seed);
        params.sound_manager.set_rand_seed(seed);
        params.ship.start(false, grade, seed, params.camera);
        params.stage_manager.start(
            level as f32,
            grade,
            seed,
            params.screen,
            params.tunnel,
            params.ship,
            params.bullets,
            params.enemies,
            params.barrage_manager,
        );
        params.shared_state.init_game_state(
            params.stage_manager,
            params.sound_manager,
            params.bullets,
        );
        params.sound_manager.play_bgm();
        params.shared_state.start_bgm_clear();
        params.ship.set_screen_shake(0, 0.);
        self.game_over_cnt = 0;
        self.pause_cnt = 0;
        params.tunnel.set_ship_pos(0., 0.);
        params.tunnel.set_slices();
        params.sound_manager.enable_se();
    }

    pub fn replay_data(&mut self, params: &mut ActionParams) -> ReplayData {
        self.replay_data.clone().pad_record(params.pad.get_record())
    }
}

impl State for InGameState {
    fn mov(&mut self, params: &mut ActionParams) -> MoveAction {
        let pad = &mut params.pad;
        if pad.pause_pressed() {
            if !self.pause_pressed {
                if self.pause_cnt <= 0 && !params.ship.is_game_over() {
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
        params.ship.mov(
            *pad,
            params.camera,
            params.tunnel,
            params.shared_state,
            params.stage_manager,
            params.sound_manager,
            params.shots,
            params.bullets,
            params.particles,
        );
        params.stage_manager.mov(
            params.screen,
            params.tunnel,
            params.ship,
            params.bullets,
            params.enemies,
            params.barrage_manager,
        );
        if params
            .enemies
            .mov(params.tunnel, params.ship, params.bullets, params.particles)
        {
            params.shared_state.goto_next_zone(
                false,
                params.stage_manager,
                params.sound_manager,
                params.bullets,
            );
        }
        params.shots.mov(
            params.tunnel,
            params.shared_state,
            params.stage_manager,
            params.sound_manager,
            params.ship,
            params.bullets,
            params.enemies,
            params.particles,
            &mut params.float_letters,
        );
        params.bullets.mov(
            params.tunnel,
            params.shared_state,
            params.sound_manager,
            params.ship,
            params.particles,
        );
        params.particles.mov(params.ship.speed(), params.tunnel);
        params.float_letters.mov();
        params.shared_state.decrement_time(params.ship);
        let mut action = MoveAction::None;
        if params.shared_state.check_time_overflow() {
            if !params.ship.is_game_over() {
                params.ship.game_over();
                self.btn_pressed = true;
                params.sound_manager.fade_bgm();
                params.sound_manager.disable_se();
                params.pref_manager.record_result(
                    params.stage_manager.level() as u32,
                    params.shared_state.score(),
                );
            }
            self.game_over_cnt += 1;
            let btn = pad.get_buttons();
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

    fn draw(&self, params: &mut ActionParams, _render_args: &RenderArgs) {
        unsafe {
            gl::Enable(gl::GL_CULL_FACE);
        }
        params
            .tunnel
            .draw(&params.stage_manager.slice_draw_state(), params.screen);
        unsafe {
            gl::Disable(gl::GL_CULL_FACE);
        }
        params.particles.draw(params.screen);
        params.enemies.draw(params.tunnel, params.bullets);
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

    fn draw_front(&self, params: &ActionParams, render_args: &RenderArgs) {
        params.shared_state.draw_front(params, render_args);
        if self.pause_cnt > 0 && (self.pause_cnt % 64) < 32 {
            params.letter.draw_string("PAUSE", 240., 185., 17.);
        }
    }
}
