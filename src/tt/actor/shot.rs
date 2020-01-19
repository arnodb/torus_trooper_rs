use std::ops::{Index, IndexMut};
use std::rc::Rc;

use crate::gl;

use crate::tt::actor::bullet::BulletPool;
use crate::tt::actor::enemy::EnemyPool;
use crate::tt::actor::float_letter::FloatLetterPool;
use crate::tt::actor::particle::{ParticlePool, ParticleSpec};
use crate::tt::actor::{Pool, PoolActorRef};
use crate::tt::screen::Screen;
use crate::tt::shape::shot_shape::ShotShape;
use crate::tt::shape::{Drawable, ResizableDrawable};
use crate::tt::ship::{self, Ship};
use crate::tt::sound::SoundManager;
use crate::tt::tunnel::Tunnel;
use crate::tt::GeneralParams;
use crate::util::rand::Rand;
use crate::util::vector::Vector;

const SPEED: f32 = 0.75;

const RANGE_MIN: f32 = 2.;
const RANGE_RATIO: f32 = 0.5;

const SIZE_MIN: f32 = 0.1;
const SIZE_RATIO: f32 = 0.15;

const MAX_CHARGE: u32 = 90;

const CHARGE_RELEASE_RATIO: f32 = 0.25;

const MAX_MULTIPLIER: u32 = 100;

#[derive(Default)]
pub struct Shot {
    pub pos: Vector,
    charge_cnt: u32,
    charge_se_cnt: u32,
    cnt: u32,
    range: f32,
    size: f32,
    trg_size: f32,
    pub charge_shot: bool,
    in_charge: bool,
    star_shell: bool,
    pub shape: Option<ResizableDrawable<ShotShape>>,
    multiplier: u32,
    damage: i32,
    deg: f32,
}

impl Shot {
    pub fn set(&mut self, sound_manager: &SoundManager) {
        self.set_charge(false, sound_manager)
    }

    pub fn set_charge(&mut self, charge: bool, sound_manager: &SoundManager) {
        self.set_charge_star(charge, false, sound_manager)
    }

    pub fn set_charge_star(&mut self, charge: bool, star: bool, sound_manager: &SoundManager) {
        self.set_charge_star_deg(charge, star, 0., sound_manager)
    }

    pub fn set_charge_star_deg(
        &mut self,
        charge: bool,
        star: bool,
        d: f32,
        sound_manager: &SoundManager,
    ) {
        self.cnt = 0;
        self.multiplier = 1;
        self.in_charge = charge;
        self.charge_cnt = 0;
        self.charge_se_cnt = 0;
        self.deg = d;
        if charge {
            self.range = 0.;
            self.size = 0.;
            self.trg_size = 0.;
            self.damage = 100;
            self.star_shell = false;
        } else {
            self.range = ship::IN_SIGHT_DEPTH_DEFAULT;
            self.size = 1.;
            self.trg_size = 1.;
            self.damage = 1;
            self.star_shell = star;
            sound_manager.play_se("shot.wav");
        }
    }

    pub fn update(&mut self, p: Vector) {
        self.pos = Vector::new_at(p.x, p.y + 0.3);
    }

    pub fn release(&mut self, sound_manager: &SoundManager) -> bool {
        if (self.charge_cnt as f32) < MAX_CHARGE as f32 * CHARGE_RELEASE_RATIO {
            return true;
        }
        self.in_charge = false;
        self.range = RANGE_MIN + self.charge_cnt as f32 * RANGE_RATIO;
        self.trg_size = SIZE_MIN + self.charge_cnt as f32 * SIZE_RATIO;
        sound_manager.play_se("charge_shot.wav");
        false
    }

    fn mov(
        &mut self,
        params: &mut GeneralParams,
        ship: &mut Ship,
        bullets: &mut BulletPool,
        enemies: &mut EnemyPool,
        particles: &mut ParticlePool,
        float_letters: &mut FloatLetterPool,
        rand: &mut Rand,
    ) -> bool {
        let mut release = false;
        if self.in_charge {
            if self.charge_cnt < MAX_CHARGE {
                self.charge_cnt += 1;
                self.trg_size = (SIZE_MIN + self.charge_cnt as f32 * SIZE_RATIO) * 0.33;
            }
            if (self.charge_se_cnt % 52) == 0 {
                params.sound_manager.play_se("charge.wav");
            }
            self.charge_se_cnt += 1;
        } else {
            self.pos.x += f32::sin(self.deg) * SPEED;
            self.pos.y += f32::cos(self.deg) * SPEED;
            self.range -= SPEED;
            if self.range <= 0. {
                release = true;
            } else if self.range < 10. {
                self.trg_size *= 0.75;
            }
        }
        self.size += (self.trg_size - self.size) * 0.1;
        self.shape.as_mut().unwrap().size(self.size);
        if !self.in_charge {
            if self.charge_shot {
                let hit_release =
                    bullets.check_shot_hit(self, params, ship.is_game_over(), float_letters);
                if hit_release {
                    release = true;
                }
            }
            let hit_release =
                enemies.check_shot_hit(self, params, ship, bullets, particles, float_letters);
            if hit_release {
                release = true;
            }
        }
        if self.star_shell || self.charge_cnt as f32 > MAX_CHARGE as f32 * CHARGE_RELEASE_RATIO {
            let pn = if self.charge_shot { 3 } else { 1 };
            for _ in 0..pn {
                particles.get_instance_and(|pt, particles_rand| {
                    pt.set(
                        &ParticleSpec::Spark,
                        self.pos,
                        1.,
                        rand.gen_signed_f32(std::f32::consts::PI / 2.) + std::f32::consts::PI,
                        rand.gen_signed_f32(0.5),
                        0.05,
                        (0.6, 1., 0.8).into(),
                        (self.charge_cnt * 32 / MAX_CHARGE + 4) as i32,
                        params.tunnel,
                        particles_rand,
                    );
                });
            }
        }
        self.cnt += 1;
        release
    }

    pub fn add_score(
        &mut self,
        sc: u32,
        pos: Vector,
        params: &mut GeneralParams,
        game_over: bool,
        float_letters: &mut FloatLetterPool,
    ) -> bool {
        params.add_score(sc * self.multiplier, game_over);
        if self.multiplier > 1 {
            let size = if sc >= 100 {
                0.2
            } else if sc >= 500 {
                0.4
            } else if sc > 2000 {
                0.7
            } else {
                0.07
            };
            float_letters.spawn(
                format!("X{}", self.multiplier),
                pos,
                size * (1 as f32 + self.multiplier as f32 * 0.01) * pos.y,
                (30 as f32 + self.multiplier as f32 * 0.3) as i32,
            );
        }
        if self.charge_shot {
            if self.multiplier < MAX_MULTIPLIER {
                self.multiplier += 1;
            }
            false
        } else {
            true
        }
    }

    fn draw(&self, tunnel: &Tunnel) {
        let sp = tunnel.get_pos_v(self.pos);
        unsafe {
            gl::PushMatrix();
        }
        sp.gl_translate();
        unsafe {
            gl::Rotatef(self.deg * 180. / std::f32::consts::PI, 0., 1., 10.);
            gl::Rotatef(self.cnt as f32 * 7., 0., 0., 1.);
        }
        self.shape.as_ref().unwrap().draw();
        unsafe {
            gl::PopMatrix();
        }
    }

    pub fn damage(&self) -> i32 {
        self.damage
    }
}

pub struct ShotPool {
    pool: Pool<Shot>,
    shot_shape: Rc<ShotShape>,
    charge_shot_shape: Rc<ShotShape>,
    rand: Rand,
}

impl ShotPool {
    pub fn new(n: usize, screen: &Screen) -> Self {
        ShotPool {
            pool: Pool::new(n),
            shot_shape: Rc::new(ShotShape::new(false, screen)),
            charge_shot_shape: Rc::new(ShotShape::new(true, screen)),
            rand: Rand::new(Rand::rand_seed()),
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rand.set_seed(seed);
    }

    pub fn get_instance_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Shot),
    {
        let inst = self.pool.get_instance();
        if let Some((shot, _)) = inst {
            shot.charge_shot = false;
            shot.shape = Some(ResizableDrawable::new(&self.shot_shape, 0.));
            op(shot);
        }
    }

    pub fn get_charging_instance(&mut self) -> PoolActorRef {
        let (shot, shot_ref) = self.pool.get_instance_forced();
        shot.charge_shot = true;
        shot.shape = Some(ResizableDrawable::new(&self.charge_shot_shape, 0.));
        shot_ref
    }

    pub fn release(&mut self, shot_ref: PoolActorRef) {
        self.pool.release(shot_ref);
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn mov(
        &mut self,
        params: &mut GeneralParams,
        ship: &mut Ship,
        bullets: &mut BulletPool,
        enemies: &mut EnemyPool,
        particles: &mut ParticlePool,
        float_letters: &mut FloatLetterPool,
    ) {
        let (mut current_pool, _) = self.pool.split();
        let mut iter = current_pool.into_iter();
        while let Some((shot, _)) = iter.next() {
            let release = shot.mov(
                params,
                ship,
                bullets,
                enemies,
                particles,
                float_letters,
                &mut self.rand,
            );
            if release {
                iter.release();
            }
        }
    }

    pub fn draw(&self, tunnel: &Tunnel) {
        for shot in &self.pool {
            shot.draw(tunnel);
        }
    }
}

impl Index<PoolActorRef> for ShotPool {
    type Output = Shot;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<PoolActorRef> for ShotPool {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        &mut self.pool[index]
    }
}
