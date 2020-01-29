use std::ops::{Index, IndexMut};

use crate::gl;

use crate::tt::actor::pool::{Pool, PoolActorRef};
use crate::tt::screen::Screen;
use crate::tt::tunnel::{InCourseSliceCheck, Tunnel};
use crate::util::color::Color;
use crate::util::rand::Rand;
use crate::util::vector::{Vector, Vector3};

const GRAVITY: f32 = 0.02;
const SIZE: f32 = 0.3;

#[derive(Default)]
pub struct Particle {
    pos: Vector3,
    vel: Vector3,
    sp: Vector3,
    psp: Vector3,
    rsp: Vector3,
    rpsp: Vector3,
    color: Color,
    lum_alp: f32,
    cnt: i32,
    in_course: bool,
    spec: ParticleSpec,
}

#[derive(Clone, Copy, Debug)]
pub enum ParticleSpec {
    Spark,
    Star,
    Fragment {
        d1: f32,
        d2: f32,
        md1: f32,
        md2: f32,
        width: f32,
        height: f32,
    },
    Jet,
}

impl Default for ParticleSpec {
    fn default() -> Self {
        ParticleSpec::Spark
    }
}

impl Particle {
    pub fn set(
        &mut self,
        spec: &ParticleSpec,
        p: Vector,
        z: f32,
        d: f32,
        mz: f32,
        speed: f32,
        color: Color,
        c: i32,
        tunnel: &Tunnel,
        rand: &mut Rand,
    ) {
        self.pos = Vector3::new_at(p.x, p.y, z);
        let sb = rand.gen_f32(0.8) + 0.4;
        self.vel = Vector3::new_at(f32::sin(d) * speed * sb, f32::cos(d) * speed * sb, mz);
        self.color = color;
        self.cnt = c + rand.gen_usize((c / 2) as usize) as i32;
        self.lum_alp = 0.8 + rand.gen_f32(0.2);
        self.in_course = if let ParticleSpec::Star = spec {
            false
        } else {
            true
        };
        self.spec = *spec;
        if let ParticleSpec::Fragment { md1, md2, .. } = &mut self.spec {
            *md1 = rand.gen_signed_f32(12.);
            *md2 = rand.gen_signed_f32(12.);
        }
        self.check_in_course(tunnel);
        self.calc_screen_pos(tunnel);
    }

    fn mov(&mut self, ship_speed: f32, tunnel: &Tunnel) -> bool {
        self.cnt -= 1;
        if self.cnt < 0 || self.pos.y < -2. {
            return true;
        }
        self.psp = self.sp;
        if self.in_course {
            self.rpsp = self.rsp;
        }
        self.pos += self.vel;
        let do_cic = match &mut self.spec {
            ParticleSpec::Fragment {
                d1,
                d2,
                md1,
                md2,
                width,
                height,
            } => {
                self.pos.y -= ship_speed / 2.;
                self.vel.z -= GRAVITY / 2.;
                *d1 += *md1;
                *d2 += *md2;
                *md1 *= 0.98;
                *md2 *= 0.98;
                *width *= 0.98;
                *height *= 0.98;
                true
            }
            ParticleSpec::Spark => {
                self.pos.y -= ship_speed * 0.33;
                self.vel.z -= GRAVITY;
                true
            }
            ParticleSpec::Star => {
                self.pos.y -= ship_speed;
                false
            }
            ParticleSpec::Jet => {
                self.pos.y -= ship_speed;
                self.vel.z -= GRAVITY;
                true
            }
        };
        if do_cic && self.in_course && self.pos.z < 0. {
            self.vel.z *= -0.8;
            self.vel *= 0.9;
            self.pos.z += self.vel.z * 2.;
            self.check_in_course(tunnel);
        }
        self.lum_alp *= 0.98;
        self.calc_screen_pos(tunnel);
        false
    }

    fn calc_screen_pos(&mut self, tunnel: &Tunnel) {
        self.sp = tunnel.get_pos_v3(self.pos);
        if self.in_course {
            let pos_z = Vector3::new_at(self.pos.x, self.pos.y, -self.pos.z);
            self.rsp = tunnel.get_pos_v3(pos_z);
        }
    }

    fn check_in_course(&mut self, tunnel: &Tunnel) {
        if let InCourseSliceCheck::NotInCourse(..) =
            tunnel.check_in_course(Vector::new_at(self.pos.x, self.pos.y))
        {
            self.in_course = false;
        }
    }

    fn draw(&self, screen: &Screen) {
        match self.spec {
            ParticleSpec::Spark | ParticleSpec::Jet => self.draw_spark(screen),
            ParticleSpec::Star => self.draw_star(screen),
            ParticleSpec::Fragment {
                d1,
                d2,
                width,
                height,
                ..
            } => self.draw_fragment(d1, d2, width, height, screen),
        }
    }

    fn draw_spark(&self, screen: &Screen) {
        unsafe {
            gl::Begin(gl::GL_TRIANGLE_FAN);
        }
        screen.set_alpha_color(self.color.with_alpha(0.5));
        self.psp.gl_vertex();
        screen.set_alpha_color(self.color.with_alpha(0.));
        unsafe {
            gl::Vertex3f(self.sp.x - SIZE, self.sp.y - SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x + SIZE, self.sp.y - SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x + SIZE, self.sp.y + SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x - SIZE, self.sp.y + SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x - SIZE, self.sp.y - SIZE, self.sp.z);
            gl::End();
        }
        if self.in_course {
            unsafe {
                gl::Begin(gl::GL_TRIANGLE_FAN);
            }
            screen.set_alpha_color(self.color.with_alpha(0.2));
            self.rpsp.gl_vertex();
            screen.set_alpha_color(self.color.with_alpha(0.));
            unsafe {
                gl::Vertex3f(self.rsp.x - SIZE, self.rsp.y - SIZE, self.sp.z);
                gl::Vertex3f(self.rsp.x + SIZE, self.rsp.y - SIZE, self.sp.z);
                gl::Vertex3f(self.rsp.x + SIZE, self.rsp.y + SIZE, self.sp.z);
                gl::Vertex3f(self.rsp.x - SIZE, self.rsp.y + SIZE, self.sp.z);
                gl::Vertex3f(self.rsp.x - SIZE, self.rsp.y - SIZE, self.sp.z);
                gl::End();
            }
        }
    }

    fn draw_star(&self, screen: &Screen) {
        unsafe {
            gl::Begin(gl::GL_LINES);
        }
        screen.set_alpha_color(self.color.with_alpha(1.));
        self.psp.gl_vertex();
        screen.set_alpha_color(self.color.with_alpha(0.2));
        self.sp.gl_vertex();
        unsafe {
            gl::End();
        }
    }

    fn draw_fragment(&self, d1: f32, d2: f32, width: f32, height: f32, screen: &Screen) {
        unsafe {
            gl::PushMatrix();
        }
        self.sp.gl_translate();
        unsafe {
            gl::Rotatef(d1, 0., 0., 1.);
            gl::Rotatef(d2, 0., 1., 0.);
            gl::Begin(gl::GL_LINE_LOOP);
        }
        screen.set_alpha_color(self.color.with_alpha(0.5));
        unsafe {
            gl::Vertex3f(width, 0., height);
            gl::Vertex3f(-width, 0., height);
            gl::Vertex3f(-width, 0., -height);
            gl::Vertex3f(width, 0., -height);
            gl::End();
            gl::Begin(gl::GL_TRIANGLE_FAN);
        }
        screen.set_alpha_color(self.color.with_alpha(0.2));
        unsafe {
            gl::Vertex3f(width, 0., height);
            gl::Vertex3f(-width, 0., height);
            gl::Vertex3f(-width, 0., -height);
            gl::Vertex3f(width, 0., -height);
            gl::End();
            gl::PopMatrix();
        }
    }

    fn draw_luminous(&self, screen: &Screen) {
        if self.lum_alp < 0.2 || {
            if let ParticleSpec::Spark = self.spec {
                false
            } else {
                true
            }
        } {
            return;
        }
        unsafe {
            gl::Begin(gl::GL_TRIANGLE_FAN);
        }
        screen.set_alpha_color(self.color.with_alpha(self.lum_alp * 0.6));
        unsafe {
            gl::Vertex3f(self.psp.x, self.psp.y, self.psp.z);
        }
        screen.set_alpha_color(self.color.with_alpha(0.));
        unsafe {
            gl::Vertex3f(self.sp.x - SIZE, self.sp.y - SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x + SIZE, self.sp.y - SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x + SIZE, self.sp.y + SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x - SIZE, self.sp.y + SIZE, self.sp.z);
            gl::Vertex3f(self.sp.x - SIZE, self.sp.y - SIZE, self.sp.z);
            gl::End();
        }
    }
}

pub struct ParticlePool {
    pool: Pool<Particle>,
    rand: Rand,
}

impl ParticlePool {
    pub fn new(n: usize, seed: u64) -> Self {
        ParticlePool {
            pool: Pool::new(n),
            rand: Rand::new(seed),
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rand.set_seed(seed);
    }

    pub fn get_instance_and<O>(&mut self, mut op: O) -> bool
    where
        O: FnMut(&mut Particle, &mut Rand),
    {
        let inst = self.pool.get_instance();
        if let Some((particle, _)) = inst {
            op(particle, &mut self.rand);
            true
        } else {
            false
        }
    }

    pub fn get_instance_forced_and<O>(&mut self, mut op: O)
    where
        O: FnMut(&mut Particle, &mut Rand),
    {
        let particle = self.pool.get_instance_forced().0;
        op(particle, &mut self.rand);
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn mov(&mut self, ship_speed: f32, tunnel: &Tunnel) {
        let (mut current_pool, _) = self.pool.split();
        let mut iter = current_pool.into_iter();
        while let Some((particle, _)) = iter.next() {
            let release = particle.mov(ship_speed, tunnel);
            if release {
                iter.release();
            }
        }
    }

    pub fn draw(&mut self, screen: &Screen) {
        for particle in &self.pool {
            particle.draw(screen);
        }
    }

    pub fn draw_luminous(&mut self, screen: &Screen) {
        for particle in &self.pool {
            particle.draw_luminous(screen);
        }
    }
}

impl Index<PoolActorRef> for ParticlePool {
    type Output = Particle;
    fn index(&self, index: PoolActorRef) -> &Self::Output {
        &self.pool[index]
    }
}

impl IndexMut<PoolActorRef> for ParticlePool {
    fn index_mut(&mut self, index: PoolActorRef) -> &mut Self::Output {
        &mut self.pool[index]
    }
}
