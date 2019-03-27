use crate::tt::actor::bullet::{Bullet, BulletPool};
use crate::tt::actor::float_letter::FloatLetterPool;
use crate::tt::actor::particle::{ParticlePool, ParticleSpec};
use crate::tt::actor::shot::Shot;
use crate::tt::actor::{Pool, PoolActorRef};
use crate::tt::barrage::BarrageManager;
use crate::tt::manager::stage::StageManager;
use crate::tt::screen::Screen;
use crate::tt::shape::bit_shape::BitShape;
use crate::tt::shape::{Collidable, Drawable};
use crate::tt::ship::{self, Ship};
use crate::tt::sound::SoundManager;
use crate::tt::state::shared::SharedState;
use crate::tt::tunnel::{self, Tunnel};
use crate::util::rand::Rand;
use crate::util::vector::Vector;

use crate::gl;

use self::ship_spec::ShipSpec;

const OUT_OF_COURSE_BANK: f32 = 1.0;
const DISAP_DEPTH: f32 = -5.0;

#[derive(Default)]
pub struct Enemy {
    spec: EnemySpec,
    pos: Vector,
    ppos: Vector,
    flip_mv: Vector,
    flip_mv_cnt: u32,
    speed: f32,
    d1: f32,
    d2: f32,
    base_bank: f32,
    bank: f32,
    top_bullet: Option<PoolActorRef>,
    shield: i32,
    first_shield: i32,
    damaged: bool,
    high_order: bool,
    limit_y: f32,
    bit_bullet: Vec<PoolActorRef>,
    bit_cnt: u32,
}

#[derive(Clone, Copy, Debug)]
enum EnemySpec {
    Small(usize),
    Medium(usize),
    Boss(usize),
}

impl Default for EnemySpec {
    fn default() -> Self {
        EnemySpec::Small(0)
    }
}

pub enum EnemySetOption<'a> {
    New {
        spec: &'a ShipSpec,
        rand: &'a mut Rand,
    },
    Passed {
        shield: i32,
        base_bank: f32,
    },
}

impl Enemy {
    pub fn set(&mut self, x: f32, y: f32, option: EnemySetOption) {
        self.pos.x = x;
        self.limit_y = y;
        self.pos.y = y;
        self.speed = 0.;
        self.d1 = 0.;
        self.d2 = 0.;
        self.bank = 0.;
        match option {
            EnemySetOption::New { spec, rand } => {
                self.shield = spec.shield();
                self.base_bank = spec.create_base_bank(rand);
            }
            EnemySetOption::Passed { shield, base_bank } => {
                self.shield = shield;
                self.base_bank = base_bank;
            }
        }
        self.first_shield = self.shield;
        self.flip_mv_cnt = 0;
        self.damaged = false;
        self.high_order = true;
        self.top_bullet = None;
        self.bit_bullet.clear();
    }

    fn mov(
        &mut self,
        en_spec: EnemySpec,
        passed: bool,
        spec: &mut ShipSpec,
        tunnel: &Tunnel,
        ship: &mut Ship,
        bullets: &mut BulletPool,
        particles: &mut ParticlePool,
        passed_pool: Option<&mut Pool<Enemy>>,
    ) -> (bool, bool) {
        let mut goto_next_zone = false;
        if !passed {
            if self.high_order {
                if self.pos.y <= ship.rel_pos().y {
                    goto_next_zone = ship.rank_up(spec.is_boss());
                    self.high_order = false;
                }
            } else {
                if self.pos.y > ship.rel_pos().y {
                    ship.rank_down();
                    self.high_order = true;
                }
            }
        }
        self.ppos = self.pos;
        if ship.is_boss_mode_end() {
            self.speed -= self.speed * 0.05;
            self.flip_mv_cnt = 0;
        } else if !ship.has_collision() {
            self.speed += (1.5 - self.speed) * 0.15;
        }
        if spec.has_limit_y() {
            self.speed = spec.speed_ship(self.speed, ship.speed());
        } else if self.pos.y > 5. && self.pos.y < ship::IN_SIGHT_DEPTH_DEFAULT * 2. {
            self.speed = spec.speed_ship(self.speed, ship.speed());
        } else {
            self.speed = spec.speed(self.speed);
        }
        let mut my = self.speed - ship.speed();
        if passed && my > 0. {
            my = 0.;
        }
        self.pos.y += my;
        if !passed && spec.has_limit_y() {
            let py = self.pos.y;
            let limit_y = self.limit_y;
            self.handle_limit_y(py, limit_y);
        }
        let mut steer = false;
        if let Some((ld, rd)) = spec.get_range_of_movement(self.pos, tunnel) {
            let cdf = Tunnel::check_deg_inside(self.pos.x, ld, rd);
            if cdf != 0 {
                steer = true;
                if cdf == -1 {
                    self.bank = spec.try_to_move(self.bank, self.pos.x, ld);
                } else if cdf == 1 {
                    self.bank = spec.try_to_move(self.bank, self.pos.x, rd);
                }
            }
        }
        if !steer {
            if spec.aim_ship() {
                let mut ox = f32::abs(self.pos.x - ship.pos().x);
                if ox > std::f32::consts::PI {
                    ox = std::f32::consts::PI * 2. - ox;
                }
                if ox > std::f32::consts::PI / 3. {
                    steer = true;
                    self.bank = spec.try_to_move(self.bank, self.pos.x, ship.pos().x);
                }
            }
        }
        if !steer {
            self.bank += (self.base_bank - self.bank) * 0.2;
        }
        self.bank *= 0.9;
        self.pos.x += self.bank * 0.08 * (tunnel::DEFAULT_RAD / tunnel.get_radius(self.pos.y));
        if self.flip_mv_cnt > 0 {
            self.flip_mv_cnt -= 1;
            self.pos += self.flip_mv;
            self.flip_mv *= 0.95;
        }
        if self.pos.x < 0. {
            self.pos.x += std::f32::consts::PI * 2.;
        } else if self.pos.x >= std::f32::consts::PI * 2. {
            self.pos.x -= std::f32::consts::PI * 2.;
        }
        if !passed && self.flip_mv_cnt <= 0 && !ship.is_boss_mode_end() {
            let mut ax = f32::abs(self.pos.x - ship.rel_pos().x);
            if ax > std::f32::consts::PI {
                ax = std::f32::consts::PI * 2. - ax;
            }
            ax *= (tunnel.get_radius(0.) / tunnel::DEFAULT_RAD) * 3.;
            let ay = f32::abs(self.pos.y - ship.rel_pos().y);
            if ship.has_collision()
                && spec
                    .shape()
                    .check_collision_shape(ax, ay, ship.shape(), ship.speed())
            {
                let mut ox = self.ppos.x - ship.pos().x;
                if ox > std::f32::consts::PI {
                    ox -= std::f32::consts::PI * 2.;
                } else if ox < -std::f32::consts::PI {
                    ox += std::f32::consts::PI * 2.;
                }
                let oy = self.ppos.y;
                let od = f32::atan2(ox, oy);
                self.flip_mv_cnt = 48;
                self.flip_mv = Vector::new_at(
                    f32::sin(od) * ship.speed() * 0.4,
                    f32::cos(od) * ship.speed() * 7.,
                );
            }
        }
        let sl = tunnel.get_slice(self.pos.y);
        let co = tunnel.check_in_course(self.pos);
        // TODO epsilon
        if co != 0. {
            let bm = f32::max(
                f32::min((-OUT_OF_COURSE_BANK * co - self.bank) * 0.075, 1.),
                -1.,
            );
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
        }
        self.d1 += (sl.d1() - self.d1) * 0.1;
        self.d2 += (sl.d2() - self.d2) * 0.1;
        if !passed && self.top_bullet.is_none() {
            self.top_bullet = spec.barrage().add_top_bullet(bullets);
            if let Some(bit_spec) = &mut spec.bit_spec {
                for _ in 0..bit_spec.bit_num {
                    let ba = bit_spec.bit_barrage.add_top_bullet(bullets);
                    if let Some(ba) = ba {
                        let ba_inst = &mut bullets[ba];
                        ba_inst.unset_aim_top();
                        self.bit_bullet.push(ba);
                    }
                }
            }
        }
        if let Some(tb_ref) = self.top_bullet {
            if let Some(mut top_bullet) = bullets.maybe_index_mut(tb_ref) {
                top_bullet.options.as_mut().unwrap().bullet.pos = self.pos;
                self.check_bullet_in_range(spec, tunnel, ship, &mut top_bullet);
            } else {
                self.top_bullet = None;
            }
            for (i, bb) in self.bit_bullet.iter().enumerate() {
                let bb_inst = &mut bullets[*bb];
                let (bit_offset, d) = spec
                    .bit_spec
                    .as_ref()
                    .unwrap()
                    .get_bit_offset(i as u32, self.bit_cnt);
                {
                    let bb_bullet = &mut bb_inst.options.as_mut().unwrap().bullet;
                    bb_bullet.pos.x = bit_offset.x + self.pos.x;
                    bb_bullet.pos.y = bit_offset.y + self.pos.y;
                    bb_bullet.deg = d;
                }
                self.check_bullet_in_range(spec, tunnel, ship, bb_inst);
            }
        }
        if !passed && self.pos.y <= ship.in_sight_depth() {
            spec.shape().add_particles(self.pos, tunnel, particles);
        }
        let mut release = false;
        if !passed {
            if let Some(passed_pool) = passed_pool {
                if (!spec.has_limit_y() && self.pos.y > ship::IN_SIGHT_DEPTH_DEFAULT * 5.)
                    || self.pos.y < DISAP_DEPTH
                {
                    if ship.is_replay_mode() && self.pos.y < DISAP_DEPTH {
                        let inst = passed_pool.get_instance();
                        if let Some((enemy, _)) = inst {
                            enemy.spec = en_spec;
                            enemy.set(
                                self.pos.x,
                                self.pos.y,
                                EnemySetOption::Passed {
                                    shield: spec.shield(),
                                    base_bank: self.base_bank,
                                },
                            );
                        }
                    }
                    release = true;
                }
            }
        } else if self.pos.y < -ship::IN_SIGHT_DEPTH_DEFAULT * 3. {
            release = true;
        }
        self.damaged = false;
        self.bit_cnt += 1;
        (release, goto_next_zone)
    }

    fn check_bullet_in_range(
        &self,
        spec: &mut ShipSpec,
        tunnel: &Tunnel,
        ship: &Ship,
        top_bullet: &mut Bullet,
    ) {
        if !tunnel.check_in_screen(self.pos, ship) {
            top_bullet.root_rank = 0.;
        } else {
            let ship_rel_pos = ship.rel_pos();
            if self.pos.dist(ship_rel_pos) > 20. + ship_rel_pos.y * 10. / ship::RELPOS_MAX_Y
                && self.pos.y > ship_rel_pos.y
                && self.flip_mv_cnt <= 0
            {
                if spec.no_fire_depth_limit {
                    top_bullet.root_rank = 1.;
                } else if self.pos.y <= ship.in_sight_depth() {
                    top_bullet.root_rank = 1.;
                } else {
                    top_bullet.root_rank = 0.;
                }
            } else {
                top_bullet.root_rank = 0.;
            }
        }
    }

    fn check_shot_hit(
        &mut self,
        spec: &ShipSpec,
        shot: &mut Shot,
        tunnel: &Tunnel,
        shared_state: &mut SharedState,
        stage_manager: &StageManager,
        sound_manager: &SoundManager,
        ship: &mut Ship,
        particles: &mut ParticlePool,
        float_letters: &mut FloatLetterPool,
        rand: &mut Rand,
    ) -> (bool, bool) {
        let mut ox = f32::abs(self.pos.x - shot.pos.x);
        let oy = f32::abs(self.pos.y - shot.pos.y);
        if ox > std::f32::consts::PI {
            ox = std::f32::consts::PI * 2. - ox;
        }
        ox *= (tunnel.get_radius(self.pos.y) / tunnel::DEFAULT_RAD) * 3.;
        let mut release_enemy = false;
        let mut release_shot = false;
        if spec
            .shape()
            .check_collision_shape(ox, oy, shot.shape.as_ref().unwrap(), 1.)
        {
            self.shield -= shot.damage();
            if self.shield <= 0 {
                self.destroyed(spec, sound_manager, tunnel, ship, particles, rand);
                release_enemy = true;
            } else {
                self.damaged = true;
                for _ in 0..4 {
                    particles.get_instance_and(|pt, particles_rand| {
                        pt.set(
                            &ParticleSpec::Spark,
                            self.pos,
                            1.,
                            rand.gen_signed_f32(0.1),
                            rand.gen_signed_f32(1.6),
                            0.75,
                            1.,
                            0.4 + rand.gen_f32(0.4),
                            0.3,
                            16,
                            tunnel,
                            particles_rand,
                        );
                    });
                    particles.get_instance_and(|pt, particles_rand| {
                        pt.set(
                            &ParticleSpec::Spark,
                            self.pos,
                            1.,
                            rand.gen_signed_f32(0.1) + std::f32::consts::PI,
                            rand.gen_signed_f32(1.6),
                            0.75,
                            1.,
                            0.4 + rand.gen_f32(0.4),
                            0.3,
                            16,
                            tunnel,
                            particles_rand,
                        );
                    });
                }
                sound_manager.play_se("hit.wav");
            }
            release_shot = shot.add_score(
                spec.score(),
                self.pos,
                shared_state,
                stage_manager,
                sound_manager,
                ship,
                float_letters,
            );
        }
        (release_enemy, release_shot)
    }

    fn destroyed(
        &self,
        spec: &ShipSpec,
        sound_manager: &SoundManager,
        tunnel: &Tunnel,
        ship: &mut Ship,
        particles: &mut ParticlePool,
        rand: &mut Rand,
    ) {
        for _ in 0..30 {
            let got_pt = particles.get_instance_and(|pt, particles_rand| {
                pt.set(
                    &ParticleSpec::Spark,
                    self.pos,
                    1.,
                    rand.gen_f32(std::f32::consts::PI * 2.),
                    rand.gen_signed_f32(1.),
                    0.01 + rand.gen_f32(0.1),
                    1.,
                    0.2 + rand.gen_f32(0.8),
                    0.4,
                    24,
                    tunnel,
                    particles_rand,
                );
            });
            if !got_pt {
                break;
            }
        }
        spec.shape.add_fragments(self.pos, tunnel, particles, rand);
        ship.rank_up(spec.is_boss());
        if self.first_shield == 1 {
            sound_manager.play_se("small_dest.wav");
        } else if self.first_shield < 20 {
            sound_manager.play_se("middle_dest.wav");
        } else {
            sound_manager.play_se("boss_dest.wav");
            ship.set_screen_shake(56, 0.064);
        }
    }

    fn remove_shallow(&mut self) {
        self.top_bullet = None;
        self.bit_bullet.clear();
    }

    fn remove(&mut self, bullets: &mut BulletPool) {
        if let Some(tb_ref) = self.top_bullet {
            // TODO might not exist?
            bullets.release(tb_ref);
            self.top_bullet = None;
        }
        for bb_ref in &self.bit_bullet {
            // TODO might not exist?
            bullets.release(*bb_ref);
        }
        self.bit_bullet.clear();
    }

    pub fn draw(
        &self,
        spec: &ShipSpec,
        tunnel: &Tunnel,
        bullets: &BulletPool,
        bit_shape: &BitShape,
    ) {
        let sp = tunnel.get_pos_v(self.pos);
        unsafe {
            gl::PushMatrix();
        }
        sp.gl_translate();
        unsafe {
            gl::Rotatef(
                (self.pos.x - self.bank) * 180. / std::f32::consts::PI,
                0.,
                0.,
                1.,
            );
        }
        if sp.z > 200. {
            let sz = 1. - (sp.z - 200.) * 0.0025;
            unsafe {
                gl::Scalef(sz, sz, sz);
            }
        }
        unsafe {
            gl::Rotatef(self.d1 * 180. / std::f32::consts::PI, 0., 1., 0.);
            gl::Rotatef(self.d2 * 180. / std::f32::consts::PI, 1., 0., 0.);
        }
        if !self.damaged {
            spec.shape().draw();
        } else {
            spec.damaged_shape().draw();
        }
        unsafe {
            gl::PopMatrix();
        }
        for bb in &self.bit_bullet {
            let bb_inst = &bullets[*bb];
            let sp = tunnel.get_pos_v(bb_inst.options.as_ref().unwrap().bullet.pos);
            unsafe {
                gl::PushMatrix();
            }
            sp.gl_translate();
            unsafe {
                gl::Rotatef((self.bit_cnt * 7) as f32, 0., 1., 0.);
                gl::Rotatef(self.pos.x * 180. / std::f32::consts::PI, 0., 0., 1.);
            }
            bit_shape.draw();
            unsafe {
                gl::PopMatrix();
            }
        }
    }

    pub fn handle_limit_y(&mut self, y: f32, limit_y: f32) {
        if y > limit_y {
            self.pos.y += (limit_y - y) * 0.05;
            self.limit_y -= 0.01;
        } else {
            self.limit_y += (y - limit_y) * 0.05 - 0.01;
        }
    }
}

pub struct EnemyPool {
    pool: Pool<Enemy>,
    passed_pool: Pool<Enemy>,
    boss_spec_idx: usize,
    rand: Rand,
    small_ship_specs: Vec<ShipSpec>,
    medium_ship_specs: Vec<ShipSpec>,
    boss_ship_specs: Vec<ShipSpec>,
    bit_shape: BitShape,
}

impl EnemyPool {
    pub fn new(n: usize, seed: u64, screen: &Screen) -> Self {
        EnemyPool {
            pool: Pool::new(n),
            passed_pool: Pool::new(n),
            boss_spec_idx: 0,
            rand: Rand::new(seed),
            small_ship_specs: Vec::new(),
            medium_ship_specs: Vec::new(),
            boss_ship_specs: Vec::new(),
            bit_shape: BitShape::new(screen),
        }
    }

    pub fn renew_ship_specs(
        &mut self,
        level: f32,
        grade: u32,
        medium_boss_zone: bool,
        boss_num: u32,
        screen: &Screen,
        barrage_manager: &mut BarrageManager,
    ) {
        self.small_ship_specs.clear();
        self.medium_ship_specs.clear();
        self.boss_ship_specs.clear();
        for _ in 0..(2 + self.rand.gen_usize(2)) {
            let ss =
                ShipSpec::new_small(&mut self.rand, level * 1.8, grade, screen, barrage_manager);
            self.small_ship_specs.push(ss);
        }
        for _ in 0..(2 + self.rand.gen_usize(2)) {
            let ss = ShipSpec::new_medium(&mut self.rand, level * 1.9, screen, barrage_manager);
            self.medium_ship_specs.push(ss);
        }
        for _ in 0..boss_num {
            let mut lv = level * 2.0 / boss_num as f32;
            if medium_boss_zone {
                lv *= 1.33;
            }
            let boss_speed = 0.8 + grade as f32 * 0.04 + self.rand.gen_f32(0.03);
            let ss = ShipSpec::new_boss(
                &mut self.rand,
                lv,
                boss_speed,
                medium_boss_zone,
                screen,
                barrage_manager,
            );
            self.boss_ship_specs.push(ss);
        }
        self.boss_spec_idx = 0;
    }

    pub fn get_small_instance_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Enemy, &ShipSpec),
    {
        let idx = self.rand.gen_usize(self.small_ship_specs.len());
        let spec = &self.small_ship_specs[idx];
        let inst = self.pool.get_instance();
        if let Some((enemy, _)) = inst {
            enemy.spec = EnemySpec::Small(idx);
            op(enemy, spec);
        }
    }

    pub fn get_medium_instance_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Enemy, &ShipSpec),
    {
        let idx = self.rand.gen_usize(self.medium_ship_specs.len());
        let spec = &self.medium_ship_specs[idx];
        let inst = self.pool.get_instance();
        if let Some((enemy, _)) = inst {
            enemy.spec = EnemySpec::Medium(idx);
            op(enemy, spec);
        }
    }

    pub fn get_boss_instance_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Enemy, &ShipSpec),
    {
        let idx = self.boss_spec_idx;
        self.boss_spec_idx += 1;
        let spec = &self.boss_ship_specs[idx];
        let inst = self.pool.get_instance();
        if let Some((enemy, _)) = inst {
            enemy.spec = EnemySpec::Boss(idx);
            op(enemy, spec);
        }
    }

    pub fn clear_shallow(&mut self) {
        for enemy in &mut self.pool {
            enemy.remove_shallow();
        }
        self.pool.clear();
        for enemy in &mut self.passed_pool {
            enemy.remove_shallow();
        }
        self.passed_pool.clear();
    }

    pub fn clear(&mut self, bullets: &mut BulletPool) {
        for enemy in &mut self.pool {
            enemy.remove(bullets);
        }
        self.pool.clear();
        for enemy in &mut self.passed_pool {
            enemy.remove(bullets);
        }
        self.passed_pool.clear();
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rand.set_seed(seed);
    }

    pub fn mov(
        &mut self,
        tunnel: &Tunnel,
        ship: &mut Ship,
        bullets: &mut BulletPool,
        particles: &mut ParticlePool,
    ) -> bool {
        let mut goto_next_zone = false;
        for enemy_ref in self.pool.as_refs() {
            let release = {
                let enemy = &mut self.pool[enemy_ref];
                let spec = match enemy.spec {
                    EnemySpec::Small(idx) => &mut self.small_ship_specs[idx],
                    EnemySpec::Medium(idx) => &mut self.medium_ship_specs[idx],
                    EnemySpec::Boss(idx) => &mut self.boss_ship_specs[idx],
                };
                let (release, goto_nz) = enemy.mov(
                    enemy.spec,
                    false,
                    spec,
                    tunnel,
                    ship,
                    bullets,
                    particles,
                    Some(&mut self.passed_pool),
                );
                if goto_nz {
                    goto_next_zone = true;
                }
                if release {
                    enemy.remove(bullets);
                }
                release
            };
            if release {
                self.pool.release(enemy_ref);
            }
        }
        goto_next_zone
    }

    pub fn mov_passed(
        &mut self,
        tunnel: &Tunnel,
        ship: &mut Ship,
        bullets: &mut BulletPool,
        particles: &mut ParticlePool,
    ) {
        for enemy_ref in self.passed_pool.as_refs() {
            let release = {
                let enemy = &mut self.passed_pool[enemy_ref];
                let spec = match enemy.spec {
                    EnemySpec::Small(idx) => &mut self.small_ship_specs[idx],
                    EnemySpec::Medium(idx) => &mut self.medium_ship_specs[idx],
                    EnemySpec::Boss(idx) => &mut self.boss_ship_specs[idx],
                };
                let (release, _) = enemy.mov(
                    enemy.spec, true, spec, tunnel, ship, bullets, particles, None,
                );
                if release {
                    enemy.remove_shallow();
                }
                release
            };
            if release {
                self.passed_pool.release(enemy_ref);
            }
        }
    }

    pub fn check_shot_hit(
        &mut self,
        shot: &mut Shot,
        tunnel: &Tunnel,
        shared_state: &mut SharedState,
        stage_manager: &StageManager,
        sound_manager: &SoundManager,
        ship: &mut Ship,
        bullets: &mut BulletPool,
        particles: &mut ParticlePool,
        float_letters: &mut FloatLetterPool,
    ) -> bool {
        let mut release_shot = false;
        for enemy_ref in self.pool.as_refs() {
            let release_enemy = {
                let enemy = &mut self.pool[enemy_ref];
                let spec = match enemy.spec {
                    EnemySpec::Small(idx) => &self.small_ship_specs[idx],
                    EnemySpec::Medium(idx) => &self.medium_ship_specs[idx],
                    EnemySpec::Boss(idx) => &self.boss_ship_specs[idx],
                };
                let (release_enemy, rel_shot) = enemy.check_shot_hit(
                    spec,
                    shot,
                    tunnel,
                    shared_state,
                    stage_manager,
                    sound_manager,
                    ship,
                    particles,
                    float_letters,
                    &mut self.rand,
                );
                if rel_shot {
                    release_shot = true;
                }
                if release_enemy {
                    enemy.remove(bullets);
                }
                release_enemy
            };
            if release_enemy {
                self.pool.release(enemy_ref);
            }
        }
        release_shot
    }

    pub fn draw(&self, tunnel: &Tunnel, bullets: &BulletPool) {
        for enemy in &self.pool {
            let spec = match enemy.spec {
                EnemySpec::Small(idx) => &self.small_ship_specs[idx],
                EnemySpec::Medium(idx) => &self.medium_ship_specs[idx],
                EnemySpec::Boss(idx) => &self.boss_ship_specs[idx],
            };
            enemy.draw(spec, tunnel, bullets, &self.bit_shape);
        }
    }

    pub fn draw_passed(&self, tunnel: &Tunnel, bullets: &BulletPool) {
        for enemy in &self.passed_pool {
            let spec = match enemy.spec {
                EnemySpec::Small(idx) => &self.small_ship_specs[idx],
                EnemySpec::Medium(idx) => &self.medium_ship_specs[idx],
                EnemySpec::Boss(idx) => &self.boss_ship_specs[idx],
            };
            enemy.draw(spec, tunnel, bullets, &self.bit_shape);
        }
    }

    pub fn get_num(&self) -> usize {
        self.pool.get_num()
    }
}

pub mod ship_spec {
    use std::ffi::OsStr;
    use std::rc::Rc;

    use crate::util::rand::Rand;
    use crate::util::vector::Vector;

    use crate::tt::barrage::{Barrage, BarrageManager, BulletShapeType};
    use crate::tt::screen::Screen;
    use crate::tt::shape::ship_shape::ShipShape;
    use crate::tt::shape::{Drawable, ResizableDrawable};
    use crate::tt::ship;
    use crate::tt::tunnel::Tunnel;

    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    enum BitType {
        Round,
        Line,
    }

    const BIT_TYPES: [BitType; 2] = [BitType::Round, BitType::Line];

    const SPEED_CHANGE_RATIO: f32 = 0.2;

    pub struct ShipSpec {
        pub shape: ShipShape,
        damaged_shape: ShipShape,
        barrage: Barrage,
        shield: i32,
        base_speed: f32,
        ship_speed_ratio: f32,
        visual_range: f32,
        base_bank: f32,
        bank_max: f32,
        score: u32,
        pub bit_spec: Option<BitSpec>,
        aim_ship: bool,
        has_limit_y: bool,
        pub no_fire_depth_limit: bool,
        is_boss: bool,
    }

    impl ShipSpec {
        pub fn new_small(
            rand: &mut Rand,
            level: f32,
            grade: u32,
            screen: &Screen,
            barrage_manager: &mut BarrageManager,
        ) -> Self {
            let base_speed = 0.05 + rand.gen_f32(0.1);
            let ship_speed_ratio = 0.25 + rand.gen_f32(0.25);
            let visual_range = 10. + rand.gen_f32(32.);
            let base_bank = if rand.gen_usize(3) == 0 {
                0.1 + rand.gen_f32(0.2)
            } else {
                0.
            };
            let bank_max = 0.3 + rand.gen_f32(0.7);
            let rs = rand.gen_usize(99999) as u64;
            let bi_min = usize::max(usize::min((160.0 / level) as usize, 40), 80)
                + (ship::GRADE_NUM - 1 - grade as usize) * 8;
            let brg_interval =
                bi_min + rand.gen_usize(80 + (ship::GRADE_NUM - 1 - grade as usize) * 8 - bi_min);
            let brg_rank = level / (150.0 / brg_interval as f32);
            let barrage =
                ShipSpec::create_barrage(rand, brg_rank, 0, brg_interval as u32, barrage_manager);
            Self {
                shape: ShipShape::new_small(false, screen, rs),
                damaged_shape: ShipShape::new_small(true, screen, rs),
                barrage,
                shield: 1,
                base_speed,
                ship_speed_ratio,
                visual_range,
                base_bank,
                bank_max,
                score: 100,
                bit_spec: None,
                aim_ship: false,
                has_limit_y: false,
                no_fire_depth_limit: false,
                is_boss: false,
            }
        }

        pub fn new_medium(
            rand: &mut Rand,
            level: f32,
            screen: &Screen,
            barrage_manager: &mut BarrageManager,
        ) -> Self {
            let base_speed = 0.1 + rand.gen_f32(0.1);
            let ship_speed_ratio = 0.4 + rand.gen_f32(0.4);
            let visual_range = 10. + rand.gen_f32(32.);
            let base_bank = if rand.gen_usize(4) == 0 {
                0.05 + rand.gen_f32(0.1)
            } else {
                0.
            };
            let bank_max = 0.2 + rand.gen_f32(0.5);
            let rs = rand.gen_usize(99999) as u64;
            let barrage = ShipSpec::create_barrage_full(
                rand,
                level,
                0,
                0,
                1.,
                Some(OsStr::new("middle")),
                BulletShapeType::Square,
                false,
                barrage_manager,
            );
            Self {
                shape: ShipShape::new_medium(false, screen, rs),
                damaged_shape: ShipShape::new_medium(true, screen, rs),
                barrage,
                shield: 10,
                base_speed,
                ship_speed_ratio,
                visual_range,
                base_bank,
                bank_max,
                score: 500,
                bit_spec: None,
                aim_ship: false,
                has_limit_y: false,
                no_fire_depth_limit: false,
                is_boss: false,
            }
        }

        pub fn new_boss(
            rand: &mut Rand,
            level: f32,
            speed: f32,
            medium_boss: bool,
            screen: &Screen,
            barrage_manager: &mut BarrageManager,
        ) -> Self {
            let base_speed = 0.1 + rand.gen_f32(0.1);
            let visual_range = 16. + rand.gen_f32(24.);
            let bank_max = 0.8 + rand.gen_f32(0.4);
            let rs = rand.gen_usize(99999) as u64;
            let barrage = ShipSpec::create_barrage_full(
                rand,
                level,
                0,
                0,
                1.2,
                Some(OsStr::new("middle")),
                BulletShapeType::Square,
                true,
                barrage_manager,
            );
            let bit_spec = if !medium_boss {
                let bit_num = 2 + rand.gen_usize(3) as u32 * 2;
                let bit_type = BIT_TYPES[rand.gen_usize(2)];
                let bit_distance = 0.33 + rand.gen_f32(0.3);
                let bit_md = 0.02 + rand.gen_f32(0.02);
                let mut bit_brg_rank = level / (bit_num / 2) as f32;
                let bi_min = f32::max(f32::min(120.0 / bit_brg_rank, 60.), 20.) as u32;
                let brg_interval = bi_min + rand.gen_usize(60 - bi_min as usize) as u32;
                bit_brg_rank /= 60.0 / brg_interval as f32;
                let mut bit_barrage = ShipSpec::create_barrage_full(
                    rand,
                    bit_brg_rank,
                    0,
                    brg_interval,
                    1.,
                    None,
                    BulletShapeType::Bar,
                    true,
                    barrage_manager,
                );
                bit_barrage.set_no_x_reverse();
                Some(BitSpec {
                    bit_num,
                    bit_type,
                    bit_distance,
                    bit_md,
                    bit_barrage,
                })
            } else {
                None
            };
            Self {
                shape: ShipShape::new_large(false, screen, rs),
                damaged_shape: ShipShape::new_large(true, screen, rs),
                barrage,
                shield: 30,
                base_speed,
                ship_speed_ratio: speed,
                visual_range,
                base_bank: 0.,
                bank_max,
                score: 2000,
                bit_spec,
                aim_ship: true,
                has_limit_y: true,
                no_fire_depth_limit: true,
                is_boss: true,
            }
        }

        fn create_barrage(
            rand: &mut Rand,
            level: f32,
            pre_wait: u32,
            post_wait: u32,
            barrage_manager: &mut BarrageManager,
        ) -> Barrage {
            ShipSpec::create_barrage_full(
                rand,
                level,
                pre_wait,
                post_wait,
                1.,
                None,
                BulletShapeType::Triangle,
                false,
                barrage_manager,
            )
        }

        fn create_barrage_full(
            rand: &mut Rand,
            level: f32,
            pre_wait: u32,
            post_wait: u32,
            size: f32,
            base_dir: Option<&OsStr>,
            shape_type: BulletShapeType,
            long_range: bool,
            barrage_manager: &mut BarrageManager,
        ) -> Barrage {
            // TODO
            let mut rank = f32::sqrt(level) / (8. - rand.gen_usize(3) as f32);
            if rank > 0.8 {
                rank = rand.gen_f32(0.2) + 0.8;
            }
            let mut speed_rank = f32::sqrt(rank) * (rand.gen_f32(0.2) + 0.8);
            if speed_rank < 1. {
                speed_rank = 1.;
            }
            if speed_rank > 2. {
                speed_rank = f32::sqrt(speed_rank * 2.);
            }
            let mut morph_rank = level / (rank + 2.) / speed_rank;
            let mut morph_cnt = 0;
            while morph_rank > 1. {
                morph_cnt += 1;
                morph_rank /= 3.;
            }
            let (bsr, dbsr) = barrage_manager.get_shape(shape_type);
            let bsr = Rc::new(ResizableDrawable::new(bsr, size * 1.25)) as Rc<Drawable>;
            let dbsr = Rc::new(ResizableDrawable::new(dbsr, size * 1.25)) as Rc<Drawable>;
            let mut br = Barrage::new(&bsr, &dbsr);
            br.set_wait(pre_wait, post_wait);
            br.set_long_range(long_range);
            if let Some(base_dir) = base_dir {
                let ps = barrage_manager.get_instance_list(base_dir);
                let pi = rand.gen_usize(ps.len());
                br.add_bml(&ps[pi].1, rank, true, speed_rank);
            } else {
                br.add_bml(
                    barrage_manager.get_instance(OsStr::new("basic"), OsStr::new("straight.xml")),
                    rank,
                    true,
                    speed_rank,
                );
            }
            let ps = barrage_manager.get_instance_list(OsStr::new("morph"));
            let psn = ps.len();
            let mut used_ps = vec![false; psn];
            for _ in 0..morph_cnt {
                let mut pi = rand.gen_usize(psn);
                while used_ps[pi] {
                    pi = (pi + psn - 1) % psn;
                }
                br.add_bml(&ps[pi].1, morph_rank, true, speed_rank);
                used_ps[pi] = true;
            }
            br
        }

        pub fn speed(&self, sp: f32) -> f32 {
            ShipSpec::change_speed(sp, self.base_speed)
        }

        pub fn speed_ship(&self, sp: f32, ship_sp: f32) -> f32 {
            let asp = ship_sp * self.ship_speed_ratio;
            if asp > self.base_speed {
                ShipSpec::change_speed(sp, asp)
            } else {
                ShipSpec::change_speed(sp, self.base_speed)
            }
        }

        fn change_speed(sp: f32, aim: f32) -> f32 {
            sp + (aim - sp) * SPEED_CHANGE_RATIO
        }

        pub fn get_range_of_movement(&self, p: Vector, tunnel: &Tunnel) -> Option<(f32, f32)> {
            let mut py: f32 = p.y;
            let cs = tunnel.get_slice(py);
            py += self.visual_range;
            let vs = tunnel.get_slice(py);
            if !cs.is_nearly_round() {
                let mut from = cs.get_left_edge_deg();
                let mut to = cs.get_right_edge_deg();
                if !vs.is_nearly_round() {
                    let vld = vs.get_left_edge_deg();
                    let vrd = vs.get_right_edge_deg();
                    if Tunnel::check_deg_inside(from, vld, vrd) == -1 {
                        from = vld;
                    }
                    if Tunnel::check_deg_inside(to, vld, vrd) == 1 {
                        to = vrd;
                    }
                }
                Some((from, to))
            } else if !vs.is_nearly_round() {
                let from = vs.get_left_edge_deg();
                let to = vs.get_right_edge_deg();
                Some((from, to))
            } else {
                None
            }
        }

        pub fn try_to_move(&self, bank: f32, deg: f32, aim_deg: f32) -> f32 {
            let mut bk = aim_deg - deg;
            if bk > std::f32::consts::PI {
                bk -= std::f32::consts::PI * 2.;
            } else if bk < -std::f32::consts::PI {
                bk += std::f32::consts::PI * 2.;
            }
            bk = f32::max(f32::min(bk, self.bank_max), -self.bank_max);
            bank + (bk - bank) * 0.1
        }

        pub fn create_base_bank(&self, rand: &mut Rand) -> f32 {
            rand.gen_signed_f32(self.base_bank)
        }

        pub fn shape(&self) -> &ShipShape {
            &self.shape
        }

        pub fn damaged_shape(&self) -> &ShipShape {
            &self.damaged_shape
        }

        pub fn barrage(&mut self) -> &mut Barrage {
            &mut self.barrage
        }

        pub fn shield(&self) -> i32 {
            self.shield
        }

        pub fn score(&self) -> u32 {
            self.score
        }

        pub fn aim_ship(&self) -> bool {
            self.aim_ship
        }

        pub fn has_limit_y(&self) -> bool {
            self.has_limit_y
        }

        pub fn is_boss(&self) -> bool {
            self.is_boss
        }
    }

    pub struct BitSpec {
        pub bit_num: u32,
        bit_type: BitType,
        bit_distance: f32,
        bit_md: f32,
        pub bit_barrage: Barrage,
    }

    impl BitSpec {
        pub fn get_bit_offset(&self, idx: u32, cnt: u32) -> (Vector, f32) {
            match self.bit_type {
                BitType::Round => {
                    let od = std::f32::consts::PI * 2. / self.bit_num as f32;
                    let d = od * idx as f32 + cnt as f32 * self.bit_md;
                    (
                        Vector::new_at(
                            self.bit_distance * 2. * f32::sin(d),
                            self.bit_distance * 2. * f32::cos(d) * 5.,
                        ),
                        std::f32::consts::PI - f32::sin(d) * 0.05,
                    )
                }
                BitType::Line => {
                    let of = (idx as i32 % 2) * 2 - 1;
                    let oi = idx / 2 + 1;
                    (
                        Vector::new_at(self.bit_distance * 1.5 * oi as f32 * of as f32, 0.),
                        std::f32::consts::PI,
                    )
                }
            }
        }
    }
}
