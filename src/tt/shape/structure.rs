use crate::gl;

use crate::tt::screen::Screen;
use crate::util::vector::Vector;

pub struct Structure {
    pos: Vector,
    d1: f32,
    d2: f32,
    width: f32,
    height: f32,
    shape: Shape,
    shape_x_reverse: f32,
    color: usize,
    div_num: usize,
}

impl Structure {
    pub fn new(
        pos: Vector,
        d1: f32,
        d2: f32,
        width: f32,
        height: f32,
        shape: Shape,
        shape_x_reverse: f32,
        color: usize,
        div_num: usize,
    ) -> Self {
        Structure {
            pos,
            d1,
            d2,
            width,
            height,
            shape,
            shape_x_reverse,
            color,
            div_num,
        }
    }

    pub fn create_display_list(&self, screen: &Screen) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(self.pos.x, self.pos.y, 0.);
            gl::Rotatef(-self.d2, 1., 0., 0.);
            gl::Rotatef(self.d1, 0., 0., 1.);
            if self.shape == Shape::Rocket {
                gl::Scalef(self.width, self.width, self.height);
            } else {
                gl::Scalef(self.width, self.height, 1.);
            }
            gl::Scalef(self.shape_x_reverse, 1., 1.);
            let alp = match self.color {
                0 => 1.,
                _ => 0.5,
            };
            screen.set_color_rgb(
                COLOR_RGB[self.color][0],
                COLOR_RGB[self.color][1],
                COLOR_RGB[self.color][2],
            );
            match self.shape {
                Shape::Square => {
                    for i in 0..self.div_num {
                        let x11 = -0.5 + (1.0 / self.div_num as f32) * i as f32;
                        let x12 = x11 + (1.0 / self.div_num as f32) * 0.8;
                        let x21 = -0.5 + (0.8 / self.div_num as f32) * i as f32;
                        let x22 = x21 + (0.8 / self.div_num as f32) * 0.8;
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(x21, 0., -0.5);
                        gl::Vertex3f(x22, 0., -0.5);
                        gl::Vertex3f(x12, 0., 0.5);
                        gl::Vertex3f(x11, 0., 0.5);
                        gl::End();
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(x21, 0.1, -0.5);
                        gl::Vertex3f(x22, 0.1, -0.5);
                        gl::Vertex3f(x12, 0.1, 0.5);
                        gl::Vertex3f(x11, 0.1, 0.5);
                        gl::End();
                        screen.set_color_rgba(
                            COLOR_RGB[self.color][0],
                            COLOR_RGB[self.color][1],
                            COLOR_RGB[self.color][2],
                            alp,
                        );
                        gl::Begin(gl::GL_TRIANGLE_FAN);
                        gl::Vertex3f(x21, 0., -0.5);
                        gl::Vertex3f(x22, 0., -0.5);
                        gl::Vertex3f(x12, 0., 0.5);
                        gl::Vertex3f(x11, 0., 0.5);
                        gl::End();
                    }
                }
                Shape::Wing => {
                    for i in 0..self.div_num {
                        let x1 = -0.5 + (1.0 / self.div_num as f32) * i as f32;
                        let x2 = x1 + (1.0 / self.div_num as f32) * 0.8;
                        let y1 = x1;
                        let y2 = x2;
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(x1, 0., y1);
                        gl::Vertex3f(x2, 0., y2);
                        gl::Vertex3f(x2, 0., 0.5);
                        gl::Vertex3f(x1, 0., 0.5);
                        gl::End();
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(x1, 0.1, y1);
                        gl::Vertex3f(x2, 0.1, y2);
                        gl::Vertex3f(x2, 0.1, 0.5);
                        gl::Vertex3f(x1, 0.1, 0.5);
                        gl::End();
                        screen.set_color_rgba(
                            COLOR_RGB[self.color][0],
                            COLOR_RGB[self.color][1],
                            COLOR_RGB[self.color][2],
                            alp,
                        );
                        gl::Begin(gl::GL_TRIANGLE_FAN);
                        gl::Vertex3f(x1, 0., y1);
                        gl::Vertex3f(x2, 0., y2);
                        gl::Vertex3f(x2, 0., 0.5);
                        gl::Vertex3f(x1, 0., 0.5);
                        gl::End();
                    }
                }
                Shape::Triangle => {
                    for i in 0..self.div_num {
                        let x1 = -0.5 + (1.0 / self.div_num as f32) * i as f32;
                        let x2 = x1 + (1.0 / self.div_num as f32) * 0.8;
                        let y1 = -0.5
                            + (1.0 / self.div_num as f32)
                                * f32::abs(i as f32 - self.div_num as f32 / 2.)
                                * 2.;
                        let y2 = -0.5
                            + (1.0 / self.div_num as f32)
                                * f32::abs(i as f32 + 0.8 - self.div_num as f32 / 2.)
                                * 2.;
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(x1, 0., y1);
                        gl::Vertex3f(x2, 0., y2);
                        gl::Vertex3f(x2, 0., 0.5);
                        gl::Vertex3f(x1, 0., 0.5);
                        gl::End();
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(x1, 0.1, y1);
                        gl::Vertex3f(x2, 0.1, y2);
                        gl::Vertex3f(x2, 0.1, 0.5);
                        gl::Vertex3f(x1, 0.1, 0.5);
                        gl::End();
                        screen.set_color_rgba(
                            COLOR_RGB[self.color][0],
                            COLOR_RGB[self.color][1],
                            COLOR_RGB[self.color][2],
                            alp,
                        );
                        gl::Begin(gl::GL_TRIANGLE_FAN);
                        gl::Vertex3f(x1, 0., y1);
                        gl::Vertex3f(x2, 0., y2);
                        gl::Vertex3f(x2, 0., 0.5);
                        gl::Vertex3f(x1, 0., 0.5);
                        gl::End();
                    }
                }
                Shape::Rocket => {
                    for i in 0..4 {
                        let d = i as f32 * std::f32::consts::PI / 2. + std::f32::consts::PI / 4.;
                        gl::Begin(gl::GL_LINE_LOOP);
                        gl::Vertex3f(f32::sin(d - 0.3), f32::cos(d - 0.3), -0.5);
                        gl::Vertex3f(f32::sin(d + 0.3), f32::cos(d + 0.3), -0.5);
                        gl::Vertex3f(f32::sin(d + 0.3), f32::cos(d + 0.3), 0.5);
                        gl::Vertex3f(f32::sin(d - 0.3), f32::cos(d - 0.3), 0.5);
                        gl::End();
                        screen.set_color_rgba(
                            COLOR_RGB[self.color][0],
                            COLOR_RGB[self.color][1],
                            COLOR_RGB[self.color][2],
                            alp,
                        );
                        gl::Begin(gl::GL_TRIANGLE_FAN);
                        gl::Vertex3f(f32::sin(d - 0.3), f32::cos(d - 0.3), -0.5);
                        gl::Vertex3f(f32::sin(d + 0.3), f32::cos(d + 0.3), -0.5);
                        gl::Vertex3f(f32::sin(d + 0.3), f32::cos(d + 0.3), 0.5);
                        gl::Vertex3f(f32::sin(d - 0.3), f32::cos(d - 0.3), 0.5);
                        gl::End();
                    }
                }
            }
            gl::PopMatrix();
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Shape {
    Square,
    Wing,
    Triangle,
    Rocket,
}

pub const SHIP_SHAPES: [Shape; 3] = [Shape::Square, Shape::Wing, Shape::Triangle];

pub const COLOR_RGB: [[f32; 3]; 8] = [
    [1., 1., 1.],
    [0.6, 0.6, 0.6],
    [0.9, 0.5, 0.5],
    [0.5, 0.9, 0.5],
    [0.5, 0.5, 0.9],
    [0.7, 0.7, 0.5],
    [0.7, 0.5, 0.7],
    [0.5, 0.7, 0.7],
];
