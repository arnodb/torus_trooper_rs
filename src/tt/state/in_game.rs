use piston::input::RenderArgs;

use crate::gl;

use crate::tt::errors::GameError;
use crate::tt::letter::Direction;
use crate::tt::manager::MoveAction;
use crate::tt::pad::PadButtons;
use crate::tt::ship::Ship;
use crate::tt::{DrawParams, MoveParams, StartParams};

use super::State;

const DEFAULT_TIME: i32 = 120000;
const BEEP_START_TIME: i32 = 15000;

pub struct InGameState {
    grade: u32,
    seed: u64,

    score: u32,
    next_extend: u32,
    time: i32,
    next_beep_time: i32,
    time_changed_show_cnt: i32,
    game_over_cnt: u32,
    btn_pressed: bool,
    pause_cnt: u32,
    pause_pressed: bool,
}

impl InGameState {
    pub fn new() -> Result<Self, GameError> {
        Ok(InGameState {
            grade: 0,
            seed: 0,

            score: 0,
            next_extend: 0,
            time: 0,
            next_beep_time: 0,
            time_changed_show_cnt: -1,
            game_over_cnt: 0,
            btn_pressed: false,
            pause_cnt: 0,
            pause_pressed: false,
        })
    }

    fn init_game_state(&mut self) {
        self.score = 0;
        self.next_extend = 0;
        // TODO stage manager setNextExtend();
        self.time_changed_show_cnt = -1;
        self.goto_next_zone(true);
    }

    fn goto_next_zone(&mut self, is_first: bool) {
        //TODO clearVisibleBullets();
        if is_first {
            self.time = DEFAULT_TIME;
            self.next_beep_time = BEEP_START_TIME;
        } else {
            /*TODO if (stageManager.middleBossZone) {
                changeTime(NEXT_ZONE_ADDITION_TIME, NEXT_ZONE_ADDITION_TIME_MSG);
            } else {
                changeTime(NEXT_LEVEL_ADDITION_TIME, NEXT_LEVEL_ADDITION_TIME_MSG);
                startBgmCnt = 90;
                SoundManager.fadeBgm();
            }*/
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
}

impl State for InGameState {
    fn start(&mut self, params: &mut StartParams) {
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
        ship.start(false, self.grade, self.seed, params.camera);
        // TODOstageManager.start(_level, _grade, _seed);
        self.init_game_state();
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
        ship.mov(pad, params.camera, params.tunnel);
        /* TODO
        stageManager.move();
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
                prefManager.prefData.recordResult(cast(int) stageManager.level, score);
                */
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
        params.tunnel.draw(params.screen);
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
            // TODO letter.draw_string_ex1(self.time_changed_msg, 250., 24., 7., Direction::ToRight, 1);
        }
        letter.draw_string_ex1("LEVEL", 20., 410., 8., Direction::ToRight, 1);
        // TODO letter.draw_num(cast(int) stageManager.level, 135, 410, 8);
        if ship.is_game_over() {
            letter.draw_string("GAME OVER", 140., 180., 20.);
        }
        if self.pause_cnt > 0 && (self.pause_cnt % 64) < 32 {
            letter.draw_string("PAUSE", 240., 185., 17.);
        }
    }
}
