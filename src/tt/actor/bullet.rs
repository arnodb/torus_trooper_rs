use std::ops::{Index, IndexMut};
use std::rc::Rc;

use bulletml::{AppRunner, BulletML, Runner, RunnerData, State};

use crate::gl;

use crate::util::rand::Rand;
use crate::util::vector::Vector;

use crate::tt::actor::shot::Shot;
use crate::tt::actor::{Pool, PoolActor, PoolActorRef};
use crate::tt::bullet::BulletTarget;
use crate::tt::shape::{Collidable, Drawable};
use crate::tt::ship::Ship;
use crate::tt::state::in_game::ScoreAccumulator;
use crate::tt::tunnel::{self, Tunnel};

#[derive(Default)]
pub struct Bullet {
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

const DISAP_CNT: u32 = 45;

impl Bullet {
    fn set(&mut self, bullet: &mut BulletImpl, pos: Vector, deg: f32, speed: f32) {
        bullet.set(pos, deg, speed);
        self.is_simple = false;
        self.start(bullet);
    }

    fn set_simple(&mut self, bullet: &mut BulletImpl, pos: Vector, deg: f32, speed: f32) {
        bullet.set(pos, deg, speed);
        self.is_simple = true;
        self.start(bullet);
    }

    fn start(&mut self, bullet: &BulletImpl) {
        self.is_top = false;
        self.is_aim_top = false;
        self.is_wait = false;
        self.is_visible = true;
        self.is_morph_seed = false;
        self.ppos = bullet.pos;
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
        runner: &mut Runner<TTRunner>,
        bullet: &mut BulletImpl,
        bml_params: &Rc<Vec<BMLParam>>,
        tunnel: &Tunnel,
        ship: &Ship,
        rand: &mut Rand,
    ) -> bool {
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
        let mx =
            (f32::sin(bullet.deg) * bullet.speed + bullet.acc.x) * speed_rank * bullet.x_reverse;
        let my =
            (f32::cos(bullet.deg) * bullet.speed - bullet.acc.y) * speed_rank * bullet.y_reverse;
        let d = f32::atan2(mx, my);
        let r = (1. - f32::abs(f32::sin(d)) * 0.999) * (ship.speed() * 5.);
        bullet.pos.x += mx * r;
        bullet.pos.y += my * r;

        if bullet.pos.x >= std::f32::consts::PI * 2. {
            bullet.pos.x -= std::f32::consts::PI * 2.;
        } else if bullet.pos.x < 0. {
            bullet.pos.x += std::f32::consts::PI * 2.;
        }
        let mut release = false;
        if self.is_visible && self.disap_cnt <= 0 {
            if ship.check_bullet_hit(bullet.pos, self.ppos) {
                release = true;
            }
            if bullet.pos.y < -2.
                || (!bullet.long_range && bullet.pos.y > ship.in_sight_depth())
                || !tunnel.check_in_screen(bullet.pos, ship)
            {
                self.start_disappear();
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
                self.start_disappear();
            }
        }
        release
    }

    fn check_shot_hit(
        &mut self,
        bullet: &BulletImpl,
        p: Vector,
        shape: &Collidable,
        shot: &mut Shot,
        charge_shot: bool,
        tunnel: &Tunnel,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        if !self.is_visible || self.disap_cnt > 0 {
            return false;
        }
        let mut ox = f32::abs(bullet.pos.x - p.x);
        let oy = f32::abs(bullet.pos.y - p.y);
        if ox > std::f32::consts::PI {
            ox = std::f32::consts::PI * 2. - ox;
        }
        ox *= (tunnel.get_radius(bullet.pos.y) / tunnel::DEFAULT_RAD) * 3.;
        let mut release_shot = false;
        if shape.check_collision(ox, oy) {
            self.start_disappear();
            release_shot = shot.add_score(charge_shot, 10, bullet.pos, score_accumulator);
        }
        release_shot
    }

    fn draw(&self, bullet: &BulletImpl, tunnel: &Tunnel) {
        if self.is_visible {
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
    pool: Pool<Bullet, BulletSpec>,
    cnt: u32,
    bullet_rand: Rand,
}

pub struct BulletSpec {
    bml_params: Rc<Vec<BMLParam>>,
    runner: Runner<TTRunner>,
    pub bullet: BulletImpl,
}

impl BulletPool {
    pub fn new(n: usize) -> Self {
        BulletPool {
            pool: Pool::new(n),
            cnt: 0,
            bullet_rand: Rand::new(Rand::rand_seed()),
        }
    }

    fn add_bullet(
        &mut self,
        pa: &PoolActor<Bullet, BulletSpec>,
        deg: f32,
        speed: f32,
    ) -> Option<PoolActorRef> {
        let spec = pa.state.spec();
        if let Some(rb) = spec.bullet.root_bullet {
            let rb = &self.pool[rb];
            if rb.actor.root_rank <= 0. {
                return None;
            }
        }
        let bml_idx = pa.actor.bml_idx;
        let inst = self.pool.get_instance();
        if let Some((pa, pa_ref)) = inst {
            let bml_params = &spec.bml_params;
            let orig_bullet = &spec.bullet;
            let (goto_next_parser, bml_idx) = {
                if bml_idx + 1 >= bml_params.len() {
                    (false, bml_idx)
                } else {
                    (true, bml_idx + 1)
                }
            };
            pa.actor.bml_idx = bml_idx;
            let bml_param = &bml_params[bml_idx];
            let runner = Runner::new(TTRunner::new(), &bml_param.bml);
            pa.prepare(
                pa_ref,
                BulletSpec {
                    bml_params: bml_params.clone(),
                    runner,
                    // TODO?
                    bullet: BulletImpl::new_param(
                        &orig_bullet.shape,
                        &orig_bullet.disap_shape,
                        orig_bullet.x_reverse,
                        orig_bullet.y_reverse,
                        orig_bullet.long_range,
                        /*TODO*/
                    ),
                },
            );
            let spec = pa.state.spec_mut();
            if goto_next_parser {
                pa.actor.set(&mut spec.bullet, orig_bullet.pos, deg, speed);
                pa.actor.set_morph_seed();
            } else {
                pa.actor
                    .set_simple(&mut spec.bullet, orig_bullet.pos, deg, speed);
            }
            Some(pa_ref)
        } else {
            None
        }
    }

    fn add_bullet_state(
        &mut self,
        pa: &PoolActor<Bullet, BulletSpec>,
        state: State,
        deg: f32,
        speed: f32,
    ) -> Option<PoolActorRef> {
        let spec = pa.state.spec();
        if let Some(rb) = spec.bullet.root_bullet {
            let rb = &self.pool[rb];
            if rb.actor.root_rank <= 0. {
                return None;
            }
        }
        let bml_idx = pa.actor.bml_idx;
        let inst = self.pool.get_instance();
        if let Some((pa, pa_ref)) = inst {
            pa.actor.bml_idx = bml_idx;
            let bml_params = &spec.bml_params;
            let orig_bullet = &spec.bullet;
            let runner = Runner::new_from_state(TTRunner::new(), state);
            pa.prepare(
                pa_ref,
                BulletSpec {
                    bml_params: bml_params.clone(),
                    runner,
                    // TODO?
                    bullet: BulletImpl::new_param(
                        &orig_bullet.shape,
                        &orig_bullet.disap_shape,
                        orig_bullet.x_reverse,
                        orig_bullet.y_reverse,
                        orig_bullet.long_range,
                        /*TODO*/
                    ),
                },
            );
            let spec = pa.state.spec_mut();
            pa.actor.set(&mut spec.bullet, orig_bullet.pos, deg, speed);
            Some(pa_ref)
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
        if let Some((pa, pa_ref)) = inst {
            pa.actor.bml_idx = 0;
            let bml_param = &bml_params[0];
            let runner = Runner::new(TTRunner::new(), &bml_param.bml);
            pa.prepare(
                pa_ref,
                BulletSpec {
                    bml_params: bml_params.clone(),
                    runner,
                    bullet: BulletImpl::new_param_first(
                        shape,
                        disap_shape,
                        x_reverse,
                        y_reverse,
                        long_range,
                        /*TODO*/
                        Some(pa_ref),
                    ),
                },
            );
            let spec = pa.state.spec_mut();
            pa.actor.set(&mut spec.bullet, pos, deg, speed);
            pa.actor.set_wait(prev_wait, post_wait);
            pa.actor.set_top();
            Some(pa_ref)
        } else {
            None
        }
    }

    pub fn mov(&mut self, tunnel: &Tunnel, ship: &Ship) {
        let invariant_self = unsafe { &mut *(self as *mut BulletPool) };
        // XXX acting_refs resolves currently acting bullets so
        // invariant_self is safe and so is invariant_pa.
        for pa_ref in self.pool.acting_refs() {
            let mut pa = &mut self.pool[pa_ref];
            let release = {
                let invariant_pa = unsafe { &mut *(pa as *mut PoolActor<Bullet, BulletSpec>) };
                let spec = pa.state.spec_mut();
                let mut manager = BulletsManager::new(invariant_self, invariant_pa);
                pa.actor.mov(
                    &mut manager,
                    &mut spec.runner,
                    &mut spec.bullet,
                    &spec.bml_params,
                    tunnel,
                    ship,
                    &mut self.bullet_rand,
                )
            };
            if release {
                pa.release();
            }
        }
        self.cnt += 1;
    }

    pub fn draw(&self, tunnel: &Tunnel) {
        for pa in &self.pool {
            let spec = pa.state.spec();
            pa.actor.draw(&spec.bullet, tunnel);
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
        self.cnt = 0;
    }

    fn get_turn(&self) -> u32 {
        self.cnt
    }

    pub fn check_shot_hit(
        &mut self,
        p: Vector,
        shape: &Collidable,
        shot: &mut Shot,
        charge_shot: bool,
        tunnel: &Tunnel,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        let mut release_shot = false;
        for pa in &mut self.pool {
            let rel_shot = pa.actor.check_shot_hit(
                &pa.state.spec().bullet,
                p,
                shape,
                shot,
                charge_shot,
                tunnel,
                score_accumulator,
            );
            if rel_shot {
                release_shot = true;
            }
        }
        release_shot
    }

    pub fn maybe_index_mut(
        &mut self,
        index: PoolActorRef,
    ) -> Option<&mut PoolActor<Bullet, BulletSpec>> {
        self.pool.maybe_index_mut(index)
    }
}

impl Index<PoolActorRef> for BulletPool {
    type Output = PoolActor<Bullet, BulletSpec>;
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
    pa: &'a PoolActor<Bullet, BulletSpec>,
    bullet_should_be_released: bool,
}

impl<'a> BulletsManager<'a> {
    fn new(bullets: &'a mut BulletPool, pa: &'a mut PoolActor<Bullet, BulletSpec>) -> Self {
        BulletsManager {
            bullets,
            pa,
            bullet_should_be_released: false,
        }
    }

    fn add_bullet(&mut self, deg: f32, speed: f32) {
        self.bullets.add_bullet(self.pa, deg, speed);
    }

    fn add_bullet_state(&mut self, state: State, deg: f32, speed: f32) {
        self.bullets.add_bullet_state(self.pa, state, deg, speed);
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
    deg: f32,
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
