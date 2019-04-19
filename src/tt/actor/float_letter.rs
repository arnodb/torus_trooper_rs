use std::ops::{Index, IndexMut};

use crate::gl;

use crate::tt::actor::{Pool, PoolActorRef};
use crate::tt::letter::{self, Letter};
use crate::tt::screen::Screen;
use crate::tt::tunnel::Tunnel;
use crate::tt::GeneralParams;
use crate::util::rand::Rand;
use crate::util::vector::{Vector, Vector3};

#[derive(Default)]
pub struct FloatLetter {
    pos: Vector3,
    mx: f32,
    my: f32,
    d: f32,
    size: f32,
    msg: String,
    cnt: i32,
    alpha: f32,
}

impl FloatLetter {
    pub fn set(&mut self, msg: String, p: Vector, s: f32, c: i32, rand: &mut Rand) {
        self.pos = Vector3::new_at(p.x, p.y, 1.);
        self.mx = rand.gen_signed_f32(0.001);
        self.my = rand.gen_f32(0.2) + 0.2;
        self.d = p.x;
        self.size = s;
        self.msg = msg;
        self.cnt = c;
        self.alpha = 0.8;
    }

    fn mov(&mut self) -> bool {
        self.pos += Vector3::new_at(self.mx * self.pos.y, self.my, -0.03 * self.pos.y);
        self.cnt -= 1;
        if self.alpha >= 0.03 {
            self.alpha -= 0.03;
        }
        self.cnt < 0
    }

    fn draw(&self, screen: &Screen, letter: &Letter, tunnel: &Tunnel) {
        unsafe {
            gl::PushMatrix();
        }
        let sp = tunnel.get_pos_v3(self.pos);
        unsafe {
            gl::Translatef(0., 0., sp.z);
        }
        screen.set_alpha_color((1., 1., 1., 1.));
        letter.draw_string_full(
            &self.msg,
            sp.x,
            sp.y,
            self.size,
            letter::Direction::ToRight,
            2,
            false,
            self.d * 180. / std::f32::consts::PI,
        );
        screen.set_alpha_color((1., 1., 1., self.alpha));
        letter.draw_string_full(
            &self.msg,
            sp.x,
            sp.y,
            self.size,
            letter::Direction::ToRight,
            3,
            false,
            self.d * 180. / std::f32::consts::PI,
        );
        unsafe {
            gl::PopMatrix();
        }
    }
}

pub struct FloatLetterPool {
    pool: Pool<FloatLetter>,
    rand: Rand,
}

impl FloatLetterPool {
    pub fn new(n: usize) -> Self {
        FloatLetterPool {
            pool: Pool::new(n),
            rand: Rand::new(Rand::rand_seed()),
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rand.set_seed(seed);
    }

    pub fn spawn(&mut self, msg: String, p: Vector, s: f32, c: i32) {
        let fl = self.pool.get_instance_forced().0;
        fl.set(msg, p, s, c, &mut self.rand);
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn mov(&mut self) {
        for fl_ref in self.pool.as_refs() {
            let release = {
                let fl = &mut self.pool[fl_ref];
                fl.mov()
            };
            if release {
                self.pool.release(fl_ref);
            }
        }
    }

    pub fn draw(&self, params: &GeneralParams) {
        for fl in &self.pool {
            fl.draw(params.screen, params.letter, params.tunnel);
        }
    }
}

impl Index<PoolActorRef> for FloatLetterPool {
    type Output = FloatLetter;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<PoolActorRef> for FloatLetterPool {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        &mut self.pool[index]
    }
}
