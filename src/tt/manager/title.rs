use piston::input::RenderArgs;

use crate::gl;
use crate::glu;

use crate::util::display_list::DisplayList;
use crate::util::texture::Texture;
use crate::util::vector::{Vector, Vector3};

use crate::tt::errors::GameError;
use crate::tt::manager::MoveAction;
use crate::tt::pad::{PadButtons, PadDirection};
use crate::tt::screen::Screen;
use crate::tt::ship;
use crate::tt::ActionParams;

use super::Manager;

const REPLAY_CHANGE_DURATION: u32 = 30;
const AUTO_REPEAT_START_TIME: u32 = 30;
const AUTO_REPEAT_CNT: u32 = 5;

pub struct TitleManager {
    display_list: DisplayList,
    title_texture: Texture,
    cnt: u32,
    grade: u32,
    level: u32,
    dir_pressed: bool,
    btn_pressed: bool,
    key_repeat_cnt: u32,
    replay_cnt: u32,
    replay_mode: bool,
    replay_change_ratio: f32,
}

impl TitleManager {
    pub fn new(screen: &Screen) -> Result<Self, GameError> {
        Ok(TitleManager {
            display_list: TitleManager::create_torus_shape(screen),
            title_texture: Texture::create("title.bmp")?,
            cnt: 0,
            grade: 0,
            level: 1,
            dir_pressed: true,
            btn_pressed: true,
            key_repeat_cnt: 0,
            replay_cnt: 0,
            replay_mode: false,
            replay_change_ratio: 0.,
        })
    }

    pub fn mov(&mut self, has_replay_data: bool, params: &mut ActionParams) -> MoveAction {
        let pref_manager = &mut params.pref_manager;
        let dir = params.pad.get_direction();
        if !self.replay_mode {
            if dir & (PadDirection::RIGHT | PadDirection::LEFT) != PadDirection::NONE {
                if !self.dir_pressed {
                    self.dir_pressed = true;
                    let old_max_level = pref_manager.max_level(self.grade);
                    if dir & PadDirection::RIGHT != PadDirection::NONE {
                        self.grade = (self.grade + 1) % ship::GRADE_NUM as u32;
                    }
                    if dir & PadDirection::LEFT != PadDirection::NONE {
                        self.grade =
                            (self.grade + ship::GRADE_NUM as u32 - 1) % ship::GRADE_NUM as u32;
                    }
                    let max_level = pref_manager.max_level(self.grade);
                    if self.level == old_max_level || self.level > max_level {
                        self.level = max_level;
                    }
                }
            }
            if dir & (PadDirection::UP | PadDirection::DOWN) != PadDirection::NONE {
                let mut mv: u32 = 0;
                if !self.dir_pressed {
                    self.dir_pressed = true;
                    mv = 1;
                } else {
                    self.key_repeat_cnt += 1;
                    if self.key_repeat_cnt >= AUTO_REPEAT_START_TIME {
                        if self.key_repeat_cnt % AUTO_REPEAT_CNT == 0 {
                            mv = (self.key_repeat_cnt / AUTO_REPEAT_START_TIME)
                                * (self.key_repeat_cnt / AUTO_REPEAT_START_TIME);
                        }
                    }
                }
                let max_level = pref_manager.max_level(self.grade);
                if dir & PadDirection::DOWN != PadDirection::NONE {
                    self.level += mv;
                    if self.level > max_level {
                        self.level = match self.key_repeat_cnt >= AUTO_REPEAT_START_TIME {
                            true => max_level,
                            false => 1,
                        };
                        self.key_repeat_cnt = 0;
                    }
                }
                if dir & PadDirection::UP != PadDirection::NONE {
                    self.level -= mv;
                    if self.level < 1 {
                        self.level = match self.key_repeat_cnt >= AUTO_REPEAT_START_TIME {
                            true => 1,
                            false => max_level,
                        };
                        self.key_repeat_cnt = 0;
                    }
                }
            }
        } else {
            if dir & (PadDirection::RIGHT | PadDirection::LEFT) != PadDirection::NONE {
                if !self.dir_pressed {
                    self.dir_pressed = true;
                    if dir & PadDirection::RIGHT != PadDirection::NONE {
                        params.ship.camera_mode(false);
                    }
                    if dir & PadDirection::LEFT != PadDirection::NONE {
                        params.ship.camera_mode(true);
                    }
                }
            }
            if dir & (PadDirection::UP | PadDirection::DOWN) != PadDirection::NONE {
                if !self.dir_pressed {
                    self.dir_pressed = true;
                    if dir & PadDirection::UP != PadDirection::NONE {
                        params.ship.draw_front_mode(true);
                    }
                    if dir & PadDirection::DOWN != PadDirection::NONE {
                        params.ship.draw_front_mode(false);
                    }
                }
            }
        }
        if dir == PadDirection::NONE {
            self.dir_pressed = false;
            self.key_repeat_cnt = 0;
        }
        let btn = params.pad.get_buttons();
        let mut action = MoveAction::None;
        if btn & PadButtons::ANY != PadButtons::NONE {
            if !self.btn_pressed {
                self.btn_pressed = true;
                if btn & PadButtons::A != PadButtons::NONE {
                    if !self.replay_mode {
                        pref_manager.record_start_game(self.grade, self.level);
                        action = MoveAction::StartInGame;
                    }
                }
                if has_replay_data && (btn & PadButtons::B != PadButtons::NONE) {
                    self.replay_mode = !self.replay_mode;
                }
            }
        } else {
            self.btn_pressed = false;
        }
        self.cnt += 1;
        if self.replay_mode {
            if self.replay_cnt < REPLAY_CHANGE_DURATION {
                self.replay_cnt += 1;
            }
        } else {
            if self.replay_cnt > 0 {
                self.replay_cnt -= 1;
            }
        }
        self.replay_change_ratio = self.replay_cnt as f32 / REPLAY_CHANGE_DURATION as f32;
        action
    }

    fn create_torus_shape(screen: &Screen) -> DisplayList {
        let mut cp = Vector3::default();
        let mut ring_ofs = Vector3::default();
        let torus_rad = 5.;
        let ring_rad = 0.7;
        let mut display_list = DisplayList::new(3);
        display_list.new_list();
        let mut d1 = 0.;
        for _i in 0..32 {
            let mut d2 = 0.;
            for _j in 0..16 {
                cp.x = f32::sin(d1) * torus_rad;
                cp.y = f32::cos(d1) * torus_rad;
                unsafe {
                    gl::Begin(gl::GL_LINE_STRIP);
                }
                TitleManager::create_ring_offset(&mut ring_ofs, &cp, ring_rad, d1, d2);
                ring_ofs.gl_vertex();
                TitleManager::create_ring_offset(
                    &mut ring_ofs,
                    &cp,
                    ring_rad,
                    d1,
                    d2 + std::f32::consts::PI * 2. / 16.,
                );
                ring_ofs.gl_vertex();
                cp.x = f32::sin(d1 + std::f32::consts::PI * 2. / 32.) * torus_rad;
                cp.y = f32::cos(d1 + std::f32::consts::PI * 2. / 32.) * torus_rad;
                TitleManager::create_ring_offset(
                    &mut ring_ofs,
                    &cp,
                    ring_rad,
                    d1 + std::f32::consts::PI * 2. / 32.,
                    d2 + std::f32::consts::PI * 2. / 16.,
                );
                ring_ofs.gl_vertex();
                unsafe {
                    gl::End();
                }
                d2 += std::f32::consts::PI * 2. / 16.;
            }
            d1 += std::f32::consts::PI * 2. / 32.;
        }
        display_list.end_list();
        display_list.new_list();
        d1 = 0.;
        unsafe {
            gl::Begin(gl::GL_QUADS);
        }
        for _i in 0..32 {
            cp.x = f32::sin(d1) * (torus_rad + ring_rad);
            cp.y = f32::cos(d1) * (torus_rad + ring_rad);
            cp.gl_vertex();;
            cp.x = f32::sin(d1) * (torus_rad + ring_rad * 10.);
            cp.y = f32::cos(d1) * (torus_rad + ring_rad * 10.);
            cp.gl_vertex();
            cp.x = f32::sin(d1 + std::f32::consts::PI * 2. / 32.) * (torus_rad + ring_rad * 10.);
            cp.y = f32::cos(d1 + std::f32::consts::PI * 2. / 32.) * (torus_rad + ring_rad * 10.);
            cp.gl_vertex();
            cp.x = f32::sin(d1 + std::f32::consts::PI * 2. / 32.) * (torus_rad + ring_rad);
            cp.y = f32::cos(d1 + std::f32::consts::PI * 2. / 32.) * (torus_rad + ring_rad);
            cp.gl_vertex();
            d1 += std::f32::consts::PI * 2. / 32.;
        }
        d1 = 0.;
        for _i in 0..32 {
            let mut d2 = 0.;
            for _i in 0..16 {
                cp.x = f32::sin(d1) * torus_rad;
                cp.y = f32::cos(d1) * torus_rad;
                TitleManager::create_ring_offset(&mut ring_ofs, &cp, ring_rad, d1, d2);
                ring_ofs.gl_vertex();
                TitleManager::create_ring_offset(
                    &mut ring_ofs,
                    &cp,
                    ring_rad,
                    d1,
                    d2 + std::f32::consts::PI * 2. / 16.,
                );
                ring_ofs.gl_vertex();
                cp.x = f32::sin(d1 + std::f32::consts::PI * 2. / 32.) * torus_rad;
                cp.y = f32::cos(d1 + std::f32::consts::PI * 2. / 32.) * torus_rad;
                TitleManager::create_ring_offset(
                    &mut ring_ofs,
                    &cp,
                    ring_rad,
                    d1 + std::f32::consts::PI * 2. / 32.,
                    d2 + std::f32::consts::PI * 2. / 16.,
                );
                ring_ofs.gl_vertex();
                TitleManager::create_ring_offset(
                    &mut ring_ofs,
                    &cp,
                    ring_rad,
                    d1 + std::f32::consts::PI * 2. / 32.,
                    d2,
                );
                ring_ofs.gl_vertex();
                d2 += std::f32::consts::PI * 2. / 16.;
            }
            d1 += std::f32::consts::PI * 2. / 32.;
        }
        unsafe {
            gl::End();
        }
        display_list.end_list();
        display_list.new_list();
        d1 = 0.;
        screen.set_color_rgb(1., 1., 1.);
        unsafe {
            gl::Begin(gl::GL_LINE_LOOP);
        }
        for _i in 0..128 {
            cp.x = f32::sin(d1);
            cp.y = f32::cos(d1);
            cp.gl_vertex();
            d1 += std::f32::consts::PI * 2. / 128.;
        }
        unsafe {
            gl::End();
        }
        screen.set_color_rgba(1., 1., 1., 0.3);
        unsafe {
            gl::Begin(gl::GL_TRIANGLE_FAN);
            gl::Vertex3f(0., 0., 0.);
        }
        for _i in 0..129 {
            cp.x = f32::sin(d1);
            cp.y = f32::cos(d1);
            cp.gl_vertex();
            d1 += std::f32::consts::PI * 2. / 128.;
        }
        unsafe {
            gl::End();
        }
        display_list.end_list();
        display_list
    }

    fn calc_cursor_pos(gd: usize, lv: u32) -> Vector {
        let mut x = 460. + gd as f32 * 70.;
        let mut y = 90.;
        if lv > 1 {
            y += 30. + lv as f32;
            x -= lv as f32 * 0.33;
        }
        Vector::new_at(x, y)
    }

    fn draw_cursor_ring(&self, pos: Vector, s: f32) {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(pos.x, pos.y, 0.);
            gl::Rotatef(-20., 0., 0., 1.);
            gl::Scalef(s * 2., s, 1.);
        }
        self.display_list.call(2);
        unsafe {
            gl::PopMatrix();
        }
    }

    fn create_ring_offset(
        ring_ofs: &mut Vector3,
        center_pos: &Vector3,
        rad: f32,
        d1: f32,
        d2: f32,
    ) {
        ring_ofs.x = 0.;
        ring_ofs.y = rad;
        ring_ofs.z = 0.;
        ring_ofs.roll_x(d2);
        ring_ofs.roll_z(-d1);
        *ring_ofs += *center_pos;
    }
}

impl Manager for TitleManager {
    fn start(&mut self, seed: u64, params: &mut ActionParams) {
        let pref_manager = &params.pref_manager;
        self.cnt = 0;
        self.grade = pref_manager.selected_grade();
        self.level = pref_manager.selected_level();
        self.key_repeat_cnt = 0;
        self.dir_pressed = true;
        self.btn_pressed = true;
        self.replay_cnt = 0;
        self.replay_mode = false;
    }

    fn draw(&self, params: &mut ActionParams, _render_args: &RenderArgs) {
        /* TODO
        if (_replayChangeRatio >= 1.0f)
            return;
            */
        unsafe {
            gl::PopMatrix();
        }
        Screen::view_ortho_fixed();
        unsafe {
            gl::Disable(gl::GL_BLEND);
        }
        let screen = &params.screen;
        screen.set_color_rgb(0., 0., 0.);
        let mut rcr = self.replay_change_ratio * 2.;
        if rcr > 1. {
            rcr = 1.;
        }
        unsafe {
            gl::Begin(gl::GL_QUADS);
            gl::Vertex3f(450. + (640. - 450.) * rcr, 0., 0.);
            gl::Vertex3f(640., 0., 0.);
            gl::Vertex3f(640., 480., 0.);
            gl::Vertex3f(450. + (640. - 450.) * rcr, 480., 0.);
            gl::End();
            gl::Enable(gl::GL_BLEND);
        }
        Screen::view_perspective();
        unsafe {
            gl::PushMatrix();
        }
        glu::look_at(0., 0., -1., 0., 0., 0., 0., 1., 0.);
        unsafe {
            gl::PushMatrix();
            gl::Translatef(
                3. - self.replay_change_ratio * 2.4,
                1.8,
                3.5 - self.replay_change_ratio * 1.5,
            );
            gl::Rotatef(30., 1., 0., 0.);
            gl::Rotatef(f32::sin(self.cnt as f32 * 0.005) * 12., 0., 1., 0.);
            gl::Rotatef(self.cnt as f32 * 0.2, 0., 0., 1.);
            gl::Disable(gl::GL_BLEND);
        }
        screen.set_color_rgb(0., 0., 0.);
        self.display_list.call(1);
        unsafe {
            gl::Enable(gl::GL_BLEND);
        }
        screen.set_color_rgba(1., 1., 1., 0.5);
        self.display_list.call(0);
        unsafe {
            gl::PopMatrix();
        }
    }

    fn draw_front(&self, params: &ActionParams, _render_args: &RenderArgs) {
        /*TODO
          if (_replayChangeRatio > 0)
        return;
          */
        unsafe {
            gl::PushMatrix();
            gl::Translatef(508., 400., 0.);
            gl::Rotatef(-20., 0., 0., 1.);
            gl::Scalef(128., 64., 1.);
            gl::LineWidth(2.);
        }
        self.display_list.call(2);
        unsafe {
            gl::LineWidth(1.);
            gl::PopMatrix();
        }
        let screen = &params.screen;
        screen.set_color_rgb(1., 1., 1.);
        unsafe {
            gl::Enable(gl::GL_TEXTURE_2D);
        }
        self.title_texture.bind();
        unsafe {
            gl::Begin(gl::GL_TRIANGLE_FAN);
            gl::TexCoord2f(0., 0.);
            gl::Vertex3f(470., 380., 0.);
            gl::TexCoord2f(1., 0.);
            gl::Vertex3f(598., 380., 0.);
            gl::TexCoord2f(1., 1.);
            gl::Vertex3f(598., 428., 0.);
            gl::TexCoord2f(0., 1.);
            gl::Vertex3f(470., 428., 0.);
            gl::End();
            gl::Disable(gl::GL_TEXTURE_2D);
        }
        let letter = params.letter;
        for i in 0..ship::GRADE_NUM {
            unsafe {
                gl::LineWidth(2.);
            }
            let cursor_pos = TitleManager::calc_cursor_pos(i, 1);
            self.draw_cursor_ring(cursor_pos, 15.);
            letter.draw_string(
                ship::GRADE_LETTER[i],
                cursor_pos.x - 4.,
                cursor_pos.y - 10.,
                7.,
            );
            unsafe {
                gl::LineWidth(1.);
            }
            let ml = params.pref_manager.max_level(i as u32);
            if ml > 1 {
                let e_cursor_pos = TitleManager::calc_cursor_pos(i, ml);
                self.draw_cursor_ring(e_cursor_pos, 15.);
                letter.draw_num(ml as usize, e_cursor_pos.x + 7., e_cursor_pos.y - 8., 6.);
                let l2_cursor_pos = TitleManager::calc_cursor_pos(i, 2);
                unsafe {
                    gl::Begin(gl::GL_LINES);
                    gl::Vertex3f(cursor_pos.x - 29., cursor_pos.y + 7., 0.);
                    gl::Vertex3f(l2_cursor_pos.x - 29., l2_cursor_pos.y + 7., 0.);
                    gl::Vertex3f(l2_cursor_pos.x - 29., l2_cursor_pos.y + 7., 0.);
                    gl::Vertex3f(e_cursor_pos.x - 29., e_cursor_pos.y + 7., 0.);
                    gl::Vertex3f(cursor_pos.x + 29., cursor_pos.y - 7., 0.);
                    gl::Vertex3f(l2_cursor_pos.x + 29., l2_cursor_pos.y - 7., 0.);
                    gl::Vertex3f(l2_cursor_pos.x + 29., l2_cursor_pos.y - 7., 0.);
                    gl::Vertex3f(e_cursor_pos.x + 29., e_cursor_pos.y - 7., 0.);
                    gl::End();
                }
            }
        }
        let grade_str = ship::GRADE_STR[self.grade as usize];
        letter.draw_string(grade_str, 560. - grade_str.len() as f32 * 19., 4., 9.);
        letter.draw_num(self.level as usize, 620., 10., 6.);
        letter.draw_string("LV", 570., 10., 6.);
        let gd = params.pref_manager.grade_data(self.grade);
        letter.draw_num(gd.hi_score as usize, 620., 45., 8.);
        letter.draw_num(gd.start_level as usize, 408., 54., 5.);
        letter.draw_num(gd.end_level as usize, 453., 54., 5.);
        letter.draw_string("-", 423., 54., 5.);
        let cursor_pos = TitleManager::calc_cursor_pos(self.grade as usize, self.level);
        self.draw_cursor_ring(cursor_pos, 18. + f32::sin(self.cnt as f32 * 0.1) * 3.);
    }
}
