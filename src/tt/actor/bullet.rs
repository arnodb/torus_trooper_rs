use std::ops::{Index, IndexMut};
use std::rc::Rc;

use bulletml::{AppRunner, BulletML, Runner, RunnerData, State};

use crate::gl;

use crate::util::rand::Rand;
use crate::util::vector::Vector;

use crate::tt::actor::float_letter::FloatLetterPool;
use crate::tt::actor::particle::ParticlePool;
use crate::tt::actor::shot::{Shot, ShotPool};
use crate::tt::actor::{Pool, PoolActorRef};
use crate::tt::shape::{Collidable, Drawable};
use crate::tt::ship::Ship;
use crate::tt::tunnel::{self, Tunnel};
use crate::tt::GeneralParams;

#[derive(Default)]
pub struct Bullet {
    bml_params: Rc<Vec<BMLParam>>,
    bml_idx: usize,
    runner: Runner<TTRunner>,
    pub bullet: Option<BulletImpl>,
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
}

const DISAP_CNT: u32 = 45;

impl Bullet {
    fn set(&mut self, pos: Vector, deg: f32, speed: f32) {
        self.bullet.as_mut().unwrap().set(pos, deg, speed);
        self.is_simple = false;
        self.start();
    }

    fn set_simple(&mut self, pos: Vector, deg: f32, speed: f32) {
        self.bullet.as_mut().unwrap().set(pos, deg, speed);
        self.is_simple = true;
        self.start();
    }

    fn start(&mut self) {
        self.is_top = false;
        self.is_aim_top = false;
        self.is_wait = false;
        self.is_visible = true;
        self.is_morph_seed = false;
        self.ppos = self.bullet.as_ref().unwrap().pos;
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
        params: &GeneralParams,
        ship: &mut Ship,
        shots: &mut ShotPool,
        particles: &mut ParticlePool,
        rand: &mut Rand,
    ) -> (bool, bool) {
        let mut release = false;
        let mut destroy = false;
        let mut start_disappear = false;
        {
            let bullet = self.bullet.as_mut().unwrap();
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
                return (self.should_be_released, false);
            }
            if !self.is_simple {
                let bml_param = &self.bml_params[self.bml_idx];
                bullet.mov(&mut self.runner, manager, bml_param, tpos, rand);
                if manager.bullet_should_be_released {
                    self.should_be_released = true;
                }
                if self.runner.is_end() {
                    if self.is_top {
                        self.bml_idx = 0;
                        let bml_param = &self.bml_params[self.bml_idx];
                        self.runner.init(&bml_param.bml);
                        if self.is_wait {
                            self.wait_cnt = self.post_wait;
                            return (false, false);
                        }
                    } else if self.is_morph_seed {
                        return (true, false);
                    }
                }
            }
            if self.should_be_released {
                return (true, false);
            }
            let speed_rank = self.bml_params[self.bml_idx].speed;
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
                if ship.check_bullet_hit(bullet.pos, self.ppos, params, shots, particles) {
                    release = true;
                    destroy = true;
                }
                if bullet.pos.y < -2.
                    || (!bullet.long_range && bullet.pos.y > ship.in_sight_depth())
                    || !params.tunnel.check_in_screen(bullet.pos, ship)
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
        (release, destroy)
    }

    fn check_shot_hit(
        &mut self,
        shot: &mut Shot,
        params: &mut GeneralParams,
        game_over: bool,
        float_letters: &mut FloatLetterPool,
    ) -> bool {
        if !self.is_visible || self.disap_cnt > 0 {
            return false;
        }
        let bullet_pos = self.bullet.as_ref().unwrap().pos;
        let mut ox = f32::abs(bullet_pos.x - shot.pos.x);
        let oy = f32::abs(bullet_pos.y - shot.pos.y);
        if ox > std::f32::consts::PI {
            ox = std::f32::consts::PI * 2. - ox;
        }
        ox *= (params.tunnel.get_radius(bullet_pos.y) / tunnel::DEFAULT_RAD) * 3.;
        if shot.shape.as_ref().unwrap().check_collision(ox, oy) {
            self.start_disappear();
            shot.add_score(10, bullet_pos, params, game_over, float_letters)
        } else {
            false
        }
    }

    fn draw(&self, tunnel: &Tunnel) {
        if self.is_visible {
            let bullet = self.bullet.as_ref().unwrap();
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
    barrage_rand: Rand,
    bullet_rand: Rand,
}

impl BulletPool {
    pub fn new(n: usize, seed: u64) -> Self {
        BulletPool {
            pool: Pool::new(n),
            cnt: 0,
            barrage_rand: Rand::new(seed),
            bullet_rand: Rand::new(seed),
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.barrage_rand.set_seed(seed);
        self.bullet_rand.set_seed(seed);
    }

    fn add_bullet(&mut self, src_bullet: &Bullet, deg: f32, speed: f32) -> Option<PoolActorRef> {
        let src_bullet_impl = src_bullet.bullet.as_ref().unwrap();
        if let Some(rb) = src_bullet_impl.root_bullet {
            let rb = &self.pool[rb];
            if rb.root_rank <= 0. {
                return None;
            }
        }
        let bml_idx = src_bullet.bml_idx;
        let inst = self.pool.get_instance();
        if let Some((bullet, bullet_ref)) = inst {
            let bml_params = &src_bullet.bml_params;
            let (goto_next_parser, bml_idx) = {
                if bml_idx + 1 >= bml_params.len() {
                    (false, bml_idx)
                } else {
                    (true, bml_idx + 1)
                }
            };
            bullet.bml_params = bml_params.clone();
            bullet.bml_idx = bml_idx;
            let bml_param = &bml_params[bml_idx];
            bullet.runner.init(&bml_param.bml);
            bullet.bullet = Some(BulletImpl::new_param(
                &src_bullet_impl.shape,
                &src_bullet_impl.disap_shape,
                src_bullet_impl.x_reverse,
                src_bullet_impl.y_reverse,
                src_bullet_impl.long_range,
            ));
            if goto_next_parser {
                bullet.set(src_bullet_impl.pos, deg, speed);
                bullet.set_morph_seed();
            } else {
                bullet.set_simple(src_bullet_impl.pos, deg, speed);
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
        let src_bullet_impl = src_bullet.bullet.as_ref().unwrap();
        if let Some(rb) = src_bullet_impl.root_bullet {
            let rb = &self.pool[rb];
            if rb.root_rank <= 0. {
                return None;
            }
        }
        let bml_idx = src_bullet.bml_idx;
        let inst = self.pool.get_instance();
        if let Some((bullet, bullet_ref)) = inst {
            let bml_params = &src_bullet.bml_params;
            bullet.bml_params = bml_params.clone();
            bullet.bml_idx = bml_idx;
            bullet.runner.init_from_state(state);
            bullet.bullet = Some(BulletImpl::new_param(
                &src_bullet_impl.shape,
                &src_bullet_impl.disap_shape,
                src_bullet_impl.x_reverse,
                src_bullet_impl.y_reverse,
                src_bullet_impl.long_range,
            ));
            bullet.set(src_bullet_impl.pos, deg, speed);
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
            bullet.bml_params = bml_params.clone();
            bullet.bml_idx = 0;
            let bml_param = &bml_params[0];
            bullet.runner.init(&bml_param.bml);
            bullet.bullet = Some(BulletImpl::new_param_first(
                shape,
                disap_shape,
                x_reverse,
                y_reverse,
                long_range,
                Some(bullet_ref),
            ));
            bullet.set(pos, deg, speed);
            bullet.set_wait(prev_wait, post_wait);
            bullet.set_top();
            Some(bullet_ref)
        } else {
            None
        }
    }

    pub fn mov(
        &mut self,
        params: &mut GeneralParams,
        ship: &mut Ship,
        shots: &mut ShotPool,
        particles: &mut ParticlePool,
    ) {
        let mut ship_destroyed = false;
        let invariant_pool = unsafe { &mut *(self as *mut BulletPool) };
        // XXX acting_refs resolves currently acting bullets so
        // invariant_self is safe and so is invariant_bullet.
        for bullet_ref in self.pool.as_refs() {
            let (release, destroy) = {
                let bullet = &mut self.pool[bullet_ref];
                let invariant_bullet = unsafe { &mut *(bullet as *mut Bullet) };
                let mut manager = BulletsManager::new(invariant_pool, invariant_bullet);
                bullet.mov(
                    &mut manager,
                    params,
                    ship,
                    shots,
                    particles,
                    &mut self.bullet_rand,
                )
            };
            if release {
                self.pool.release(bullet_ref);
            }
            if destroy {
                ship_destroyed = true;
            }
        }
        if ship_destroyed {
            self.clear_visible();
            params.shared_state.ship_destroyed();
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
            bullet.bullet = None;
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
        params: &mut GeneralParams,
        game_over: bool,
        float_letters: &mut FloatLetterPool,
    ) -> bool {
        let mut release_shot = false;
        for bullet in &mut self.pool {
            let rel_shot = bullet.check_shot_hit(shot, params, game_over, float_letters);
            if rel_shot {
                release_shot = true;
            }
        }
        release_shot
    }

    pub fn maybe_index_mut(&mut self, index: PoolActorRef) -> Option<&mut Bullet> {
        self.pool.maybe_index_mut(index)
    }

    pub fn barrage_rand(&mut self) -> &mut Rand {
        &mut self.barrage_rand
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
}

struct TTRunner {}

impl TTRunner {
    fn new() -> Self {
        TTRunner {}
    }
}

impl Default for TTRunner {
    fn default() -> Self {
        Self::new()
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
        f64::from(rtod(data.bullet.deg))
    }

    fn get_aim_direction(&self, data: &TTRunnerData) -> f64 {
        let b = data.bullet.pos;
        let t = data.target;
        f64::from(rtod(f32::atan2(t.x - b.x, t.y - b.y)))
    }

    fn get_bullet_speed(&self, data: &TTRunnerData) -> f64 {
        f64::from(data.bullet.speed * VEL_SS_SDM_RATIO)
    }

    fn get_default_speed(&self) -> f64 {
        1.
    }

    fn get_rank(&self, data: &TTRunnerData) -> f64 {
        f64::from(f32::min(data.bml_param.rank, 1.))
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
        f64::from(data.rand.gen_f32(1.))
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
