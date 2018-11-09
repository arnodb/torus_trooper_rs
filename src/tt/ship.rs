use crate::gl;
use crate::glu;

use crate::util::rand::Rand;
use crate::util::vector::{Vector, Vector3};

use crate::tt::actor::shot::ShotPool;
use crate::tt::bullet::BulletTarget;
use crate::tt::camera::Camera;
use crate::tt::pad::{Pad, PadButtons, PadDirection};
use crate::tt::screen::Screen;
use crate::tt::shape::ship_shape::ShipShape;
use crate::tt::shape::Drawable;
use crate::tt::state::in_game::ScoreAccumulator;
use crate::tt::tunnel::{Tunnel, DEFAULT_RAD};
use crate::tt::DrawParams;

pub const GRADE_NUM: usize = 3;
pub const GRADE_LETTER: [&str; 3] = ["N", "H", "E"];
pub const GRADE_STR: [&str; 3] = ["NORMAL", "HARD", "EXTREME"];

pub const IN_SIGHT_DEPTH_DEFAULT: f32 = 35.;
const RELPOS_MAX_Y: f32 = 10.;

const RESTART_CNT: i32 = 268;
const INVINCIBLE_CNT: i32 = 228;

const SPEED_DEFAULT: [f32; GRADE_NUM] = [0.4, 0.6, 0.8];
const SPEED_MAX: [f32; GRADE_NUM] = [0.8, 1.2, 1.6];
const ACCEL_RATIO: [f32; GRADE_NUM] = [0.002, 0.003, 0.004];

const BANK_MAX_DEFAULT: [f32; GRADE_NUM] = [0.8, 1.0, 1.2];
const OUT_OF_COURSE_BANK: f32 = 1.0;
const RELPOS_Y_MOVE: f32 = 0.1;

const FIRE_INTERVAL: u32 = 2;
const STAR_SHELL_INTERVAL: u32 = 7;

const GUNPOINT_WIDTH: f32 = 0.05;

pub struct Ship {
    replay_mode: bool,
    camera_mode: bool,
    draw_front_mode: bool,
    is_game_over: bool,

    rand: Rand,
    pos: Vector,
    rel_pos: Vector,
    eye_pos: Vector,
    rocket_pos: Vector,

    d1: f32,
    d2: f32,
    grade: u32,
    next_star_app_dist: f32,
    lap: u32,

    target_speed: f32,
    speed: f32,
    in_sight_depth: f32,

    bank: f32,
    bank_max: f32,
    tunnel_ofs: f32,
    pos3: Vector3,
    shape: ShipShape,

    charging_shot: bool,
    regenerative_charge: f32,
    fire_cnt: u32,
    fire_shot_cnt: u32,
    side_fire_cnt: u32,
    side_fire_shot_cnt: u32,
    gunpoint_pos: Vector,

    rank: u32,
    in_boss_mode: bool,
    is_boss_mode_end: bool,
    cnt: i32,

    screen_shake_cnt: u32,
    screen_shake_intense: f32,

    btn_pressed: bool,
}

impl Ship {
    // TODO split what is memorized and what is always reset
    pub fn new(screen: &Screen, seed: u64) -> Self {
        Ship {
            replay_mode: false,
            camera_mode: true,
            draw_front_mode: true,
            is_game_over: false,

            rand: Rand::new(seed),
            pos: Vector::default(),
            rel_pos: Vector::default(),
            eye_pos: Vector::default(),
            rocket_pos: Vector::default(),

            d1: 0.,
            d2: 0.,
            grade: 0,
            next_star_app_dist: 0.,
            lap: 1,

            target_speed: 0.,
            speed: 0.,
            in_sight_depth: IN_SIGHT_DEPTH_DEFAULT,

            bank: 0.,
            bank_max: BANK_MAX_DEFAULT[0],
            tunnel_ofs: 0.,
            pos3: Vector3::default(),
            shape: ShipShape::new_small(false, screen, seed),

            charging_shot: false,
            regenerative_charge: 0.,
            fire_cnt: 0,
            fire_shot_cnt: 0,
            side_fire_cnt: 0,
            side_fire_shot_cnt: 0,
            gunpoint_pos: Vector::default(),

            rank: 0,
            in_boss_mode: false,
            is_boss_mode_end: true,
            cnt: -INVINCIBLE_CNT,

            screen_shake_cnt: 0,
            screen_shake_intense: 0.,

            btn_pressed: true,
        }
    }

    pub fn start(&mut self, replay_mode: bool, grd: u32, seed: u64, camera: &mut Camera) {
        self.replay_mode = replay_mode;
        self.rand.set_seed(seed);
        self.grade = grd;
        self.tunnel_ofs = 0.;
        self.pos = Vector::default();
        self.rel_pos = Vector::default();
        self.eye_pos = Vector::default();
        self.bank = 0.;
        self.speed = 0.;
        self.d1 = 0.;
        self.d2 = 0.;
        self.cnt = -INVINCIBLE_CNT;
        self.fire_shot_cnt = 0;
        self.side_fire_shot_cnt = 0;
        self.in_sight_depth = IN_SIGHT_DEPTH_DEFAULT;
        self.rank = 0;
        self.bank_max = BANK_MAX_DEFAULT[self.grade as usize];
        self.next_star_app_dist = 0.;
        self.lap = 1;
        self.is_game_over = false;
        self.restart();
        if self.replay_mode {
            camera.start();
        }
        self.btn_pressed = true;
    }

    fn restart(&mut self) {
        self.target_speed = 0.;
        self.fire_shot_cnt = 0;
        self.side_fire_shot_cnt = 99999;
        self.charging_shot = false;
        self.regenerative_charge = 0.;
    }

    pub fn mov(
        &mut self,
        pad: &Pad,
        camera: &mut Camera,
        tunnel: &mut Tunnel,
        shots: &mut ShotPool,
        score_accumulator: &mut ScoreAccumulator
    ) {
        self.cnt += 1;
        let (mut btn, mut dir) = if !self.replay_mode {
            // TODO pad.record();
            (pad.get_buttons(), pad.get_direction())
        } else {
            /* TODO
            let ps = pad.replay();
            if (ps == RecordablePad.REPLAY_END) {
                ps = 0;
                isGameOver = true;
            }
            dir = ps & (Pad.Dir.UP | Pad.Dir.DOWN | Pad.Dir.LEFT | Pad.Dir.RIGHT);
            btn = ps & Pad.Button.ANY;*/
            (PadButtons::NONE, PadDirection::NONE)
        };
        if self.btn_pressed {
            if btn != PadButtons::NONE {
                btn = PadButtons::NONE;
            } else {
                self.btn_pressed = false;
            }
        }
        if self.is_game_over {
            btn = PadButtons::NONE;
            dir = PadDirection::NONE;
            self.speed *= 0.9;
            // TODO clearVisibleBullets();
            if self.cnt < -INVINCIBLE_CNT {
                self.cnt = -RESTART_CNT;
            }
        } else if self.cnt < -INVINCIBLE_CNT {
            btn = PadButtons::NONE;
            dir = PadDirection::NONE;
            self.rel_pos.y *= 0.99;
            //TODO clearVisibleBullets();
        }
        let mut aspeed = self.target_speed;
        if btn & PadButtons::B != PadButtons::NONE {
            aspeed *= 0.5;
        } else {
            let acc = self.regenerative_charge * 0.1;
            self.speed += acc;
            aspeed += acc;
            self.regenerative_charge -= acc;
        }
        if self.speed < aspeed {
            self.speed += (aspeed - self.speed) * 0.015;
        } else {
            if btn & PadButtons::B != PadButtons::NONE {
                self.regenerative_charge -= (aspeed - self.speed) * 0.05;
            }
            self.speed += (aspeed - self.speed) * 0.05;
        }
        self.pos.y += self.speed;
        self.tunnel_ofs += self.speed;
        let tmv = self.tunnel_ofs as usize;
        tunnel.go_to_next_slice(tmv);
        score_accumulator.add_score(tmv as u32);
        self.tunnel_ofs = self.pos.y - f32::floor(self.pos.y);
        if self.pos.y >= tunnel.get_torus_length() as f32 {
            self.pos.y -= tunnel.get_torus_length() as f32;
            self.lap += 1;
        }

        tunnel.set_ship_pos(self.tunnel_ofs, self.pos.y);
        tunnel.set_slices();
        tunnel.set_slices_backward();
        self.pos3 = tunnel.get_pos_v(self.rel_pos);

        if dir & PadDirection::RIGHT != PadDirection::NONE {
            self.bank += (-self.bank_max - self.bank) * 0.1;
        }
        if dir & PadDirection::LEFT != PadDirection::NONE {
            self.bank += (self.bank_max - self.bank) * 0.1;
        }
        let mut over_accel = false;
        if dir & PadDirection::UP != PadDirection::NONE {
            if self.rel_pos.y < RELPOS_MAX_Y {
                self.rel_pos.y += RELPOS_Y_MOVE;
            } else {
                self.target_speed += ACCEL_RATIO[self.grade as usize];
                if !(btn & PadButtons::B != PadButtons::NONE)
                    && !self.in_boss_mode
                    && !self.is_boss_mode_end
                {
                    over_accel = true;
                }
            }
        }
        if dir & PadDirection::DOWN != PadDirection::NONE && self.rel_pos.y > 0. {
            self.rel_pos.y -= RELPOS_Y_MOVE;
        }
        let acc = self.rel_pos.y
            * (SPEED_MAX[self.grade as usize] - SPEED_DEFAULT[self.grade as usize])
            / RELPOS_MAX_Y
            + SPEED_DEFAULT[self.grade as usize];
        if over_accel {
            self.target_speed += (acc - self.target_speed) * 0.001;
        } else if self.target_speed < acc {
            self.target_speed += (acc - self.target_speed) * 0.005;
        } else {
            self.target_speed += (acc - self.target_speed) * 0.03;
        }
        self.in_sight_depth = IN_SIGHT_DEPTH_DEFAULT * (1. + self.rel_pos.y / RELPOS_MAX_Y);
        if self.speed > SPEED_MAX[self.grade as usize] {
            self.in_sight_depth += IN_SIGHT_DEPTH_DEFAULT
                * (self.speed - SPEED_MAX[self.grade as usize])
                / SPEED_MAX[self.grade as usize]
                * 3.0;
        }
        self.bank *= 0.9;
        self.pos.x += self.bank * 0.08 * (DEFAULT_RAD / tunnel.get_radius(self.rel_pos.y));
        if self.pos.x < 0. {
            self.pos.x += std::f32::consts::PI * 2.;
        } else if self.pos.x >= std::f32::consts::PI * 2. {
            self.pos.x -= std::f32::consts::PI * 2.;
        }
        self.rel_pos.x = self.pos.x;
        let mut ox = self.rel_pos.x - self.eye_pos.x;
        if ox > std::f32::consts::PI {
            ox -= std::f32::consts::PI * 2.;
        } else if ox < -std::f32::consts::PI {
            ox += std::f32::consts::PI * 2.;
        }
        self.eye_pos.x += ox * 0.1;
        if self.eye_pos.x < 0. {
            self.eye_pos.x += std::f32::consts::PI * 2.;
        } else if self.eye_pos.x >= std::f32::consts::PI * 2. {
            self.eye_pos.x -= std::f32::consts::PI * 2.;
        }
        let sl = tunnel.get_slice(self.rel_pos.y);
        let co = tunnel.check_in_course(self.rel_pos);
        if co != 0. {
            let mut bm = (-OUT_OF_COURSE_BANK * co - self.bank) * 0.075;
            if bm > 1. {
                bm = 1.;
            } else if bm < -1. {
                bm = -1.;
            }
            self.speed *= 1. - f32::abs(bm);
            self.bank += bm;
            let mut lo = f32::abs(self.pos.x - sl.get_left_edge_deg());
            if lo > std::f32::consts::PI {
                lo = std::f32::consts::PI * 2. - lo;
            }
            let mut ro = f32::abs(self.pos.x - sl.get_right_edge_deg());
            if ro > std::f32::consts::PI {
                ro = std::f32::consts::PI * 2. - ro;
            }
            if lo > ro {
                self.pos.x = sl.get_right_edge_deg();
            } else {
                self.pos.x = sl.get_left_edge_deg();
            }
            self.rel_pos.x = self.pos.x;
        }
        self.d1 += (sl.d1() - self.d1) * 0.05;
        self.d2 += (sl.d2() - self.d2) * 0.05;

        if btn & PadButtons::B != PadButtons::NONE {
            if !self.charging_shot {
                shots.get_charging_instance_and(|shot| {
                    shot.set_charge(true);
                });
                self.charging_shot = true;
            }
        } else {
            if self.charging_shot {
                shots.release_charging_instance();
                self.charging_shot = false;
            }
            if btn & PadButtons::A != PadButtons::NONE {
                if self.fire_cnt <= 0 {
                    self.fire_cnt = FIRE_INTERVAL;
                    shots.get_instance_and(|shot| {
                        if (self.fire_shot_cnt % STAR_SHELL_INTERVAL) == 0 {
                            shot.set_charge_star(false, true);
                        } else {
                            shot.set();
                        }
                        self.gunpoint_pos.x = self.rel_pos.x
                            + GUNPOINT_WIDTH * ((self.fire_shot_cnt as f32 % 2.) * 2. - 1.);
                        self.gunpoint_pos.y = self.rel_pos.y;
                        shot.update(self.gunpoint_pos);
                        self.fire_shot_cnt += 1;
                    });
                }
                if self.side_fire_cnt <= 0 {
                    self.side_fire_cnt = 99999;
                    shots.get_instance_and(|shot| {
                        let mut side_fire_deg = (self.speed - SPEED_DEFAULT[self.grade as usize])
                            / (SPEED_MAX[self.grade as usize] - SPEED_DEFAULT[self.grade as usize])
                            * 0.1;
                        if side_fire_deg < 0.01 {
                            side_fire_deg = 0.01;
                        }
                        let mut d = side_fire_deg * (self.side_fire_shot_cnt % 5) as f32 * 0.2;
                        if (self.side_fire_shot_cnt % 2) == 1 {
                            d = -d;
                        }
                        if (self.side_fire_shot_cnt % STAR_SHELL_INTERVAL) == 0 {
                            shot.set_charge_star_deg(false, true, d);
                        } else {
                            shot.set_charge_star_deg(false, false, d);
                        }
                        self.gunpoint_pos.x = self.rel_pos.x
                            + GUNPOINT_WIDTH * ((self.fire_shot_cnt as f32 % 2.) * 2. - 1.);
                        self.gunpoint_pos.y = self.rel_pos.y;
                        shot.update(self.gunpoint_pos);
                        self.side_fire_shot_cnt += 1;
                    });
                }
            }
        }
        if self.fire_cnt > 0 {
            self.fire_cnt -= 1;
        }
        let mut ssc = 99999;
        if self.speed > SPEED_DEFAULT[self.grade as usize] * 1.33 {
            ssc = (100000.
                / ((self.speed - SPEED_DEFAULT[self.grade as usize] * 1.33) * 99999.
                    / (SPEED_MAX[self.grade as usize] - SPEED_DEFAULT[self.grade as usize])
                    + 1.)) as u32;
        }
        if self.side_fire_cnt > ssc {
            self.side_fire_cnt = ssc;
        }
        if self.side_fire_cnt > 0 {
            self.side_fire_cnt -= 1;
        }
        self.rocket_pos.x = self.rel_pos.x - self.bank * 0.1;
        self.rocket_pos.y = self.rel_pos.y;
        if self.charging_shot {
            shots.get_charging_instance_and(|shot| {
                shot.update(self.rocket_pos);
            });
        }
        /* TODO
        if self.cnt >= -INVINCIBLE_CNT {
            shape.addParticles(rocketPos, particles);
        }
        nextStarAppDist -= speed;
        if (nextStarAppDist <= 0) {
            for (int i = 0; i < 5; i++) {
                Particle pt = particles.getInstance();
                if (!pt)
                break;
                starPos.x = relPos.x + rand.nextSignedFloat(PI / 2) + PI;
                starPos.y = 32;
                pt.set(starPos, -8 - rand.nextFloat(56), PI, 0, 0,
                       0.6, 0.7, 0.9, 100, Particle.PType.STAR);
            }
            nextStarAppDist = 1;
        }
        */
        if self.screen_shake_cnt > 0 {
            self.screen_shake_cnt -= 1;
        }
        if self.replay_mode {
            camera.mov(self);
        }
    }

    pub fn set_eye_pos(&mut self, screen: &Screen, camera: &Camera, tunnel: &Tunnel) {
        let mut e;
        let mut l;
        let deg;
        if !self.replay_mode || !self.camera_mode {
            let mut epos = Vector3::new_at(self.eye_pos.x, -1.1 + self.rel_pos.y * 0.3, 30.0);
            e = tunnel.get_pos_v3(epos);
            epos = Vector3::new_at(self.eye_pos.x, epos.y + 6.0 + self.rel_pos.y * 0.3, 0.);
            l = tunnel.get_pos_v3(epos);
            deg = self.eye_pos.x;
        } else {
            e = tunnel.get_pos_v3(camera.camera_pos());
            l = tunnel.get_pos_v3(camera.look_at_pos());
            deg = camera.deg();
            unsafe {
                gl::MatrixMode(gl::GL_PROJECTION);
                gl::LoadIdentity();
                let np = screen.near_plane() * camera.zoom();
                gl::Frustum(
                    -np as f64,
                    np as f64,
                    (-np * screen.height() as f32 / screen.width() as f32) as f64,
                    (np * screen.height() as f32 / screen.width() as f32) as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
                gl::MatrixMode(gl::GL_MODELVIEW);
            }
        }
        if self.screen_shake_cnt > 0 {
            let mx = self
                .rand
                .gen_signed_f32(self.screen_shake_intense * (self.screen_shake_cnt + 6) as f32);
            let my = self
                .rand
                .gen_signed_f32(self.screen_shake_intense * (self.screen_shake_cnt + 6) as f32);
            let mz = self
                .rand
                .gen_signed_f32(self.screen_shake_intense * (self.screen_shake_cnt + 6) as f32);
            let m = Vector3::new_at(mx, my, mz);
            e += m;
            l += m;
        }
        glu::look_at(
            e.x as f64,
            e.y as f64,
            e.z as f64,
            l.x as f64,
            l.y as f64,
            l.z as f64,
            f64::sin(deg as f64),
            -f64::cos(deg as f64),
            0.,
        );
    }

    pub fn set_screen_shake(&mut self, cnt: u32, its: f32) {
        self.screen_shake_cnt = cnt;
        self.screen_shake_intense = its;
    }

    pub fn draw(&self) {
        if self.cnt < -INVINCIBLE_CNT || (self.cnt < 0 && (-self.cnt % 32) < 16) {
            return;
        }
        unsafe {
            gl::PushMatrix();
            gl::Translatef(self.pos3.x, self.pos3.y, self.pos3.z);
            gl::Rotatef(
                (self.pos.x - self.bank) * 180. / std::f32::consts::PI,
                0.,
                0.,
                1.,
            );
            gl::Rotatef(self.d1 * 180. / std::f32::consts::PI, 0., 1., 0.);
            gl::Rotatef(self.d2 * 180. / std::f32::consts::PI, 1., 0., 0.);
        }
        self.shape.draw();
        unsafe {
            gl::PopMatrix();
        }
    }

    pub fn draw_front(&self, params: &DrawParams) {
        let letter = params.letter;
        letter.draw_num((self.speed * 2500.) as usize, 490., 420., 20.);
        letter.draw_string("KM/H", 540., 445., 12.);
        letter.draw_num(self.rank as usize, 150., 432., 16.);
        letter.draw_string("/", 185., 448., 10.);
        // TODO letter.draw_num(self.zoneEndRank - self.rank, 250., 448., 10.);
        // NOT TODO
        /*Letter.drawString("LAP", 20, 388, 8, Letter.Direction.TO_RIGHT, 1);
        Letter.drawNum(lap, 120, 388, 8);
        Letter.drawString(".", 130, 386, 8);
        Letter.drawNum(cast(int) (pos.y * 1000000 / tunnel.getTorusLength()), 230, 388, 8,
        Letter.Direction.TO_RIGHT, 0, 6);*/    }

    pub fn is_replay_mode(&self) -> bool {
        self.replay_mode
    }

    pub fn camera_mode(&mut self, camera_mode: bool) {
        self.camera_mode = camera_mode;
    }

    pub fn draw_front_mode(&mut self, draw_front_mode: bool) {
        self.draw_front_mode = draw_front_mode;
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }

    pub fn game_over(&mut self) {
        self.is_game_over = true;
    }

    pub fn rel_pos(&self) -> Vector {
        self.rel_pos
    }
}

impl BulletTarget for Ship {
    fn get_target_pos(&self) -> Vector {
        self.rel_pos
    }
}
