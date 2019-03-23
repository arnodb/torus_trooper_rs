use piston::input::RenderArgs;

use crate::tt::actor::bullet::BulletPool;
use crate::tt::letter::Direction;
use crate::tt::manager::stage::StageManager;
use crate::tt::ship::Ship;
use crate::tt::ActionParams;

const DEFAULT_EXTEND_SCORE: u32 = 100000;
const MAX_EXTEND_SCORE: u32 = 500000;
const DEFAULT_TIME: i32 = 120000;
const MAX_TIME: i32 = 120000;

const SHIP_DESTROYED_PENALTY_TIME: i32 = -15000;
const SHIP_DESTROYED_PENALTY_TIME_MSG: &str = "-15 SEC.";

const EXTEND_TIME: i32 = 15000;
const EXTEND_TIME_MSG: &str = "+15 SEC.";

const NEXT_ZONE_ADDITION_TIME: i32 = 30000;
const NEXT_ZONE_ADDITION_TIME_MSG: &str = "+30 SEC.";

const NEXT_LEVEL_ADDITION_TIME: i32 = 45000;
const NEXT_LEVEL_ADDITION_TIME_MSG: &str = "+45 SEC.";

const BEEP_START_TIME: i32 = 15000;

pub struct SharedState<'a> {
    score: u32,
    next_extend: u32,
    time: i32,
    next_beep_time: i32,
    time_changed_msg: &'a str,
    time_changed_show_cnt: i32,
}

impl<'a> SharedState<'a> {
    pub fn new() -> Self {
        SharedState {
            score: 0,
            next_extend: 0,
            time: 0,
            next_beep_time: 0,
            time_changed_msg: "",
            time_changed_show_cnt: -1,
        }
    }

    pub fn init_game_state(&mut self, stage_manager: &StageManager, bullets: &mut BulletPool) {
        self.score = 0;
        self.next_extend = 0;
        self.set_next_extend(stage_manager.level());
        self.time_changed_show_cnt = -1;
        self.goto_next_zone(true, stage_manager, bullets);
    }

    pub fn goto_next_zone(
        &mut self,
        is_first: bool,
        stage_manager: &StageManager,
        bullets: &mut BulletPool,
    ) {
        bullets.clear_visible();
        if is_first {
            self.time = DEFAULT_TIME;
            self.next_beep_time = BEEP_START_TIME;
        } else {
            if stage_manager.medium_boss_zone() {
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

    pub fn add_score(&mut self, score: u32, game_over: bool, level: f32) {
        if !game_over {
            self.score += score;
            while self.score > self.next_extend {
                self.set_next_extend(level);
                self.extend_ship();
            }
        }
    }

    fn extend_ship(&mut self) {
        self.change_time(EXTEND_TIME, EXTEND_TIME_MSG);
        // TODO SoundManager.playSe("extend.wav");
    }

    fn set_next_extend(&mut self, level: f32) {
        self.next_extend += u32::min(
            ((level * 0.5) as u32 + 10) * DEFAULT_EXTEND_SCORE / 10,
            MAX_EXTEND_SCORE,
        );
    }

    fn change_time(&mut self, ct: i32, msg: &'a str) {
        self.time = i32::min(self.time + ct, MAX_TIME);
        self.next_beep_time = i32::min((self.time / 1000) * 1000, BEEP_START_TIME);
        self.time_changed_show_cnt = 240;
        self.time_changed_msg = msg;
    }

    pub fn decrement_time(&mut self, ship: &mut Ship) {
        self.time -= 17;
        if self.time_changed_show_cnt >= 0 {
            self.time_changed_show_cnt -= 1;
        }
        if ship.is_replay_mode() && self.time < 0 {
            ship.game_over();
        }
    }

    pub fn ship_destroyed(&mut self) {
        self.change_time(SHIP_DESTROYED_PENALTY_TIME, SHIP_DESTROYED_PENALTY_TIME_MSG);
    }

    pub fn check_time_overflow(&mut self) -> bool {
        if self.time < 0 {
            self.time = 0;
            true
        } else {
            false
        }
    }

    pub fn draw_front(&self, params: &ActionParams, _render_args: &RenderArgs) {
        params.ship.draw_front(params);
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
        if params.ship.is_game_over() {
            letter.draw_string("GAME OVER", 140., 180., 20.);
        }
    }

    pub fn score(&self) -> u32 {
        self.score
    }
}
