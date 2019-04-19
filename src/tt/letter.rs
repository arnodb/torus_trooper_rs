const LETTER_WIDTH: f32 = 2.1;
const LETTER_HEIGHT: f32 = 3.0;
const COLOR_NUM: usize = 4;
const COLOR_RGB: [(f32, f32, f32); 2] = [(1., 1., 1.), (0.9, 0.7, 0.5)];
const LETTER_NUM: usize = 44;
const DISPLAY_LIST_NUM: usize = LETTER_NUM * COLOR_NUM;

use crate::gl;

use crate::tt::screen::Screen;
use crate::util::color::Color;
use crate::util::display_list::DisplayList;

pub struct Letter {
    display_list: DisplayList,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
    ToRight,
    ToDown,
    ToLeft,
    ToUp,
}

impl Letter {
    pub fn new(screen: &Screen) -> Self {
        let mut display_list = DisplayList::new(DISPLAY_LIST_NUM as u32);
        for j in 0..COLOR_NUM {
            for i in 0..LETTER_NUM {
                display_list.new_list();
                Letter::draw_letter_internal(i, j, screen);
                display_list.end_list();
            }
        }
        Letter { display_list }
    }

    fn draw_letter(&self, letter: usize, x: f32, y: f32, scale: f32, deg: f32, color: usize) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(x, y, 0.);
            gl::Scalef(scale, scale, scale);
            gl::Rotatef(deg, 0., 0., 1.);
        }
        self.display_list.call((letter + color * LETTER_NUM) as u32);
        unsafe {
            gl::PopMatrix();
        }
    }

    fn draw_letter_rev(&self, letter: usize, x: f32, y: f32, scale: f32, deg: f32, color: usize) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(x, y, 0.);
            gl::Scalef(scale, -scale, scale);
            gl::Rotatef(deg, 0., 0., 1.);
        }
        self.display_list.call((letter + color * LETTER_NUM) as u32);
        unsafe {
            gl::PopMatrix();
        }
    }

    fn convert_char_to_int(c: char) -> usize {
        (match c {
            '0'...'9' => c as u8 - b'0',
            'A'...'Z' => c as u8 - b'A' + 10,
            'a'...'z' => c as u8 - b'a' + 10,
            '.' => 36,
            '-' => 38,
            '+' => 39,
            '_' => 37,
            '!' => 42,
            '/' => 43,
            _ => panic!("Unexpected character {}", c as u8),
        }) as usize
    }

    pub fn draw_string(&self, str: &str, lx: f32, y: f32, scale: f32) {
        self.draw_string_color(str, lx, y, scale, Direction::ToRight, 0)
    }

    pub fn draw_string_color(
        &self,
        str: &str,
        lx: f32,
        y: f32,
        scale: f32,
        d: Direction,
        color: usize,
    ) {
        self.draw_string_full(str, lx, y, scale, d, color, false, 0.)
    }

    pub fn draw_string_full(
        &self,
        str: &str,
        lx: f32,
        y: f32,
        scale: f32,
        d: Direction,
        color: usize,
        rev: bool,
        odeg: f32,
    ) {
        let mut mut_x = lx + LETTER_WIDTH * scale / 2.;
        let mut mut_y = y + LETTER_HEIGHT * scale / 2.;
        let ldeg: f32 = match d {
            Direction::ToRight => 0.,
            Direction::ToDown => 90.,
            Direction::ToLeft => 180.,
            Direction::ToUp => 270.,
        } + odeg;
        for c in str.chars() {
            if c != ' ' {
                let idx = Letter::convert_char_to_int(c);
                if rev {
                    self.draw_letter_rev(idx, mut_x, mut_y, scale, ldeg, color);
                } else {
                    self.draw_letter(idx, mut_x, mut_y, scale, ldeg, color);
                }
            }
            if odeg == 0. {
                match d {
                    Direction::ToRight => {
                        mut_x += scale * LETTER_WIDTH;
                    }
                    Direction::ToDown => {
                        mut_y += scale * LETTER_WIDTH;
                    }
                    Direction::ToLeft => {
                        mut_x -= scale * LETTER_WIDTH;
                    }
                    Direction::ToUp => {
                        mut_y -= scale * LETTER_WIDTH;
                    }
                }
            } else {
                mut_x += f32::cos(ldeg * std::f32::consts::PI / 180.) * scale * LETTER_WIDTH;
                mut_y += f32::sin(ldeg * std::f32::consts::PI / 180.) * scale * LETTER_WIDTH;
            }
        }
    }

    pub fn draw_num(&self, num: usize, lx: f32, y: f32, scale: f32) {
        self.draw_num_digit(num, lx, y, scale, Direction::ToRight, 0)
    }

    pub fn draw_num_digit(
        &self,
        num: usize,
        lx: f32,
        y: f32,
        scale: f32,
        d: Direction,
        digit: isize,
    ) {
        let mut mut_x = lx + LETTER_WIDTH * scale / 2.;
        let mut mut_y = y + LETTER_HEIGHT * scale / 2.;
        let ld: f32 = match d {
            Direction::ToRight => 0.,
            Direction::ToDown => 90.,
            Direction::ToLeft => 180.,
            Direction::ToUp => 270.,
        };
        let mut n = num;
        let mut dg = digit;
        loop {
            self.draw_letter(n % 10, mut_x, mut_y, scale, ld, 0);
            match d {
                Direction::ToRight => {
                    mut_x -= scale * LETTER_WIDTH;
                }
                Direction::ToDown => {
                    mut_y -= scale * LETTER_WIDTH;
                }
                Direction::ToLeft => {
                    mut_x += scale * LETTER_WIDTH;
                }
                Direction::ToUp => {
                    mut_y += scale * LETTER_WIDTH;
                }
            }
            n /= 10;
            dg -= 1;
            if n <= 0 && dg <= 0 {
                break;
            }
        }
    }

    pub fn draw_time(&self, time: isize, lx: f32, y: f32, scale: f32) {
        self.draw_time_color(time, lx, y, scale, 0)
    }

    pub fn draw_time_color(&self, time: isize, lx: f32, y: f32, scale: f32, color: usize) {
        let mut n: usize = if time >= 0 { time as usize } else { 0 };
        let mut mut_x = lx;
        for i in 0..7 {
            if i != 4 {
                self.draw_letter(n % 10, mut_x, y, scale, 0., color);
                n /= 10;
            } else {
                self.draw_letter(n % 6, mut_x, y, scale, 0., color);
                n /= 6;
            }
            if (i & 1) == 1 || i == 0 {
                match i {
                    3 => self.draw_letter(41, mut_x + scale * 1.16, y, scale, 0., color),
                    5 => self.draw_letter(40, mut_x + scale * 1.16, y, scale, 0., color),
                    _ => (),
                }
                mut_x -= scale * LETTER_WIDTH;
            } else {
                mut_x -= scale * LETTER_WIDTH * 1.3;
            }
            if n <= 0 {
                break;
            }
        }
    }

    fn draw_letter_internal(idx: usize, color: usize, screen: &Screen) {
        let data = &SP_DATA[idx];
        for point in data.iter() {
            let x = point[0];
            let mut y = -point[1];
            let mut size = point[2];
            let mut length = point[3];
            let mut deg = point[4];
            y *= 0.9;
            size *= 1.4;
            length *= 1.05;
            deg %= 180.;
            if color == 2 {
                Letter::draw_box_line(x, y, size, length, deg);
            } else if color == 3 {
                Letter::draw_box_poly(x, y, size, length, deg);
            } else {
                Letter::draw_box(x, y, size, length, deg, COLOR_RGB[color].into(), screen);
            }
        }
    }

    fn draw_box(x: f32, y: f32, width: f32, height: f32, deg: f32, color: Color, screen: &Screen) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(x - width / 2., y - height / 2., 0.);
            gl::Rotatef(deg, 0., 0., 1.);
        }
        screen.set_alpha_color(color.with_alpha(0.5));
        unsafe {
            gl::Begin(gl::GL_TRIANGLE_FAN);
        }
        Letter::draw_box_part(width, height);
        unsafe {
            gl::End();
        }
        screen.set_color(color);
        unsafe {
            gl::Begin(gl::GL_LINE_LOOP);
        }
        Letter::draw_box_part(width, height);
        unsafe {
            gl::End();
            gl::PopMatrix();
        }
    }

    fn draw_box_line(x: f32, y: f32, width: f32, height: f32, deg: f32) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(x - width / 2., y - height / 2., 0.);
            gl::Rotatef(deg, 0., 0., 1.);
            gl::Begin(gl::GL_LINE_LOOP);
        }
        Letter::draw_box_part(width, height);
        unsafe {
            gl::End();
            gl::PopMatrix();
        }
    }

    fn draw_box_poly(x: f32, y: f32, width: f32, height: f32, deg: f32) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(x - width / 2., y - height / 2., 0.);
            gl::Rotatef(deg, 0., 0., 1.);
            gl::Begin(gl::GL_TRIANGLE_FAN);
            Letter::draw_box_part(width, height);
            gl::End();
            gl::PopMatrix();
        }
    }

    fn draw_box_part(width: f32, height: f32) {
        unsafe {
            gl::Vertex3f(-width / 2., 0., 0.);
            gl::Vertex3f(-width / 3. * 1., -height / 2., 0.);
            gl::Vertex3f(width / 3. * 1., -height / 2., 0.);
            gl::Vertex3f(width / 2., 0., 0.);
            gl::Vertex3f(width / 3. * 1., height / 2., 0.);
            gl::Vertex3f(-width / 3. * 1., height / 2., 0.);
        }
    }
}

lazy_static! {
    static ref SP_DATA: Vec<Vec<[f32; 5]>> = vec![
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.6, 0.55, 0.65, 0.3, 90.], [0.6, 0.55, 0.65, 0.3, 90.],
            [-0.6, -0.55, 0.65, 0.3, 90.], [0.6, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0.5, 0.55, 0.65, 0.3, 90.],
            [0.5, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [0.65, 0.55, 0.65, 0.3, 90.],
            [0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            //A
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [-0.18, 1.15, 0.45, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.45, 0.55, 0.65, 0.3, 90.],
            [-0.18, 0., 0.45, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [-0.15, 1.15, 0.45, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.45, 0.45, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            //F
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [0.05, 0., 0.3, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 0.55, 0.65, 0.3, 90.],
            [0., -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0.65, 0.55, 0.65, 0.3, 90.],
            [0.65, -0.55, 0.65, 0.3, 90.], [-0.7, -0.7, 0.3, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            //K
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.4, 0.55, 0.65, 0.3, 100.],
            [-0.25, 0., 0.45, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.6, -0.55, 0.65, 0.3, 80.],
        ],
        vec![
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [-0.5, 1.15, 0.3, 0.3, 0.], [0.1, 1.15, 0.3, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., 0.55, 0.65, 0.3, 90.],
            [0., -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            //P
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
            [0.05, -0.55, 0.45, 0.3, 60.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.2, 0., 0.45, 0.3, 0.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.45, -0.55, 0.65, 0.3, 80.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [-0.65, 0.55, 0.65, 0.3, 90.],
            [0., 0., 0.65, 0.3, 0.],
            [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [-0.5, 1.15, 0.55, 0.3, 0.], [0.5, 1.15, 0.55, 0.3, 0.],
            [0.1, 0.55, 0.65, 0.3, 90.],
            [0.1, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            //U
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.5, -0.55, 0.65, 0.3, 90.], [0.5, -0.55, 0.65, 0.3, 90.],
            [-0.1, -1.15, 0.45, 0.3, 0.],
        ],
        vec![
            [-0.65, 0.55, 0.65, 0.3, 90.], [0.65, 0.55, 0.65, 0.3, 90.],
            [-0.65, -0.55, 0.65, 0.3, 90.], [0.65, -0.55, 0.65, 0.3, 90.],
            [-0.5, -1.15, 0.3, 0.3, 0.], [0.1, -1.15, 0.3, 0.3, 0.],
            [0., 0.55, 0.65, 0.3, 90.],
            [0., -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [-0.4, 0.6, 0.85, 0.3, 360.-120.],
            [0.4, 0.6, 0.85, 0.3, 360.-60.],
            [-0.4, -0.6, 0.85, 0.3, 360.-240.],
            [0.4, -0.6, 0.85, 0.3, 360.-300.],
        ],
        vec![
            [-0.4, 0.6, 0.85, 0.3, 360.-120.],
            [0.4, 0.6, 0.85, 0.3, 360.-60.],
            [-0.1, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            [0., 1.15, 0.65, 0.3, 0.],
            [0.3, 0.4, 0.65, 0.3, 120.],
            [-0.3, -0.4, 0.65, 0.3, 120.],
            [0., -1.15, 0.65, 0.3, 0.],
        ],
        vec![
            //.
            [0., -1.15, 0.3, 0.3, 0.],
        ],
        vec![
            //_
            [0., -1.15, 0.8, 0.3, 0.],
        ],
        vec![
            //-
            [0., 0., 0.9, 0.3, 0.],
        ],
        vec![
            //+
            [-0.5, 0., 0.45, 0.3, 0.], [0.45, 0., 0.45, 0.3, 0.],
            [0.1, 0.55, 0.65, 0.3, 90.],
            [0.1, -0.55, 0.65, 0.3, 90.],
        ],
        vec![
            //'
            [0., 1.0, 0.4, 0.2, 90.],
        ],
        vec![
            //''
            [-0.19, 1.0, 0.4, 0.2, 90.],
            [0.2, 1.0, 0.4, 0.2, 90.],
        ],
        vec![
            // !
            [0.56, 0.25, 1.1, 0.3, 90.],
            [0., -1.0, 0.3, 0.3, 90.],
        ],
        vec![
            // /
            [0.8, 0., 1.75, 0.3, 120.],
        ]
    ];
}
