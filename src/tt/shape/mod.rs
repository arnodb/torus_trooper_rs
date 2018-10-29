pub mod ship_shape;
pub mod structure;

pub trait Drawable {
    fn draw(&self);
}

pub trait Collidable {
    fn collision(&self);
    fn check_collision(&self, ax: f32, ay: f32, shape: &Collidable, speed: f32);
}
