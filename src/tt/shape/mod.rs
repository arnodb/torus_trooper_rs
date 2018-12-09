pub mod ship_shape;
pub mod shot_shape;
pub mod structure;

use std::rc::Rc;

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

pub struct ResizableDrawable<T> {
    shape: Rc<T>,
    size: f32,
}

impl<T: Drawable> ResizableDrawable<T> {
    pub fn new(shape: &Rc<T>, size: f32) -> Self {
        ResizableDrawable {
            shape: shape.clone(),
            size,
        }
    }

    pub fn size(&mut self, size: f32) {
        self.size = size;
    }
}

impl<T: Drawable> Drawable for ResizableDrawable<T> {
    fn draw(&self) {
        unsafe {
            gl::Scalef(self.size, self.size, self.size);
        }
        self.shape.draw();
    }
}

impl<T: Collidable> Collidable for ResizableDrawable<T> {
    fn collision(&self) -> Vector {
        self.shape.collision() * self.size
    }
}
