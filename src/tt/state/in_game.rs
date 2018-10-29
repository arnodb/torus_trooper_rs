use piston::input::RenderArgs;

use crate::gl;

use crate::tt::errors::GameError;
use crate::tt::letter::Direction;
use crate::tt::manager::stage::StageManager;
use crate::tt::manager::MoveAction;
use crate::tt::pad::PadButtons;
use crate::tt::ship::{Ship, ShipMoveAction};
use crate::tt::{DrawParams, MoveParams, StartParams};

use super::State;

const DEFAULT_EXTEND_SCORE: u32 = 100000;
const MAX_EXTEND_SCORE: u32 = 500000;
const DEFAULT_TIME: i32 = 120000;
const MAX_TIME: i32 = 120000;

const EXTEND_TIME: i32 = 15000;
const EXTEND_TIME_MSG: &str = "+15 SEC.";

const NEXT_ZONE_ADDITION_TIME: i32 = 30000;
const NEXT_ZONE_ADDITION_TIME_MSG: &str = "+30 SEC.";

const NEXT_LEVEL_ADDITION_TIME: i32 = 45000;
const NEXT_LEVEL_ADDITION_TIME_MSG: &str = "+45 SEC.";

const BEEP_START_TIME: i32 = 15000;

pub struct InGameState<'a> {
    grade: u32,
    level: f32,

    score: u32,
    next_extend: u32,
    time: i32,
    next_beep_time: i32,
    time_changed_msg: &'a str,
    time_changed_show_cnt: i32,
    game_over_cnt: u32,
    btn_pressed: bool,
    pause_cnt: u32,
    pause_pressed: bool,
}

impl<'a> InGameState<'a> {
    pub fn new() -> Result<Self, GameError> {
        Ok(InGameState {
            grade: 0,
            level: 1.,

            score: 0,
            next_extend: 0,
            time: 0,
            next_beep_time: 0,
            time_changed_msg: "",
            time_changed_show_cnt: -1,
            game_over_cnt: 0,
            btn_pressed: false,
            pause_cnt: 0,
            pause_pressed: false,
        })
    }

    fn init_game_state(&mut self, stage_manager: &StageManager) {
        self.score = 0;
        self.next_extend = 0;
        self.set_next_extend(stage_manager.level());
        self.time_changed_show_cnt = -1;
        self.goto_next_zone(true, stage_manager);
    }

    fn goto_next_zone(&mut self, is_first: bool, stage_manager: &StageManager) {
        //TODO clearVisibleBullets();
        if is_first {
            self.time = DEFAULT_TIME;
            self.next_beep_time = BEEP_START_TIME;
        } else {
            if stage_manager.middle_boss_zone() {
                self.change_time(NEXT_ZONE_ADDITION_TIME, NEXT_ZONE_ADDITION_TIME_MSG);
            } else {
                self.change_time(NEXT_LEVEL_ADDITION_TIME, NEXT_LEVEL_ADDITION_TIME_MSG);
                /* TODO sound
                startBgmCnt = 90;
                SoundManager.fadeBgm();
                */
            }
        }
    }

    fn decrement_time(&mut self, ship: &mut Ship) {
        self.time -= 17;
        if self.time_changed_show_cnt >= 0 {
            self.time_changed_show_cnt -= 1;
        }
        if ship.is_replay_mode() && self.time < 0 {
            ship.game_over();
        }
    }

    fn set_next_extend(&mut self, level: f32) {
        self.next_extend = u32::max(
            ((level * 0.5) as u32 + 10) * DEFAULT_EXTEND_SCORE / 10,
            MAX_EXTEND_SCORE,
        );
    }

    fn extend_ship(&mut self) {
        self.change_time(EXTEND_TIME, EXTEND_TIME_MSG);
        // TODO SoundManager.playSe("extend.wav");
    }

    fn change_time(&mut self, ct: i32, msg: &'a str) {
        self.time = i32::max(self.time + ct, MAX_TIME);
        /* TODO
        nextBeepTime = (time / 1000) * 1000;
        if (nextBeepTime > BEEP_START_TIME)
        nextBeepTime = BEEP_START_TIME;
        */
        self.time_changed_show_cnt = 240;
        self.time_changed_msg = msg;
    }
}

impl<'a> State for InGameState<'a> {
    fn start(&mut self, params: &mut StartParams) {
        self.grade = params.pref_manager.selected_grade();
        self.level = params.pref_manager.selected_level() as f32;
        /* TODO
        shots.clear();
        bullets.clear();
        enemies.clear();
        particles.clear();
        floatLetters.clear();
        RecordablePad rp = cast(RecordablePad) pad;
        rp.startRecord();
        _replayData = new ReplayData;
        _replayData.padRecord = rp.padRecord;
        _replayData.level = _level;
        _replayData.grade = _grade;
        _replayData.seed = _seed;
        Barrage.setRandSeed(_seed);
        Bullet.setRandSeed(_seed);
        Enemy.setRandSeed(_seed);
        FloatLetter.setRandSeed(_seed);
        Particle.setRandSeed(_seed);
        Shot.setRandSeed(_seed);
        SoundManager.setRandSeed(_seed);
        */
        let ship = &mut params.ship;
        ship.start(false, self.grade, params.seed, params.camera);
        params
            .stage_manager
            .start(self.level, self.grade, params.seed, params.tunnel);
        self.init_game_state(params.stage_manager);
        /* TODO sound
        SoundManager.playBgm();
        startBgmCnt = -1;
        */
        ship.set_screen_shake(0, 0.);
        self.game_over_cnt = 0;
        self.pause_cnt = 0;
        params.tunnel.set_ship_pos(0., 0.);
        params.tunnel.set_slices();
        // TODO sound SoundManager.enableSe();
    }

    fn mov(&mut self, params: &mut MoveParams) -> MoveAction {
        let pad = params.pad;
        let ship = &mut params.ship;
        if pad.pause_pressed() {
            if !self.pause_pressed {
                if self.pause_cnt <= 0 && !ship.is_game_over() {
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
        /* TODO sound
        if (startBgmCnt > 0) {
            startBgmCnt--;
            if (startBgmCnt <= 0)
            SoundManager.nextBgm();
        }
        */
        let ship_action = ship.mov(pad, params.camera, params.tunnel);
        if let ShipMoveAction::AddScore(sc) = ship_action {
            if !ship.is_game_over() {
                self.score += sc;
                while self.score > self.next_extend {
                    self.set_next_extend(params.stage_manager.level());
                    self.extend_ship();
                }
            }
        }
        params.stage_manager.mov();
        /* TODO
        enemies.move();
        shots.move();
        bullets.move();
        particles.move();
        floatLetters.move();
        */
        self.decrement_time(ship);
        let mut action = MoveAction::None;
        if self.time < 0 {
            self.time = 0;
            if !ship.is_game_over() {
                ship.game_over();
                self.btn_pressed = true;
                /* TODO sound
                SoundManager.fadeBgm();
                SoundManager.disableSe();
                */
                params
                    .pref_manager
                    .record_result(params.stage_manager.level() as u32, self.score);
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
        } /* TODO sound else if time <= nextBeepTime) {
              SoundManager.playSe("timeup_beep.wav");
              nextBeepTime -= 1000;
          }*/
        action
    }

    fn draw(&self, params: &mut DrawParams, _render_args: &RenderArgs) {
        unsafe {
            gl::Enable(gl::GL_CULL_FACE);
        }
        params
            .tunnel
            .draw(&params.stage_manager.slice_draw_state(), params.screen);
        unsafe {
            gl::Disable(gl::GL_CULL_FACE);
        }
        /*TODO
        particles.draw();
        enemies.draw();
        */
        params.ship.draw();
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
        }
        // TODO floatLetters.draw();
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
            gl::Disable(gl::GL_BLEND);
        }
        // TODO bullets.draw();
        unsafe {
            gl::Enable(gl::GL_BLEND);
        }
        // TODO shots.draw();
    }

    fn draw_front(&self, params: &DrawParams, _render_args: &RenderArgs) {
        let ship = &params.ship;
        ship.draw_front(params);
        let letter = params.letter;
        letter.draw_num(self.score as usize, 610., 0., 15.);
        letter.draw_string("/", 510., 40., 7.);
        letter.draw_num((self.next_extend - self.score) as usize, 615., 40., 7.);
        if self.time > BEEP_START_TIME {
            letter.draw_time(self.time as isize, 220., 24., 15.);
        } else {
            letter.draw_time_ex(self.time as isize, 220., 24., 15., 1);
        }
        if self.time_changed_show_cnt >= 0 && (self.time_changed_show_cnt % 64) > 32 {
            letter.draw_string_ex1(self.time_changed_msg, 250., 24., 7., Direction::ToRight, 1);
        }
        letter.draw_string_ex1("LEVEL", 20., 410., 8., Direction::ToRight, 1);
        letter.draw_num(params.stage_manager.level() as usize, 135., 410., 8.);
        if ship.is_game_over() {
            letter.draw_string("GAME OVER", 140., 180., 20.);
        }
        if self.pause_cnt > 0 && (self.pause_cnt % 64) < 32 {
            letter.draw_string("PAUSE", 240., 185., 17.);
        }
    }
}
