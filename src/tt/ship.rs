use crate::gl;
use crate::glu;

use crate::util::rand::Rand;
use crate::util::vector::{Vector, Vector3};

use crate::tt::actor::bullet::BulletPool;
use crate::tt::actor::particle::{ParticlePool, ParticleSpec};
use crate::tt::actor::shot::ShotPool;
use crate::tt::actor::PoolActorRef;
use crate::tt::camera::Camera;
use crate::tt::pad::{PadButtons, PadDirection};
use crate::tt::screen::Screen;
use crate::tt::shape::ship_shape::ShipShape;
use crate::tt::shape::Drawable;
use crate::tt::tunnel::{Tunnel, DEFAULT_RAD};
use crate::tt::GeneralParams;

pub const GRADE_NUM: usize = 3;
pub const GRADE_LETTER: [&str; 3] = ["N", "H", "E"];
pub const GRADE_STR: [&str; 3] = ["NORMAL", "HARD", "EXTREME"];

pub const IN_SIGHT_DEPTH_DEFAULT: f32 = 35.;
pub const RELPOS_MAX_Y: f32 = 10.;

const RESTART_CNT: i32 = 268;
const INVINCIBLE_CNT: i32 = 228;

const HIT_WIDTH: f32 = 0.00025;

const SPEED_DEFAULT: [f32; GRADE_NUM] = [0.4, 0.6, 0.8];
const SPEED_MAX: [f32; GRADE_NUM] = [0.8, 1.2, 1.6];
const ACCEL_RATIO: [f32; GRADE_NUM] = [0.002, 0.003, 0.004];

const BANK_MAX_DEFAULT: [f32; GRADE_NUM] = [0.8, 1.0, 1.2];
const OUT_OF_COURSE_BANK: f32 = 1.0;
const RELPOS_Y_MOVE: f32 = 0.1;

const FIRE_INTERVAL: u32 = 2;
const STAR_SHELL_INTERVAL: u32 = 7;

const GUNPOINT_WIDTH: f32 = 0.05;

const MAX_BOSS_APP_RANK: u32 = 9_999_999;

pub struct Ship {
    replay_mode: bool,
    camera_mode: bool,
    draw_front_mode: bool,
    is_game_over: bool,

    rand: Rand,
    eye_rand: Rand,
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

    charging_shot: Option<PoolActorRef>,
    regenerative_charge: f32,
    fire_cnt: u32,
    fire_shot_cnt: u32,
    side_fire_cnt: u32,
    side_fire_shot_cnt: u32,
    gunpoint_pos: Vector,

    rank: u32,
    boss_app_rank: u32,
    boss_app_num: u32,
    zone_end_rank: u32,
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
            eye_rand: Rand::new_not_recorded(seed),
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

            charging_shot: None,
            regenerative_charge: 0.,
            fire_cnt: 0,
            fire_shot_cnt: 0,
            side_fire_cnt: 0,
            side_fire_shot_cnt: 0,
            gunpoint_pos: Vector::default(),

            rank: 0,
            boss_app_rank: 0,
            boss_app_num: 0,
            zone_end_rank: 0,
            in_boss_mode: false,
            is_boss_mode_end: true,
            cnt: -INVINCIBLE_CNT,

            screen_shake_cnt: 0,
            screen_shake_intense: 0.,

            btn_pressed: true,
        }
    }

    pub fn start(
        &mut self,
        replay_mode: bool,
        grd: u32,
        seed: u64,
        camera: &mut Camera,
        shots: &mut ShotPool,
    ) {
        self.replay_mode = replay_mode;
        self.rand.set_seed(seed);
        self.eye_rand.set_seed(seed);
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
        // Shot pool have been cleared one way or another, this reference is not valid any more.
        self.charging_shot = None;
        self.restart(shots);
        if self.replay_mode {
            camera.start();
        }
        self.btn_pressed = true;
    }

    fn restart(&mut self, shots: &mut ShotPool) {
        self.target_speed = 0.;
        self.fire_shot_cnt = 0;
        self.side_fire_shot_cnt = 99999;
        if let Some(charging_shot) = self.charging_shot {
            shots.release(charging_shot);
            self.charging_shot = None;
        }
        self.regenerative_charge = 0.;
    }

    pub fn mov(
        &mut self,
        params: &mut GeneralParams,
        shots: &mut ShotPool,
        bullets: &mut BulletPool,
        particles: &mut ParticlePool,
    ) {
        self.cnt += 1;
        let (mut btn, mut dir) = if !self.replay_mode {
            let ps = params.pad.record_state();
            (ps.buttons, ps.direction)
        } else {
            let ps = params.pad.replay_state();
            if let Some(ps) = ps {
                (ps.buttons, ps.direction)
            } else {
                record_event_end!(false);
                self.is_game_over = true;
                (PadButtons::NONE, PadDirection::NONE)
            }
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
            bullets.clear_visible();
            if self.cnt < -INVINCIBLE_CNT {
                self.cnt = -RESTART_CNT;
            }
        } else if self.cnt < -INVINCIBLE_CNT {
            btn = PadButtons::NONE;
            dir = PadDirection::NONE;
            self.rel_pos.y *= 0.99;
            bullets.clear_visible();
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
        params.tunnel.go_to_next_slice(tmv);
        params.add_score(tmv as u32, self.is_game_over());
        self.tunnel_ofs = self.pos.y - f32::floor(self.pos.y);
        if self.pos.y >= params.tunnel.get_torus_length() as f32 {
            self.pos.y -= params.tunnel.get_torus_length() as f32;
            self.lap += 1;
        }

        params.tunnel.set_ship_pos(self.tunnel_ofs, self.pos.y);
        params.tunnel.set_slices();
        params.tunnel.set_slices_backward();
        self.pos3 = params.tunnel.get_pos_v(self.rel_pos);

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
        self.pos.x += self.bank * 0.08 * (DEFAULT_RAD / params.tunnel.get_radius(self.rel_pos.y));
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
        let sl = params.tunnel.get_slice(self.rel_pos.y);
        let co = params.tunnel.check_in_course(self.rel_pos);
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
            if self.charging_shot.is_none() {
                let charging_shot = shots.get_charging_instance();
                shots[charging_shot].set_charge(true, params.sound_manager);
                self.charging_shot = Some(charging_shot);
            }
        } else {
            if let Some(charging_shot) = self.charging_shot {
                let release = shots[charging_shot].release(params.sound_manager);
                if release {
                    shots.release(charging_shot)
                }
                self.charging_shot = None;
            }
            if btn & PadButtons::A != PadButtons::NONE {
                if self.fire_cnt <= 0 {
                    self.fire_cnt = FIRE_INTERVAL;
                    shots.get_instance_and(|shot| {
                        if (self.fire_shot_cnt % STAR_SHELL_INTERVAL) == 0 {
                            shot.set_charge_star(false, true, params.sound_manager);
                        } else {
                            shot.set(params.sound_manager);
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
                            shot.set_charge_star_deg(false, true, d, params.sound_manager);
                        } else {
                            shot.set_charge_star_deg(false, false, d, params.sound_manager);
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
            ssc = (100_000.
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
        if let Some(charging_shot) = self.charging_shot {
            shots[charging_shot].update(self.rocket_pos);
        }
        if self.cnt >= -INVINCIBLE_CNT {
            self.shape
                .add_particles(self.rocket_pos, params.tunnel, particles);
        }
        self.next_star_app_dist -= self.speed;
        if self.next_star_app_dist <= 0. {
            for _ in 0..5 {
                let got_pt = particles.get_instance_and(|pt, particles_rand| {
                    let star_pos = Vector::new_at(
                        self.rel_pos.x
                            + self.rand.gen_signed_f32(std::f32::consts::PI / 2.)
                            + std::f32::consts::PI,
                        32.,
                    );
                    pt.set(
                        &ParticleSpec::Star,
                        star_pos,
                        -8. - self.rand.gen_f32(56.),
                        std::f32::consts::PI,
                        0.,
                        0.,
                        (0.6, 0.7, 0.9).into(),
                        100,
                        params.tunnel,
                        particles_rand,
                    );
                });
                if !got_pt {
                    break;
                }
            }
            self.next_star_app_dist = 1.;
        }
        if self.screen_shake_cnt > 0 {
            self.screen_shake_cnt -= 1;
        }
        if self.replay_mode {
            params.camera.mov(self);
        }
    }

    pub fn get_target_pos(&self) -> Vector {
        self.rel_pos
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
                let (p_width, p_height) = screen.physical_size();
                gl::Frustum(
                    -np as f64,
                    np as f64,
                    -np as f64 * p_height / p_width,
                    np as f64 * p_height / p_width,
                    0.1,
                    screen.far_plane() as f64,
                );
                gl::MatrixMode(gl::GL_MODELVIEW);
            }
        }
        if self.screen_shake_cnt > 0 {
            let mx = self
                .eye_rand
                .gen_signed_f32(self.screen_shake_intense * (self.screen_shake_cnt + 6) as f32);
            let my = self
                .eye_rand
                .gen_signed_f32(self.screen_shake_intense * (self.screen_shake_cnt + 6) as f32);
            let mz = self
                .eye_rand
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

    pub fn check_bullet_hit(
        &mut self,
        p: Vector,
        pp: Vector,
        params: &GeneralParams,
        shots: &mut ShotPool,
        particles: &mut ParticlePool,
    ) -> bool {
        if self.cnt <= 0 {
            return false;
        }
        let mut bmv = pp - p;
        if bmv.x > std::f32::consts::PI {
            bmv.x = bmv.x - std::f32::consts::PI * 2.;
        } else if bmv.x < -std::f32::consts::PI {
            bmv.x = bmv.x + std::f32::consts::PI * 2.;
        }
        let inaa = bmv.x * bmv.x + bmv.y * bmv.y;
        if inaa > 0.00001 {
            let mut sofs = self.rel_pos - p;
            if sofs.x > std::f32::consts::PI {
                sofs.x -= std::f32::consts::PI * 2.;
            } else if sofs.x < -std::f32::consts::PI {
                sofs.x += std::f32::consts::PI * 2.;
            }
            let inab = bmv.x * sofs.x + bmv.y * sofs.y;
            if inab >= 0. && inab <= inaa {
                let hd = sofs.x * sofs.x + sofs.y * sofs.y - inab * inab / inaa;
                if hd >= 0. && hd <= HIT_WIDTH {
                    self.destroyed(params, shots, particles);
                    return true;
                }
            }
        }
        false
    }

    fn destroyed(
        &mut self,
        params: &GeneralParams,
        shots: &mut ShotPool,
        particles: &mut ParticlePool,
    ) {
        if self.cnt <= 0 {
            return;
        }
        for _ in 0..256 {
            particles.get_instance_forced_and(|pt, rand| {
                pt.set(
                    &ParticleSpec::Spark,
                    self.rel_pos,
                    1.,
                    rand.gen_signed_f32(std::f32::consts::PI / 8.),
                    rand.gen_signed_f32(2.5),
                    0.5 + rand.gen_f32(1.),
                    (1., 0.2 + rand.gen_f32(0.8), 0.2).into(),
                    32,
                    params.tunnel,
                    rand,
                );
            });
        }
        params.sound_manager.play_se("myship_dest.wav");
        self.set_screen_shake(32, 0.05);
        self.restart(shots);
        self.cnt = -RESTART_CNT;
    }

    pub fn has_collision(&self) -> bool {
        !(self.cnt <= -INVINCIBLE_CNT)
    }

    pub fn rank_up(&mut self, is_boss: bool) -> bool {
        if (self.in_boss_mode && !is_boss) || self.is_game_over {
            return false;
        }
        let mut goto_next_zone = false;
        if self.in_boss_mode {
            self.boss_app_num -= 1;
            if self.boss_app_num <= 0 {
                self.rank += 1;
                goto_next_zone = true;
                self.in_boss_mode = false;
                self.is_boss_mode_end = true;
                self.boss_app_rank = MAX_BOSS_APP_RANK;
            }
        }
        if self.rank < self.zone_end_rank {
            self.rank += 1;
        }
        if self.rank >= self.boss_app_rank {
            self.in_boss_mode = true;
        }
        goto_next_zone
    }

    pub fn goto_next_zone_forced(&mut self) {
        self.boss_app_num = 0;
        self.in_boss_mode = false;
        self.is_boss_mode_end = true;
        self.boss_app_rank = MAX_BOSS_APP_RANK;
    }

    pub fn start_next_zone(&mut self) {
        self.is_boss_mode_end = false;
    }

    pub fn rank_down(&mut self) {
        if !self.in_boss_mode {
            self.rank -= 1;
        }
    }

    pub fn set_boss_app(&mut self, rank: u32, num: u32, zone_end_rank: u32) {
        self.boss_app_rank = rank;
        self.boss_app_num = num;
        self.zone_end_rank = zone_end_rank;
        self.in_boss_mode = false;
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

    pub fn draw_front(&self, params: &GeneralParams) {
        let letter = params.letter;
        let (o_width, o_height) = params.screen.ortho_size();
        letter.draw_num(
            (self.speed * 2500.) as usize,
            o_width as f32 - 150.,
            o_height as f32 - 60.,
            20.,
        );
        letter.draw_string("KM/H", o_width as f32 - 100., o_height as f32 - 35., 12.);
        letter.draw_num(self.rank as usize, 150., o_height as f32 - 48., 16.);
        letter.draw_string("/", 185., o_height as f32 - 32., 10.);
        letter.draw_num(
            (self.zone_end_rank - self.rank) as usize,
            250.,
            o_height as f32 - 32.,
            10.,
        );
        /*Letter.drawString("LAP", 20, 388, 8, Letter.Direction.TO_RIGHT, 1);
        Letter.drawNum(lap, 120, 388, 8);
        Letter.drawString(".", 130, 386, 8);
        Letter.drawNum(cast(int) (pos.y * 1000000 / tunnel.getTorusLength()), 230, 388, 8,
        Letter.Direction.TO_RIGHT, 0, 6);*/
    }

    pub fn is_replay_mode(&self) -> bool {
        self.replay_mode
    }

    pub fn camera_mode(&mut self, camera_mode: bool) {
        self.camera_mode = camera_mode;
    }

    pub fn is_draw_front_mode(&self) -> bool {
        self.draw_front_mode
    }

    pub fn draw_front_mode(&mut self, draw_front_mode: bool) {
        self.draw_front_mode = draw_front_mode;
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }

    pub fn game_over(&mut self) {
        record_event_end!(true);
        self.is_game_over = true;
    }

    pub fn pos(&self) -> Vector {
        self.pos
    }

    pub fn rel_pos(&self) -> Vector {
        self.rel_pos
    }

    pub fn eye_pos(&self) -> Vector {
        self.eye_pos
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn in_sight_depth(&self) -> f32 {
        self.in_sight_depth
    }

    pub fn in_boss_mode(&self) -> bool {
        self.in_boss_mode
    }

    pub fn is_boss_mode_end(&self) -> bool {
        self.is_boss_mode_end
    }

    pub fn shape(&self) -> &ShipShape {
        &self.shape
    }
}
