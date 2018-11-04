pub mod ship_shape;
pub mod shot_shape;
pub mod structure;

use crate::gl;

pub trait Drawable {
    fn draw(&self);
}

pub trait Collidable {
    fn collision(&self);
    fn check_collision(&self, ax: f32, ay: f32, shape: &Collidable, speed: f32);
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
