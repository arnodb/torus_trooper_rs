use crate::gl;

use crate::tt::screen::Screen;
use crate::util::display_list::DisplayList;

use super::Drawable;

const COLOR_RGB: (f32, f32, f32) = (1., 0.9, 0.5);

pub struct BitShape {
    display_list: DisplayList,
}

impl BitShape {
    pub fn new(screen: &Screen) -> BitShape {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        for i in 0..4 {
            let mut d = i as f32 * std::f32::consts::PI / 2. + std::f32::consts::PI / 4.;
            unsafe {
                screen.set_color(COLOR_RGB);
                gl::Begin(gl::GL_LINE_LOOP);
                gl::Vertex3f(f32::sin(d - 0.3), -0.8, f32::cos(d - 0.3));
                gl::Vertex3f(f32::sin(d + 0.3), -0.8, f32::cos(d + 0.3));
                gl::Vertex3f(f32::sin(d + 0.3), 0.8, f32::cos(d + 0.3));
                gl::Vertex3f(f32::sin(d - 0.3), 0.8, f32::cos(d - 0.3));
                gl::End();
                d += std::f32::consts::PI / 4.;
                gl::Begin(gl::GL_LINE_LOOP);
                gl::Vertex3f(f32::sin(d - 0.3) * 2., -0.2, f32::cos(d - 0.3) * 2.);
                gl::Vertex3f(f32::sin(d + 0.3) * 2., -0.2, f32::cos(d + 0.3) * 2.);
                gl::Vertex3f(f32::sin(d + 0.3) * 2., 0.2, f32::cos(d + 0.3) * 2.);
                gl::Vertex3f(f32::sin(d - 0.3) * 2., 0.2, f32::cos(d - 0.3) * 2.);
                gl::End();
                d -= std::f32::consts::PI / 4.;
                gl::Begin(gl::GL_TRIANGLE_FAN);
                gl::Vertex3f(f32::sin(d - 0.3), -0.8, f32::cos(d - 0.3));
                gl::Vertex3f(f32::sin(d + 0.3), -0.8, f32::cos(d + 0.3));
                gl::Vertex3f(f32::sin(d + 0.3), 0.8, f32::cos(d + 0.3));
                gl::Vertex3f(f32::sin(d - 0.3), 0.8, f32::cos(d - 0.3));
                gl::End();
                d += std::f32::consts::PI / 4.;
                gl::Begin(gl::GL_TRIANGLE_FAN);
                gl::Vertex3f(f32::sin(d - 0.3) * 2., -0.2, f32::cos(d - 0.3) * 2.);
                gl::Vertex3f(f32::sin(d + 0.3) * 2., -0.2, f32::cos(d + 0.3) * 2.);
                gl::Vertex3f(f32::sin(d + 0.3) * 2., 0.2, f32::cos(d + 0.3) * 2.);
                gl::Vertex3f(f32::sin(d - 0.3) * 2., 0.2, f32::cos(d - 0.3) * 2.);
                gl::End();
            }
        }
        display_list.end_list();
        BitShape { display_list }
    }
}

impl Drawable for BitShape {
    fn draw(&self) {
        self.display_list.call(0);
    }
}
