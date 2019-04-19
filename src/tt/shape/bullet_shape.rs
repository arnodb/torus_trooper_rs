use crate::gl;

use crate::tt::screen::Screen;
use crate::util::color::Color;
use crate::util::display_list::DisplayList;
use crate::util::vector::Vector3;

use super::Drawable;

const COLOR_RGB: (f32, f32, f32) = (1., 0.7, 0.8);

const SQUARE_POINT_DAT: [[(f32, f32, f32); 4]; 6] = [
    [(-1., -1., 1.), (1., -1., 1.), (1., 1., 1.), (-1., 1., 1.)],
    [
        (-1., -1., -1.),
        (1., -1., -1.),
        (1., 1., -1.),
        (-1., 1., -1.),
    ],
    [(-1., 1., -1.), (1., 1., -1.), (1., 1., 1.), (-1., 1., 1.)],
    [
        (-1., -1., -1.),
        (1., -1., -1.),
        (1., -1., 1.),
        (-1., -1., 1.),
    ],
    [(1., -1., -1.), (1., -1., 1.), (1., 1., 1.), (1., 1., -1.)],
    [
        (-1., -1., -1.),
        (-1., -1., 1.),
        (-1., 1., 1.),
        (-1., 1., -1.),
    ],
];

const BAR_POINT_DAT: [[(f32, f32, f32); 4]; 5] = [
    [(-1., -1., 1.), (1., -1., 1.), (1., 1., 1.), (-1., 1., 1.)],
    [(-1., 1., -1.), (1., 1., -1.), (1., 1., 1.), (-1., 1., 1.)],
    [
        (-1., -1., -1.),
        (1., -1., -1.),
        (1., -1., 1.),
        (-1., -1., 1.),
    ],
    [(1., -1., -1.), (1., -1., 1.), (1., 1., 1.), (1., 1., -1.)],
    [
        (-1., -1., -1.),
        (-1., -1., 1.),
        (-1., 1., 1.),
        (-1., 1., -1.),
    ],
];

pub struct BulletShape {
    display_list: DisplayList,
}

impl BulletShape {
    pub fn new_square(wire_shape: bool, screen: &Screen) -> Self {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        let mut p = [Vector3::default(); 4];
        let mut np = [Vector3::default(); 4];
        for square_point_dat in &SQUARE_POINT_DAT {
            let mut cp = Vector3::default();
            for j in 0..4 {
                p[j] = Vector3::new_at_tuple(square_point_dat[j]);
                cp += p[j];
            }
            cp /= 4.;
            for j in 0..4 {
                np[j] = Vector3::blend(p[j], cp, 0.6);
            }
            if !wire_shape {
                screen.set_color(COLOR_RGB);
            } else {
                screen.set_color((COLOR_RGB.0 * 0.6, COLOR_RGB.1, COLOR_RGB.2));
            }
            unsafe {
                gl::Begin(gl::GL_LINE_LOOP);
            }
            for np_item in &np {
                np_item.gl_vertex();
            }
            unsafe {
                gl::End();
            }
            if !wire_shape {
                unsafe {
                    gl::Begin(gl::GL_TRIANGLE_FAN);
                }
                screen.set_color(Color::from(COLOR_RGB) * 0.7);
                for np_item in &np {
                    np_item.gl_vertex();
                }
                unsafe {
                    gl::End();
                }
            }
        }
        display_list.end_list();
        BulletShape { display_list }
    }

    pub fn new_triangle(wire_shape: bool, screen: &Screen) -> Self {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        for i in 0..3 {
            let d = std::f32::consts::PI * 2. / 3. * i as f32;
            let p1 = Vector3::new_at(0., 0., 2.5);
            let p2 = Vector3::new_at(f32::sin(d) * 1.8, f32::cos(d) * 1.8, -1.2);
            let p3 = Vector3::new_at(
                f32::sin(d + std::f32::consts::PI * 2. / 3.) * 1.2,
                f32::cos(d + std::f32::consts::PI * 2. / 3.) * 1.2,
                -1.2,
            );
            let mut cp = Vector3::default();
            cp += p1 + p2 + p3;
            cp /= 3.;
            let np1 = Vector3::blend(p1, cp, 0.6);
            let np2 = Vector3::blend(p2, cp, 0.6);
            let np3 = Vector3::blend(p3, cp, 0.6);
            if !wire_shape {
                screen.set_color(COLOR_RGB);
            } else {
                screen.set_color((COLOR_RGB.0 * 0.6, COLOR_RGB.1, COLOR_RGB.2));
            }
            unsafe {
                gl::Begin(gl::GL_LINE_LOOP);
            }
            np1.gl_vertex();
            np2.gl_vertex();
            np3.gl_vertex();
            unsafe {
                gl::End();
            }
            if !wire_shape {
                unsafe {
                    gl::Begin(gl::GL_TRIANGLE_FAN);
                }
                screen.set_color(Color::from(COLOR_RGB) * 0.7);
                np1.gl_vertex();
                screen.set_color(Color::from(COLOR_RGB) * 0.4);
                np2.gl_vertex();
                np3.gl_vertex();
                unsafe {
                    gl::End();
                }
            }
        }
        display_list.end_list();
        BulletShape { display_list }
    }

    pub fn new_bar(wire_shape: bool, screen: &Screen) -> Self {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        let mut p = [Vector3::default(); 4];
        let mut np = [Vector3::default(); 4];
        for bar_point_dat in &BAR_POINT_DAT {
            let mut cp = Vector3::default();
            for j in 0..4 {
                p[j] = Vector3::new_at_tuple(bar_point_dat[j])
                    * Vector3::new_at_tuple((0.7, 0.7, 1.75));
                cp += p[j];
            }
            cp /= 4.;
            for j in 0..4 {
                np[j] = Vector3::blend(p[j], cp, 0.6);
            }
            if !wire_shape {
                screen.set_color(COLOR_RGB);
            } else {
                screen.set_color((COLOR_RGB.0 * 0.6, COLOR_RGB.1, COLOR_RGB.2));
            }
            unsafe {
                gl::Begin(gl::GL_LINE_LOOP);
            }
            for np_item in &np {
                np_item.gl_vertex();
            }
            unsafe {
                gl::End();
            }
            if !wire_shape {
                unsafe {
                    gl::Begin(gl::GL_TRIANGLE_FAN);
                }
                screen.set_color(Color::from(COLOR_RGB) * 0.7);
                for np_item in &np {
                    np_item.gl_vertex();
                }
                unsafe {
                    gl::End();
                }
            }
        }
        display_list.end_list();
        BulletShape { display_list }
    }
}

impl Drawable for BulletShape {
    fn draw(&self) {
        self.display_list.call(0);
    }
}
