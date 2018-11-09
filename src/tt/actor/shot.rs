use crate::gl;

use crate::tt::actor::Pool;
use crate::tt::screen::Screen;
use crate::tt::shape::shot_shape::ShotShape;
use crate::tt::shape::ResizableDrawable;
use crate::tt::ship;
use crate::tt::tunnel::Tunnel;
use crate::util::vector::Vector;

const SPEED: f32 = 0.75;

const RANGE_MIN: f32 = 2.;
const RANGE_RATIO: f32 = 0.5;

const SIZE_MIN: f32 = 0.1;
const SIZE_RATIO: f32 = 0.15;

const MAX_CHARGE: u32 = 90;

const CHARGE_RELEASE_RATIO: f32 = 0.25;

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
    damage: u32,
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

    fn mov(&mut self, charge_shot: bool) -> bool {
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
            // TODO enemies.checkShotHit(pos, shape, this);
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
}

pub struct ShotPool {
    pool: Pool<Shot, ShotSpec>,
    shot_shape: ShotShape,
    charge_shot_shape: ShotShape,
}

enum ShotSpec {
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

    pub fn get_instance_and<O>(&mut self, op: O)
    where
        O: FnMut(&mut Shot),
    {
        self.pool.get_instance_and(ShotSpec::Normal, op)
    }

    pub fn get_charging_instance_and<O>(&mut self, op: O)
    where
        O: Fn(&mut Shot),
    {
        self.pool.get_special_instance_and(ShotSpec::Charge, |shot| {
            op(shot);
            false
        })
    }

    pub fn release_charging_instance(&mut self) {
        self.pool.get_special_instance_and(ShotSpec::Charge, |shot| {
            if shot.release() {
                true
            } else {
                false
            }
        });
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn mov(&mut self) {
        self.pool.foreach_mut(|spec, shot| match spec {
            ShotSpec::Normal => shot.mov(false),
            ShotSpec::Charge => shot.mov(true),
        });
    }

    pub fn draw(&self, tunnel: &Tunnel) {
        self.pool.foreach(|spec, shot| match spec {
            ShotSpec::Normal => shot.draw(&self.shot_shape, tunnel),
            ShotSpec::Charge => shot.draw(&self.charge_shot_shape, tunnel),
        });
    }
}
