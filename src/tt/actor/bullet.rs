use std::ops::{Index, IndexMut};
use std::rc::Rc;

use bulletml::{AppRunner, BulletML, Runner, RunnerData, State};

use crate::gl;

use crate::util::rand::Rand;
use crate::util::vector::Vector;

use crate::tt::actor::shot::Shot;
use crate::tt::actor::{Pool, PoolActorRef};
use crate::tt::shape::{Collidable, Drawable};
use crate::tt::ship::Ship;
use crate::tt::state::in_game::ScoreAccumulator;
use crate::tt::tunnel::{self, Tunnel};

#[derive(Default)]
pub struct Bullet {
    pub options: Option<BulletOptions>,
    pub root_rank: f32,
    ppos: Vector,
    cnt: u32,
    is_simple: bool,
    is_top: bool,
    is_aim_top: bool,
    is_visible: bool,
    should_be_released: bool,
    is_wait: bool,
    post_wait: u32,
    wait_cnt: u32,
    is_morph_seed: bool,
    disap_cnt: u32,
    bml_idx: usize,
}

pub struct BulletOptions {
    bml_params: Rc<Vec<BMLParam>>,
    runner: Runner<TTRunner>,
    pub bullet: BulletImpl,
}

const DISAP_CNT: u32 = 45;

impl Bullet {
    fn set(&mut self, pos: Vector, deg: f32, speed: f32) {
        self.options.as_mut().unwrap().bullet.set(pos, deg, speed);
        self.is_simple = false;
        self.start();
    }

    fn set_simple(&mut self, pos: Vector, deg: f32, speed: f32) {
        self.options.as_mut().unwrap().bullet.set(pos, deg, speed);
        self.is_simple = true;
        self.start();
    }

    fn start(&mut self) {
        self.is_top = false;
        self.is_aim_top = false;
        self.is_wait = false;
        self.is_visible = true;
        self.is_morph_seed = false;
        self.ppos = self.options.as_ref().unwrap().bullet.pos;
        self.cnt = 0;
        self.root_rank = 1.;
        self.should_be_released = false;
        self.disap_cnt = 0;
    }

    fn set_invisible(&mut self) {
        self.is_visible = false;
    }

    fn set_top(&mut self) {
        self.is_top = true;
        self.is_aim_top = true;
        self.set_invisible();
    }

    pub fn unset_aim_top(&mut self) {
        self.is_aim_top = false;
    }

    fn set_wait(&mut self, prev_wait: u32, post_wait: u32) {
        self.is_wait = true;
        self.wait_cnt = prev_wait;
        self.post_wait = post_wait;
    }

    fn set_morph_seed(&mut self) {
        self.is_morph_seed = true;
    }

    fn start_disappear(&mut self) {
        if self.is_visible && self.disap_cnt <= 0 {
            self.disap_cnt = 1;
        }
    }

    fn mov(
        &mut self,
        manager: &mut BulletsManager,
        tunnel: &Tunnel,
        ship: &Ship,
        rand: &mut Rand,
    ) -> bool {
        let mut release = false;
        let mut start_disappear = false;
        {
            let options = self.options.as_mut().unwrap();
            let bullet = &mut options.bullet;
            let bml_params = &options.bml_params;
            let tpos = ship.get_target_pos();
            self.ppos = bullet.pos;
            if self.is_aim_top {
                let mut ox = tpos.x - bullet.pos.x;
                if ox > std::f32::consts::PI {
                    ox -= std::f32::consts::PI * 2.;
                } else if ox < -std::f32::consts::PI {
                    ox += std::f32::consts::PI * 2.;
                }
                bullet.deg = (f32::atan2(ox, tpos.y - bullet.pos.y) * bullet.x_reverse
                    + std::f32::consts::PI / 2.)
                    * bullet.y_reverse
                    - std::f32::consts::PI / 2.;
            }
            if self.is_wait && self.wait_cnt > 0 {
                self.wait_cnt -= 1;
                return self.should_be_released;
            }
            if !self.is_simple {
                let bml_param = &bml_params[self.bml_idx];
                let runner = &mut options.runner;
                bullet.mov(runner, manager, bml_param, tpos, rand);
                if manager.bullet_should_be_released {
                    self.should_be_released = true;
                }
                if runner.is_end() {
                    if self.is_top {
                        self.bml_idx = 0;
                        let bml_param = &bml_params[self.bml_idx];
                        *runner = Runner::new(TTRunner::new(), &bml_param.bml);
                        if self.is_wait {
                            self.wait_cnt = self.post_wait;
                            return false;
                        }
                    } else if self.is_morph_seed {
                        return true;
                    }
                }
            }
            if self.should_be_released {
                return true;
            }
            let speed_rank = bullet.get_speed_rank(bml_params, self.bml_idx);
            let mx = (f32::sin(bullet.deg) * bullet.speed + bullet.acc.x)
                * speed_rank
                * bullet.x_reverse;
            let my = (f32::cos(bullet.deg) * bullet.speed - bullet.acc.y)
                * speed_rank
                * bullet.y_reverse;
            let d = f32::atan2(mx, my);
            let r = (1. - f32::abs(f32::sin(d)) * 0.999) * (ship.speed() * 5.);
            bullet.pos.x += mx * r;
            bullet.pos.y += my * r;

            if bullet.pos.x >= std::f32::consts::PI * 2. {
                bullet.pos.x -= std::f32::consts::PI * 2.;
            } else if bullet.pos.x < 0. {
                bullet.pos.x += std::f32::consts::PI * 2.;
            }
            if self.is_visible && self.disap_cnt <= 0 {
                if ship.check_bullet_hit(bullet.pos, self.ppos) {
                    release = true;
                }
                if bullet.pos.y < -2.
                    || (!bullet.long_range && bullet.pos.y > ship.in_sight_depth())
                    || !tunnel.check_in_screen(bullet.pos, ship)
                {
                    start_disappear = true;
                }
            }
            self.cnt += 1;
            if self.disap_cnt > 0 {
                self.disap_cnt += 1;
                if self.disap_cnt > DISAP_CNT {
                    release = true;
                }
            } else {
                if self.cnt > 600 {
                    start_disappear = true;
                }
            }
        }
        if start_disappear {
            self.start_disappear();
        }
        release
    }

    fn check_shot_hit(
        &mut self,
        shot: &mut Shot,
        tunnel: &Tunnel,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        if !self.is_visible || self.disap_cnt > 0 {
            return false;
        }
        let bullet_pos = self.options.as_ref().unwrap().bullet.pos;
        let mut ox = f32::abs(bullet_pos.x - shot.pos.x);
        let oy = f32::abs(bullet_pos.y - shot.pos.y);
        if ox > std::f32::consts::PI {
            ox = std::f32::consts::PI * 2. - ox;
        }
        ox *= (tunnel.get_radius(bullet_pos.y) / tunnel::DEFAULT_RAD) * 3.;
        let mut release_shot = false;
        if shot.shape.as_ref().unwrap().check_collision(ox, oy) {
            self.start_disappear();
            release_shot = shot.add_score(10, bullet_pos, score_accumulator);
        }
        release_shot
    }

    fn draw(&self, tunnel: &Tunnel) {
        if self.is_visible {
            let bullet = &self.options.as_ref().unwrap().bullet;
            let d = (bullet.deg * bullet.x_reverse + std::f32::consts::PI / 2.) * bullet.y_reverse
                - std::f32::consts::PI / 2.;
            let sp = tunnel.get_pos_v(bullet.pos);
            unsafe {
                gl::PushMatrix();
                gl::Translatef(sp.x, sp.y, sp.z);
                gl::Rotatef(d * 180. / std::f32::consts::PI, 0., 1., 0.);
                gl::Rotatef((self.cnt * 6) as f32, 0., 0., 1.);
            }
            if self.disap_cnt <= 0 {
                bullet.shape.draw();
            } else {
                let s = 1. - self.disap_cnt as f32 / DISAP_CNT as f32;
                unsafe {
                    gl::Scalef(s, s, s);
                }
                bullet.disap_shape.draw();
            }
            unsafe {
                gl::PopMatrix();
            }
        }
    }
}

pub struct BulletPool {
    pool: Pool<Bullet>,
    cnt: u32,
    bullet_rand: Rand,
}

impl BulletPool {
    pub fn new(n: usize) -> Self {
        BulletPool {
            pool: Pool::new(n),
            cnt: 0,
            bullet_rand: Rand::new(Rand::rand_seed()),
        }
    }

    fn add_bullet(&mut self, src_bullet: &Bullet, deg: f32, speed: f32) -> Option<PoolActorRef> {
        let src_bullet_options = src_bullet.options.as_ref().unwrap();
        if let Some(rb) = src_bullet_options.bullet.root_bullet {
            let rb = &self.pool[rb];
            if rb.root_rank <= 0. {
                return None;
            }
        }
        let bml_idx = src_bullet.bml_idx;
        let inst = self.pool.get_instance();
        if let Some((bullet, bullet_ref)) = inst {
            let bml_params = &src_bullet_options.bml_params;
            let (goto_next_parser, bml_idx) = {
                if bml_idx + 1 >= bml_params.len() {
                    (false, bml_idx)
                } else {
                    (true, bml_idx + 1)
                }
            };
            bullet.bml_idx = bml_idx;
            let bml_param = &bml_params[bml_idx];
            let runner = Runner::new(TTRunner::new(), &bml_param.bml);
            let src_bi = &src_bullet_options.bullet;
            let mut options = BulletOptions {
                bml_params: bml_params.clone(),
                runner,
                bullet: BulletImpl::new_param(
                    &src_bi.shape,
                    &src_bi.disap_shape,
                    src_bi.x_reverse,
                    src_bi.y_reverse,
                    src_bi.long_range,
                ),
            };
            bullet.options = Some(options);
            if goto_next_parser {
                bullet.set(src_bi.pos, deg, speed);
                bullet.set_morph_seed();
            } else {
                bullet.set_simple(src_bi.pos, deg, speed);
            }
            Some(bullet_ref)
        } else {
            None
        }
    }

    fn add_bullet_state(
        &mut self,
        src_bullet: &Bullet,
        state: State,
        deg: f32,
        speed: f32,
    ) -> Option<PoolActorRef> {
        let src_bullet_options = src_bullet.options.as_ref().unwrap();
        if let Some(rb) = src_bullet_options.bullet.root_bullet {
            let rb = &self.pool[rb];
            if rb.root_rank <= 0. {
                return None;
            }
        }
        let bml_idx = src_bullet.bml_idx;
        let inst = self.pool.get_instance();
        if let Some((bullet, bullet_ref)) = inst {
            bullet.bml_idx = bml_idx;
            let bml_params = &src_bullet_options.bml_params;
            let runner = Runner::new_from_state(TTRunner::new(), state);
            let src_bi = &src_bullet_options.bullet;
            let options = BulletOptions {
                bml_params: bml_params.clone(),
                runner,
                bullet: BulletImpl::new_param(
                    &src_bi.shape,
                    &src_bi.disap_shape,
                    src_bi.x_reverse,
                    src_bi.y_reverse,
                    src_bi.long_range,
                ),
            };
            bullet.options = Some(options);
            bullet.set(src_bi.pos, deg, speed);
            Some(bullet_ref)
        } else {
            None
        }
    }

    pub fn add_top_bullet(
        &mut self,
        bml_params: &Rc<Vec<BMLParam>>,
        pos: Vector,
        deg: f32,
        speed: f32,
        shape: &Rc<Drawable>,
        disap_shape: &Rc<Drawable>,
        x_reverse: f32,
        y_reverse: f32,
        long_range: bool,
        prev_wait: u32,
        post_wait: u32,
    ) -> Option<PoolActorRef> {
        let inst = self.pool.get_instance();
        if let Some((bullet, bullet_ref)) = inst {
            bullet.bml_idx = 0;
            let bml_param = &bml_params[0];
            let runner = Runner::new(TTRunner::new(), &bml_param.bml);
            let options = BulletOptions {
                bml_params: bml_params.clone(),
                runner,
                bullet: BulletImpl::new_param_first(
                    shape,
                    disap_shape,
                    x_reverse,
                    y_reverse,
                    long_range,
                    Some(bullet_ref),
                ),
            };
            bullet.options = Some(options);
            bullet.set(pos, deg, speed);
            bullet.set_wait(prev_wait, post_wait);
            bullet.set_top();
            Some(bullet_ref)
        } else {
            None
        }
    }

    pub fn mov(&mut self, tunnel: &Tunnel, ship: &Ship) {
        let invariant_pool = unsafe { &mut *(self as *mut BulletPool) };
        // XXX acting_refs resolves currently acting bullets so
        // invariant_self is safe and so is invariant_bullet.
        for bullet_ref in self.pool.as_refs() {
            let release = {
                let mut bullet = &mut self.pool[bullet_ref];
                let invariant_bullet = unsafe { &mut *(bullet as *mut Bullet) };
                let mut manager = BulletsManager::new(invariant_pool, invariant_bullet);
                bullet.mov(&mut manager, tunnel, ship, &mut self.bullet_rand)
            };
            if release {
                self.pool.release(bullet_ref);
            }
        }
        self.cnt += 1;
    }

    pub fn draw(&self, tunnel: &Tunnel) {
        for bullet in &self.pool {
            bullet.draw(tunnel);
        }
    }

    pub fn release(&mut self, bullet_ref: PoolActorRef) {
        {
            let bullet = &mut self.pool[bullet_ref];
            bullet.options = None;
        }
        self.pool.release(bullet_ref);
    }

    pub fn clear(&mut self) {
        self.pool.clear();
        self.cnt = 0;
    }

    fn get_turn(&self) -> u32 {
        self.cnt
    }

    pub fn clear_visible(&mut self) {
        for bullet in &mut self.pool {
            bullet.start_disappear();
        }
    }

    pub fn check_shot_hit(
        &mut self,
        shot: &mut Shot,
        tunnel: &Tunnel,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        let mut release_shot = false;
        for bullet in &mut self.pool {
            let rel_shot = bullet.check_shot_hit(shot, tunnel, score_accumulator);
            if rel_shot {
                release_shot = true;
            }
        }
        release_shot
    }

    pub fn maybe_index_mut(&mut self, index: PoolActorRef) -> Option<&mut Bullet> {
        self.pool.maybe_index_mut(index)
    }
}

impl Index<PoolActorRef> for BulletPool {
    type Output = Bullet;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<PoolActorRef> for BulletPool {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        &mut self.pool[index]
    }
}

struct BulletsManager<'a> {
    bullets: &'a mut BulletPool,
    bullet: &'a Bullet,
    bullet_should_be_released: bool,
}

impl<'a> BulletsManager<'a> {
    fn new(bullets: &'a mut BulletPool, bullet: &'a mut Bullet) -> Self {
        BulletsManager {
            bullets,
            bullet,
            bullet_should_be_released: false,
        }
    }

    fn add_bullet(&mut self, deg: f32, speed: f32) {
        self.bullets.add_bullet(self.bullet, deg, speed);
    }

    fn add_bullet_state(&mut self, state: State, deg: f32, speed: f32) {
        self.bullets
            .add_bullet_state(self.bullet, state, deg, speed);
    }

    fn get_turn(&self) -> u32 {
        self.bullets.get_turn()
    }

    fn kill(&mut self) {
        self.bullet_should_be_released = true;
    }
}

pub struct BulletImpl {
    pub pos: Vector,
    acc: Vector,
    pub deg: f32,
    speed: f32,
    shape: Rc<Drawable>,
    disap_shape: Rc<Drawable>,
    x_reverse: f32,
    y_reverse: f32,
    long_range: bool,
    root_bullet: Option<PoolActorRef>,
}

impl BulletImpl {
    fn new_param_first(
        shape: &Rc<Drawable>,
        disap_shape: &Rc<Drawable>,
        x_reverse: f32,
        y_reverse: f32,
        long_range: bool,
        root_bullet: Option<PoolActorRef>,
    ) -> Self {
        Self {
            pos: Vector::default(),
            acc: Vector::default(),
            deg: 0.,
            speed: 0.,
            shape: shape.clone(),
            disap_shape: disap_shape.clone(),
            x_reverse,
            y_reverse,
            long_range,
            root_bullet,
        }
    }

    fn new_param(
        shape: &Rc<Drawable>,
        disap_shape: &Rc<Drawable>,
        x_reverse: f32,
        y_reverse: f32,
        long_range: bool,
    ) -> Self {
        BulletImpl::new_param_first(shape, disap_shape, x_reverse, y_reverse, long_range, None)
    }

    fn set(&mut self, pos: Vector, deg: f32, speed: f32) {
        self.pos = pos;
        self.acc = Vector::default();
        self.deg = deg;
        self.speed = speed;
    }

    fn mov(
        &mut self,
        runner: &mut Runner<TTRunner>,
        manager: &mut BulletsManager,
        bml_param: &BMLParam,
        target: Vector,
        rand: &mut Rand,
    ) {
        if !runner.is_end() {
            runner.run(&mut RunnerData {
                bml: &bml_param.bml,
                data: &mut TTRunnerData {
                    manager,
                    rand,
                    bullet: self,
                    bml_param,
                    target,
                },
            });
        }
    }

    fn get_speed_rank(&self, bml_params: &Rc<Vec<BMLParam>>, bml_idx: usize) -> f32 {
        bml_params[bml_idx].speed
    }
}

struct TTRunner {}

impl TTRunner {
    fn new() -> Self {
        TTRunner {}
    }
}

struct TTRunnerData<'a, 'm>
where
    'm: 'a,
{
    manager: &'a mut BulletsManager<'m>,
    rand: &'a mut Rand,
    bullet: &'a mut BulletImpl,
    bml_param: &'a BMLParam,
    target: Vector,
}

const VEL_SS_SDM_RATIO: f32 = 62. / 10.;
const VEL_SDM_SS_RATIO: f32 = 10. / 62.;

impl<'a, 'm> AppRunner<TTRunnerData<'a, 'm>> for TTRunner {
    fn get_bullet_direction(&self, data: &TTRunnerData) -> f64 {
        rtod(data.bullet.deg) as f64
    }

    fn get_aim_direction(&self, data: &TTRunnerData) -> f64 {
        let b = data.bullet.pos;
        let t = data.target;
        rtod(f32::atan2(t.x - b.x, t.y - b.y)) as f64
    }

    fn get_bullet_speed(&self, data: &TTRunnerData) -> f64 {
        (data.bullet.speed * VEL_SS_SDM_RATIO) as f64
    }

    fn get_default_speed(&self) -> f64 {
        1.
    }

    fn get_rank(&self, data: &TTRunnerData) -> f64 {
        f32::min(data.bml_param.rank, 1.) as f64
    }

    fn create_simple_bullet(&mut self, data: &mut TTRunnerData, direction: f64, speed: f64) {
        data.manager
            .add_bullet(dtor(direction as f32), speed as f32 * VEL_SDM_SS_RATIO);
    }

    fn create_bullet(&mut self, data: &mut TTRunnerData, state: State, direction: f64, speed: f64) {
        data.manager.add_bullet_state(
            state,
            dtor(direction as f32),
            speed as f32 * VEL_SDM_SS_RATIO,
        );
    }

    fn get_turn(&self, data: &TTRunnerData) -> u32 {
        data.manager.get_turn()
    }

    fn do_vanish(&self, data: &mut TTRunnerData) {
        data.manager.kill();
    }

    fn do_change_direction(&self, data: &mut TTRunnerData, direction: f64) {
        data.bullet.deg = dtor(direction as f32);
    }

    fn do_change_speed(&self, data: &mut TTRunnerData, speed: f64) {
        data.bullet.speed = speed as f32 * VEL_SDM_SS_RATIO;
    }

    fn do_accel_x(&self, _: f64) {
        // TODO
        panic!("do_accel_x");
    }

    fn do_accel_y(&self, _: f64) {
        // TODO
        panic!("do_accel_y");
    }

    fn get_bullet_speed_x(&self) -> f64 {
        // TODO
        panic!("get_bullet_speed_x");
    }
    fn get_bullet_speed_y(&self) -> f64 {
        // TODO
        panic!("get_bullet_speed_y");
    }

    fn get_rand(&self, data: &mut TTRunnerData) -> f64 {
        data.rand.gen_f32(1.) as f64
    }
}

pub struct BMLParam {
    pub bml: Rc<BulletML>,
    pub rank: f32,
    pub root_rank_effect: f32,
    pub speed: f32,
}

fn rtod(a: f32) -> f32 {
    a * 180. / std::f32::consts::PI
}

fn dtor(a: f32) -> f32 {
    a * std::f32::consts::PI / 180.
}
