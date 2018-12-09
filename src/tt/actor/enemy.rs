use crate::tt::actor::Pool;
use crate::tt::screen::Screen;
use crate::tt::shape::{Collidable, Drawable};
use crate::tt::ship::{self, Ship};
use crate::tt::state::in_game::ScoreAccumulator;
use crate::tt::tunnel::{self, Tunnel};
use crate::util::rand::Rand;
use crate::util::vector::Vector;

use crate::gl;

use self::ship_spec::ShipSpec;

use super::shot::Shot;

const OUT_OF_COURSE_BANK: f32 = 1.0;
const DISAP_DEPTH: f32 = -5.0;

#[derive(Default)]
pub struct Enemy {
    pos: Vector,
    ppos: Vector,
    flip_mv: Vector,
    flip_mv_cnt: u32,
    speed: f32,
    d1: f32,
    d2: f32,
    base_bank: f32,
    bank: f32,
    shield: i32,
    first_shield: i32,
    damaged: bool,
    high_order: bool,
    limit_y: f32,
    bit_cnt: u32,
}

impl Enemy {
    pub fn set(&mut self, spec: &ShipSpec, x: f32, y: f32, rand: &mut Rand) {
        self.set_ps_bank(spec, x, y, false, 0., rand)
    }

    fn set_ps_bank(
        &mut self,
        spec: &ShipSpec,
        x: f32,
        y: f32,
        passed: bool,
        base_bank: f32,
        rand: &mut Rand,
    ) {
        self.pos.x = x;
        self.limit_y = y;
        self.pos.y = y;
        self.speed = 0.;
        self.d1 = 0.;
        self.d2 = 0.;
        self.bank = 0.;
        self.shield = spec.shield();
        self.first_shield = self.shield;
        if !passed {
            self.base_bank = spec.create_base_bank(rand);
        } else {
            self.base_bank = base_bank;
        }
        self.flip_mv_cnt = 0;
        self.damaged = false;
        self.high_order = true;
        // TODO topBullet = null;
        // TODO bitBullet = null;
    }

    fn mov(
        &mut self,
        passed: bool,
        spec: &ShipSpec,
        tunnel: &Tunnel,
        ship: &mut Ship,
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
                f32::min((-OUT_OF_COURSE_BANK * co - self.bank) * 0.075, -1.),
                1.,
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
        /* TODO
        if (!passed && !topBullet) {
            Barrage tbb = spec.barrage;
            topBullet = tbb.addTopBullet(bullets, ship);
            for (int i = 0; i < spec.bitNum; i++) {
                Barrage bbb = spec.bitBarrage;
                BulletActor ba = bbb.addTopBullet(bullets, ship);
                if (ba) {
                    ba.unsetAimTop();
                    bitBullet ~= ba;
                }
            }
        }
        if (topBullet) {
            topBullet.bullet.pos.x = pos.x;
            topBullet.bullet.pos.y = pos.y;
            checkBulletInRange(topBullet);
            float d;
            int i = 0;
            if (bitBullet) {
                foreach (BulletActor bb; bitBullet) {
                    spec.getBitOffset(bitOffset, d, i, bitCnt);
                    bb.bullet.pos.x = bitOffset.x + pos.x;
                    bb.bullet.pos.y = bitOffset.y + pos.y;
                    bb.bullet.deg = d;
                    checkBulletInRange(bb);
                    i++;
                }
            }
        }
        if !passed && self.pos.y <= ship.in_sight_depth() {
            spec.shape().add_particles(pos, particles);
        }
        */
        let mut remove = false;
        if !passed {
            if (!spec.has_limit_y() && self.pos.y > ship::IN_SIGHT_DEPTH_DEFAULT * 5.)
                || self.pos.y < DISAP_DEPTH
            {
                /* TODO
                if (Ship.replayMode && pos.y < DISAP_DEPTH) {
                    Enemy en = passedEnemies.getInstance();
                    if (en)
                    en.set(spec, pos.x, pos.y, null, true, baseBank);
                }*/
                remove = true;
            }
        } else if self.pos.y < -ship::IN_SIGHT_DEPTH_DEFAULT * 3. {
            remove = true;
        }
        self.damaged = false;
        self.bit_cnt += 1;
        (remove, goto_next_zone)
    }

    pub fn check_shot_hit(
        &mut self,
        spec: &ShipSpec,
        p: Vector,
        shape: &Collidable,
        shot: &mut Shot,
        charge_shot: bool,
        tunnel: &Tunnel,
        ship: &mut Ship,
        score_accumulator: &mut ScoreAccumulator,
    ) -> (bool, bool) {
        let mut ox = f32::abs(self.pos.x - p.x);
        let oy = f32::abs(self.pos.y - p.y);
        if ox > std::f32::consts::PI {
            ox = std::f32::consts::PI * 2. - ox;
        }
        ox *= (tunnel.get_radius(self.pos.y) / tunnel::DEFAULT_RAD) * 3.;
        let mut remove_enemy = false;
        let mut remove_shot = false;
        if spec.shape().check_collision_shape(ox, oy, shape, 1.) {
            self.shield -= shot.damage();
            if self.shield <= 0 {
                self.destroyed(spec, ship);
                remove_enemy = true;
            } else {
                self.damaged = true;
                // TODO
            }
            remove_shot = shot.add_score(charge_shot, spec.score(), self.pos, score_accumulator);
        }
        (remove_enemy, remove_shot)
    }

    fn destroyed(&self, spec: &ShipSpec, ship: &mut Ship) {
        // TODO
        ship.rank_up(spec.is_boss());
        if self.first_shield == 1 {
            // TODO SoundManager.playSe("small_dest.wav");
        } else if self.first_shield < 20 {
            // TODO SoundManager.playSe("middle_dest.wav");
        } else {
            // TODO SoundManager.playSe("boss_dest.wav");
            ship.set_screen_shake(56, 0.064);
        }
    }

    pub fn draw(&self, spec: &ShipSpec, tunnel: &Tunnel) {
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
        /*TODO
        if (bitBullet) {
            foreach (BulletActor bb; bitBullet) {
                sp = tunnel.getPos(bb.bullet.pos);
                glPushMatrix();
                Screen.glTranslate(sp);
                glRotatef(bitCnt * 7, 0, 1, 0);
                glRotatef(pos.x * 180 / PI, 0, 0, 1);
                ShipSpec.bitShape.draw();
                glPopMatrix();
            }
        }*/
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
    small_ship_specs: Vec<ShipSpec>,
    medium_ship_specs: Vec<ShipSpec>,
    boss_ship_specs: Vec<ShipSpec>,
    pool: Pool<Enemy, EnemySpec>,
    boss_spec_idx: usize,
    rand: Rand,
}

enum EnemySpec {
    Small(usize),
    Medium(usize),
    Boss(usize),
}

impl EnemyPool {
    pub fn new(n: usize, seed: u64) -> Self {
        EnemyPool {
            small_ship_specs: Vec::new(),
            medium_ship_specs: Vec::new(),
            boss_ship_specs: Vec::new(),
            pool: Pool::new(n),
            boss_spec_idx: 0,
            rand: Rand::new(seed),
        }
    }

    pub fn renew_ship_specs(
        &mut self,
        level: f32,
        grade: u32,
        medium_boss_zone: bool,
        boss_num: u32,
        screen: &Screen,
    ) {
        for _ in 0..(2 + self.rand.gen_usize(2)) {
            let ss = ShipSpec::new_small(&mut self.rand, level * 1.8, grade, screen);
            self.small_ship_specs.push(ss);
        }
        for _ in 0..(2 + self.rand.gen_usize(2)) {
            let ss = ShipSpec::new_medium(&mut self.rand, level * 1.9, screen);
            self.medium_ship_specs.push(ss);
        }
        for _ in 0..boss_num {
            let mut lv = level * 2.0 / boss_num as f32;
            if medium_boss_zone {
                lv *= 1.33;
            }
            let boss_speed = 0.8 + grade as f32 * 0.04 + self.rand.gen_f32(0.03);
            let ss = ShipSpec::new_boss(&mut self.rand, lv, boss_speed, medium_boss_zone, screen);
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
        if let Some((pa, pa_ref)) = inst {
            pa.prepare(pa_ref, EnemySpec::Small(idx));
            op(&mut pa.actor, spec);
        }
    }

    pub fn get_medium_instance_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Enemy, &ShipSpec),
    {
        let idx = self.rand.gen_usize(self.medium_ship_specs.len());
        let spec = &self.medium_ship_specs[idx];
        let inst = self.pool.get_instance();
        if let Some((pa, pa_ref)) = inst {
            pa.prepare(pa_ref, EnemySpec::Medium(idx));
            op(&mut pa.actor, spec);
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
        if let Some((pa, pa_ref)) = inst {
            pa.prepare(pa_ref, EnemySpec::Boss(idx));
            op(&mut pa.actor, spec);
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rand.set_seed(seed);
    }

    pub fn mov(&mut self, tunnel: &Tunnel, ship: &mut Ship) -> bool {
        let mut goto_next_zone = false;
        for pa in &mut self.pool {
            let spec = match pa.state.spec() {
                EnemySpec::Small(idx) => &self.small_ship_specs[*idx],
                EnemySpec::Medium(idx) => &self.medium_ship_specs[*idx],
                EnemySpec::Boss(idx) => &self.boss_ship_specs[*idx],
            };
            let (release, goto_nz) = pa.actor.mov(false, spec, tunnel, ship);
            if goto_nz {
                goto_next_zone = true;
            }
            if release {
                pa.release();
            }
        }
        goto_next_zone
    }

    pub fn check_shot_hit(
        &mut self,
        p: Vector,
        shape: &Collidable,
        shot: &mut Shot,
        charge_shot: bool,
        tunnel: &Tunnel,
        ship: &mut Ship,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        let mut release_shot = false;
        for pa in &mut self.pool {
            let spec = match pa.state.spec() {
                EnemySpec::Small(idx) => &self.small_ship_specs[*idx],
                EnemySpec::Medium(idx) => &self.medium_ship_specs[*idx],
                EnemySpec::Boss(idx) => &self.boss_ship_specs[*idx],
            };
            let (release_enemy, rel_shot) = pa.actor.check_shot_hit(
                spec,
                p,
                shape,
                shot,
                charge_shot,
                tunnel,
                ship,
                score_accumulator,
            );
            if rel_shot {
                release_shot = true;
            }
            if release_enemy {
                pa.release();
            }
        }
        release_shot
    }

    pub fn draw(&self, tunnel: &Tunnel) {
        for pa in &self.pool {
            match pa.state.spec() {
                EnemySpec::Small(idx) => pa.actor.draw(&self.small_ship_specs[*idx], tunnel),
                EnemySpec::Medium(idx) => pa.actor.draw(&self.medium_ship_specs[*idx], tunnel),
                EnemySpec::Boss(idx) => pa.actor.draw(&self.boss_ship_specs[*idx], tunnel),
            }
        }
    }

    pub fn get_num(&self) -> usize {
        self.pool.get_num()
    }
}

pub mod ship_spec {
    use crate::util::rand::Rand;
    use crate::util::vector::Vector;

    use crate::tt::screen::Screen;
    use crate::tt::shape::ship_shape::ShipShape;
    use crate::tt::ship;
    use crate::tt::tunnel::Tunnel;

    const SPEED_CHANGE_RATIO: f32 = 0.2;

    pub struct ShipSpec {
        shape: ShipShape,
        damaged_shape: ShipShape,
        shield: i32,
        base_speed: f32,
        ship_speed_ratio: f32,
        visual_range: f32,
        base_bank: f32,
        bank_max: f32,
        score: u32,
        bit_num: u32,
        aim_ship: bool,
        has_limit_y: bool,
        no_fire_depth_limit: bool,
        is_boss: bool,
    }

    impl ShipSpec {
        pub fn new_small(rand: &mut Rand, level: f32, grade: u32, screen: &Screen) -> Self {
            let rs = rand.gen_usize(99999) as u64;
            let bi_min = usize::max(usize::min((160.0 / level) as usize, 40), 80)
                + (ship::GRADE_NUM - 1 - grade as usize) * 8;
            let brg_interval =
                bi_min + rand.gen_usize(80 + (ship::GRADE_NUM - 1 - grade as usize) * 8 - bi_min);
            let brg_rank = level / (150.0 / brg_interval as f32);
            Self {
                shape: ShipShape::new_small(false, screen, rs),
                damaged_shape: ShipShape::new_small(true, screen, rs),
                shield: 1,
                base_speed: 0.05 + rand.gen_f32(0.1),
                ship_speed_ratio: 0.25 + rand.gen_f32(0.25),
                visual_range: 10. + rand.gen_f32(32.),
                base_bank: if rand.gen_usize(3) == 0 {
                    0.1 + rand.gen_f32(0.2)
                } else {
                    0.
                },
                bank_max: 0.3 + rand.gen_f32(0.7),
                score: 100,
                bit_num: 0,
                aim_ship: false,
                has_limit_y: false,
                no_fire_depth_limit: false,
                is_boss: false,
            }
            // TODO _barrage = createBarrage(rand, brgRank, 0, brgInterval);
        }

        pub fn new_medium(rand: &mut Rand, level: f32, screen: &Screen) -> Self {
            let rs = rand.gen_usize(99999) as u64;
            Self {
                shape: ShipShape::new_medium(false, screen, rs),
                damaged_shape: ShipShape::new_medium(true, screen, rs),
                shield: 10,
                base_speed: 0.1 + rand.gen_f32(0.1),
                ship_speed_ratio: 0.4 + rand.gen_f32(0.4),
                visual_range: 10. + rand.gen_f32(32.),
                base_bank: if rand.gen_usize(4) == 0 {
                    0.05 + rand.gen_f32(0.1)
                } else {
                    0.
                },
                bank_max: 0.2 + rand.gen_f32(0.5),
                score: 500,
                bit_num: 0,
                aim_ship: false,
                has_limit_y: false,
                no_fire_depth_limit: false,
                is_boss: false,
            }
            // TODO _barrage = createBarrage(rand, level, 0, 0, 1, "middle", BulletShape.BSType.SQUARE);
        }

        pub fn new_boss(
            rand: &mut Rand,
            level: f32,
            speed: f32,
            medium_boss: bool,
            screen: &Screen,
        ) -> Self {
            let rs = rand.gen_usize(99999) as u64;
            let mut spec = Self {
                shape: ShipShape::new_large(false, screen, rs),
                damaged_shape: ShipShape::new_large(true, screen, rs),
                shield: 30,
                base_speed: 0.1 + rand.gen_f32(0.1),
                ship_speed_ratio: speed,
                visual_range: 16. + rand.gen_f32(24.),
                base_bank: 0.,
                bank_max: 0.8 + rand.gen_f32(0.4),
                score: 2000,
                bit_num: 0,
                aim_ship: true,
                has_limit_y: true,
                no_fire_depth_limit: true,
                is_boss: true,
            };
            // TODO _barrage = createBarrage(rand, level, 0, 0, 1.2f, "middle", BulletShape.BSType.SQUARE, true);
            if !medium_boss {
                spec.bit_num = 2 + rand.gen_usize(3) as u32 * 2;
            }
            /* TODO
            bitType = rand.nextInt(2);
    bitDistance = 0.33 + rand.nextFloat(0.3);
    bitMd = 0.02 + rand.nextFloat(0.02);
    float bitBrgRank = level;
    bitBrgRank /= (bitNum / 2);
    int brgInterval;
    int biMin = cast(int)(120.0f / bitBrgRank);
    if (biMin > 60)
      biMin = 60;
    else if (biMin < 20)
      biMin = 20;
    brgInterval = biMin + rand.nextInt(60 - biMin);
    bitBrgRank /= (60.0f / brgInterval);
    _bitBarrage = createBarrage(rand, bitBrgRank, 0, brgInterval, 1, null,
                                BulletShape.BSType.BAR, true);
    _bitBarrage.setNoXReverse();
            */
            spec
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
            bk = f32::max(f32::min(bk, -self.bank_max), self.bank_max);
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
}
