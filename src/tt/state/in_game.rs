use piston::input::RenderArgs;

use crate::gl;

use crate::tt::actor::bullet::BulletPool;
use crate::tt::errors::GameError;
use crate::tt::letter::Direction;
use crate::tt::manager::stage::StageManager;
use crate::tt::manager::MoveAction;
use crate::tt::pad::PadButtons;
use crate::tt::ship::Ship;
use crate::tt::state::ReplayData;
use crate::tt::ActionParams;

use super::State;

#[cfg(feature = "game_recorder")]
use crate::game_recorder::{record_event, record_start, GameEvent};

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
    replay_data: ReplayData,
}

impl<'a> InGameState<'a> {
    pub fn new() -> Result<Self, GameError> {
        Ok(InGameState {
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
            replay_data: ReplayData::new(),
        })
    }

    pub fn start(&mut self, grade: u32, level: u32, seed: u64, params: &mut ActionParams) {
        #[cfg(feature = "game_recorder")]
        {
            record_start(params.next_recorder_id);
            record_event(GameEvent::Start);
        }
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
        /* TODO REPLAY
        Enemy.setRandSeed(_seed);
        FloatLetter.setRandSeed(_seed);
        */
        params.particles.set_seed(seed);
        /* TODO REPLAY
        Shot.setRandSeed(_seed);
        SoundManager.setRandSeed(_seed);
        */
        params.enemies.set_seed(seed);
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
        self.init_game_state(params.stage_manager, params.bullets);
        /* TODO sound
        SoundManager.playBgm();
        startBgmCnt = -1;
        */
        params.ship.set_screen_shake(0, 0.);
        self.game_over_cnt = 0;
        self.pause_cnt = 0;
        params.tunnel.set_ship_pos(0., 0.);
        params.tunnel.set_slices();
        // TODO sound SoundManager.enableSe();
    }

    pub fn init_game_state(&mut self, stage_manager: &StageManager, bullets: &mut BulletPool) {
        self.score = 0;
        self.next_extend = 0;
        self.set_next_extend(stage_manager.level());
        self.time_changed_show_cnt = -1;
        self.goto_next_zone(true, stage_manager, bullets);
    }

    fn goto_next_zone(
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

    pub fn decrement_time(&mut self, ship: &mut Ship) {
        self.time -= 17;
        if self.time_changed_show_cnt >= 0 {
            self.time_changed_show_cnt -= 1;
        }
        if ship.is_replay_mode() && self.time < 0 {
            ship.game_over();
        }
    }

    fn set_next_extend(&mut self, level: f32) {
        self.next_extend += u32::min(
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

    // This function consumes the accumulator on purpose.
    fn add_score(&mut self, score_accumulator: ScoreAccumulator, game_over: bool, level: f32) {
        if !game_over {
            self.score += score_accumulator.score;
            while self.score > self.next_extend {
                self.set_next_extend(level);
                self.extend_ship();
            }
        }
    }

    pub fn replay_data(&mut self, params: &mut ActionParams) -> ReplayData {
        self.replay_data.clone().pad_record(params.pad.get_record())
    }
}

impl<'a> State for InGameState<'a> {
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
        /* TODO sound
        if (startBgmCnt > 0) {
            startBgmCnt--;
            if (startBgmCnt <= 0)
            SoundManager.nextBgm();
        }
        */
        let mut score_accumulator = ScoreAccumulator { score: 0 };
        params.ship.mov(
            *pad,
            params.camera,
            params.tunnel,
            params.shots,
            params.bullets,
            params.particles,
            &mut score_accumulator,
        );
        self.add_score(
            score_accumulator,
            params.ship.is_game_over(),
            params.stage_manager.level(),
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
            self.goto_next_zone(false, params.stage_manager, params.bullets);
        }
        let mut score_accumulator = ScoreAccumulator { score: 0 };
        params.shots.mov(
            params.tunnel,
            params.ship,
            params.bullets,
            params.enemies,
            params.particles,
            &mut params.float_letters,
            &mut score_accumulator,
        );
        self.add_score(
            score_accumulator,
            params.ship.is_game_over(),
            params.stage_manager.level(),
        );
        params
            .bullets
            .mov(params.tunnel, params.ship, params.particles);
        params.particles.mov(params.ship.speed(), params.tunnel);
        params.float_letters.mov();
        self.decrement_time(params.ship);
        let mut action = MoveAction::None;
        if self.time < 0 {
            self.time = 0;
            if !params.ship.is_game_over() {
                params.ship.game_over();
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

    fn draw_front(&self, params: &ActionParams, _render_args: &RenderArgs) {
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

pub struct ScoreAccumulator {
    pub score: u32,
}

impl ScoreAccumulator {
    pub fn add_score(&mut self, score: u32) {
        self.score += score;
    }
}
