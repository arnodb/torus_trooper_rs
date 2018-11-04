use crate::gl;

use crate::util::display_list::DisplayList;
use crate::util::vector::Vector;
// FIXME we are in util!
use crate::tt::screen::Screen;

use super::Drawable;

const COLOR_RGB: [f32; 3] = [0.8, 1., 0.7];

pub struct ShotShape {
    collision: Vector,
    display_list: DisplayList,
}

impl ShotShape {
    pub fn new<'b>(charge: bool, screen: &'b Screen) -> ShotShape {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        if charge {
            for i in 0..8 {
                let d = i as f32 * std::f32::consts::PI / 4.;
                unsafe {
                    gl::Begin(gl::GL_TRIANGLES);
                    screen.set_color_rgb(COLOR_RGB[0], COLOR_RGB[1], COLOR_RGB[2]);
                    gl::Vertex3f(f32::sin(d) * 0.1, f32::cos(d) * 0.1, 0.2);
                    gl::Vertex3f(f32::sin(d) * 0.5, f32::cos(d) * 0.5, 0.5);
                    screen.set_color_rgb(
                        COLOR_RGB[0] * 0.2,
                        COLOR_RGB[1] * 0.2,
                        COLOR_RGB[2] * 0.2,
                    );
                    gl::Vertex3f(f32::sin(d) * 1.0, f32::cos(d) * 1.0, -0.7);
                    gl::End();
                    screen.set_color_rgb(COLOR_RGB[0], COLOR_RGB[1], COLOR_RGB[2]);
                    gl::Begin(gl::GL_LINE_LOOP);
                    gl::Vertex3f(f32::sin(d) * 0.1, f32::cos(d) * 0.1, 0.2);
                    gl::Vertex3f(f32::sin(d) * 0.5, f32::cos(d) * 0.5, 0.5);
                    gl::Vertex3f(f32::sin(d) * 1.0, f32::cos(d) * 1.0, -0.7);
                    gl::End();
                }
            }
        } else {
            for i in 0..4 {
                let d = i as f32 * std::f32::consts::PI / 2.;
                unsafe {
                    gl::Begin(gl::GL_TRIANGLES);
                    screen.set_color_rgb(COLOR_RGB[0], COLOR_RGB[1], COLOR_RGB[2]);
                    gl::Vertex3f(f32::sin(d) * 0.1, f32::cos(d) * 0.1, 0.4);
                    gl::Vertex3f(f32::sin(d) * 0.3, f32::cos(d) * 0.3, 1.0);
                    screen.set_color_rgb(
                        COLOR_RGB[0] * 0.2,
                        COLOR_RGB[1] * 0.2,
                        COLOR_RGB[2] * 0.2,
                    );
                    gl::Vertex3f(f32::sin(d) * 0.5, f32::cos(d) * 0.5, -1.4);
                    gl::End();
                    screen.set_color_rgb(COLOR_RGB[0], COLOR_RGB[1], COLOR_RGB[2]);
                    gl::Begin(gl::GL_LINE_LOOP);
                    gl::Vertex3f(f32::sin(d) * 0.1, f32::cos(d) * 0.1, 0.4);
                    gl::Vertex3f(f32::sin(d) * 0.3, f32::cos(d) * 0.3, 1.0);
                    gl::Vertex3f(f32::sin(d) * 0.5, f32::cos(d) * 0.5, -1.4);
                    gl::End();
                }
            }
        }
        display_list.end_list();
        ShotShape {
            collision: Vector::new_at(0.15, 0.3),
            display_list,
        }
    }
}

impl Drawable for ShotShape {
    fn draw(&self) {
        self.display_list.call(0);
    }
}
