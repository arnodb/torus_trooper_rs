use std::ops::{Index, IndexMut};

use crate::gl;

use crate::tt::actor::{Pool, PoolActor, PoolActorRef};
use crate::tt::screen::Screen;
use crate::tt::shape::shot_shape::ShotShape;
use crate::tt::shape::ResizableDrawable;
use crate::tt::ship::{self, Ship};
use crate::tt::state::in_game::ScoreAccumulator;
use crate::tt::tunnel::Tunnel;
use crate::util::vector::Vector;

use super::enemy::EnemyPool;

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
    pos: Vector,
    charge_cnt: u32,
    charge_se_cnt: u32,
    cnt: u32,
    range: f32,
    size: f32,
    trg_size: f32,
    in_charge: bool,
    star_shell: bool,
    shape: ResizableDrawable,
    multiplier: u32,
    damage: i32,
    deg: f32,
}

impl Shot {
    pub fn set(&mut self) {
        self.set_charge(false)
    }

    pub fn set_charge(&mut self, charge: bool) {
        self.set_charge_star(charge, false)
    }

    pub fn set_charge_star(&mut self, charge: bool, star: bool) {
        self.set_charge_star_deg(charge, star, 0.)
    }

    pub fn set_charge_star_deg(&mut self, charge: bool, star: bool, d: f32) {
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
            // TODO SoundManager.playSe("shot.wav");
        }
    }

    pub fn update(&mut self, p: Vector) {
        self.pos = Vector::new_at(p.x, p.y + 0.3);
    }

    pub fn release(&mut self) -> bool {
        if (self.charge_cnt as f32) < MAX_CHARGE as f32 * CHARGE_RELEASE_RATIO {
            return true;
        }
        self.in_charge = false;
        self.range = RANGE_MIN + self.charge_cnt as f32 * RANGE_RATIO;
        self.trg_size = SIZE_MIN + self.charge_cnt as f32 * SIZE_RATIO;
        // TODO SoundManager.playSe("charge_shot.wav");
        return false;
    }

    fn mov(
        &mut self,
        charge_shot: bool,
        shape: &ShotShape,
        tunnel: &Tunnel,
        ship: &mut Ship,
        enemies: &mut EnemyPool,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        let mut remove = false;
        if self.in_charge {
            if self.charge_cnt < MAX_CHARGE {
                self.charge_cnt += 1;
                self.trg_size = (SIZE_MIN + self.charge_cnt as f32 * SIZE_RATIO) * 0.33;
            }
            if (self.charge_se_cnt % 52) == 0 {
                // TODO SoundManager.playSe("charge.wav");
            }
            self.charge_se_cnt += 1;
        } else {
            self.pos.x += f32::sin(self.deg) * SPEED;
            self.pos.y += f32::cos(self.deg) * SPEED;
            self.range -= SPEED;
            if self.range <= 0. {
                remove = true;
            } else if self.range < 10. {
                self.trg_size *= 0.75;
            }
        }
        self.size += (self.trg_size - self.size) * 0.1;
        self.shape.size(self.size);
        if !self.in_charge {
            if charge_shot {
                // TODO bullets.checkShotHit(pos, shape, this);
            }
            let hit_remove = enemies.check_shot_hit(
                self.pos,
                shape,
                self,
                charge_shot,
                tunnel,
                ship,
                score_accumulator,
            );
            if hit_remove {
                remove = true;
            }
        }
        if self.star_shell || self.charge_cnt as f32 > MAX_CHARGE as f32 * CHARGE_RELEASE_RATIO {
            let pn = if charge_shot { 3 } else { 1 };
            for _i in 0..pn {
                /*TODO Particle pt = particles.getInstance();
                if (pt)
                pt.set(pos, 1, rand.nextSignedFloat(PI / 2) + PI, rand.nextSignedFloat(0.5), 0.05,
                       0.6, 1, 0.8, chargeCnt * 32 / MAX_CHARGE + 4);
                       */
            }
        }
        self.cnt += 1;
        remove
    }

    pub fn add_score(
        &mut self,
        charge_shot: bool,
        sc: u32,
        pos: Vector,
        score_accumulator: &mut ScoreAccumulator,
    ) -> bool {
        score_accumulator.add_score(sc * self.multiplier);
        /* TODO
        if (multiplier > 1) {
            FloatLetter fl = floatLetters.getInstanceForced();
            float size = 0.07;
            if (sc >= 100)
            size = 0.2;
            else if (sc >= 500)
            size = 0.4;
            else if (sc >= 2000)
            size = 0.7;
            size *= (1 + multiplier * 0.01f);
            fl.set("X" ~ std.string.toString(multiplier), pos, size * pos.y,
            cast(int) (30 + multiplier * 0.3f));
        }
        */
        if charge_shot {
            if self.multiplier < MAX_MULTIPLIER {
                self.multiplier += 1;
            }
            false
        } else {
            true
        }
    }

    fn draw(&self, shape: &ShotShape, tunnel: &Tunnel) {
        let sp = tunnel.get_pos_v(self.pos);
        unsafe {
            gl::PushMatrix();
        }
        sp.gl_translate();
        unsafe {
            gl::Rotatef(self.deg * 180. / std::f32::consts::PI, 0., 1., 10.);
            gl::Rotatef(self.cnt as f32 * 7., 0., 0., 1.);
        }
        self.shape.draw(shape);
        unsafe {
            gl::PopMatrix();
        }
    }

    pub fn damage(&self) -> i32 {
        self.damage
    }
}

pub struct ShotPool {
    pool: Pool<Shot, ShotSpec>,
    shot_shape: ShotShape,
    charge_shot_shape: ShotShape,
}

pub enum ShotSpec {
    Normal,
    Charge,
}

impl ShotPool {
    pub fn new(n: usize, screen: &Screen) -> Self {
        ShotPool {
            pool: Pool::new(n),
            shot_shape: ShotShape::new(false, screen),
            charge_shot_shape: ShotShape::new(true, screen),
        }
    }

    pub fn get_instance_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Shot),
    {
        let inst = self.pool.get_instance();
        if let Some((pa, pa_ref)) = inst {
            pa.prepare(pa_ref, ShotSpec::Normal);
            op(&mut pa.actor);
        }
    }

    pub fn get_charging_instance(&mut self) -> PoolActorRef {
        let (pa, pa_ref) = self.pool.get_instance_forced();
        pa.prepare(pa_ref, ShotSpec::Charge);
        pa_ref
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn mov(
        &mut self,
        tunnel: &Tunnel,
        ship: &mut Ship,
        enemies: &mut EnemyPool,
        score_accumulator: &mut ScoreAccumulator,
    ) {
        for pa in &mut self.pool {
            let release = match pa.state.spec() {
                ShotSpec::Normal => pa.actor.mov(
                    false,
                    &self.shot_shape,
                    tunnel,
                    ship,
                    enemies,
                    score_accumulator,
                ),
                ShotSpec::Charge => pa.actor.mov(
                    true,
                    &self.charge_shot_shape,
                    tunnel,
                    ship,
                    enemies,
                    score_accumulator,
                ),
            };
            if release {
                pa.release();
            }
        }
    }

    pub fn draw(&self, tunnel: &Tunnel) {
        for pa in &self.pool {
            match pa.state.spec() {
                ShotSpec::Normal => pa.actor.draw(&self.shot_shape, tunnel),
                ShotSpec::Charge => pa.actor.draw(&self.charge_shot_shape, tunnel),
            }
        }
    }
}

impl Index<PoolActorRef> for ShotPool {
    type Output = PoolActor<Shot, ShotSpec>;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<PoolActorRef> for ShotPool {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        &mut self.pool[index]
    }
}
