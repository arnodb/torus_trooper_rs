pub mod ship_shape;
pub mod shot_shape;
pub mod structure;

use crate::util::vector::Vector;

use crate::gl;

pub trait Drawable {
    fn draw(&self);
}

pub trait Collidable {
    fn collision(&self) -> Vector;

    fn check_collision_shape(&self, ax: f32, ay: f32, shape: &Collidable, speed: f32) -> bool {
        let mut c = self.collision() + shape.collision();
        c.y *= speed;
        ax <= c.x && ay <= c.y
    }

    fn check_collision(&self, ax: f32, ay: f32) -> bool {
        let c = self.collision();
        ax <= c.x && ay <= c.y
    }
}

#[derive(Default)]
pub struct ResizableDrawable {
    size: f32,
    // TODO collision
}

impl ResizableDrawable {
    pub fn draw(&self, shape: &Drawable) {
        unsafe {
            gl::Scalef(self.size, self.size, self.size);
        }
        shape.draw();
    }

    pub fn size(&mut self, size: f32) {
        self.size = size;
    }
}
