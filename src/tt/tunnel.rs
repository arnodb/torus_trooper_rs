use std::fmt;

use crate::gl;

use crate::tt::screen::Screen;
use crate::tt::ship::Ship;
use crate::util::color::Color;
use crate::util::display_list::DisplayList;
use crate::util::rand::Rand;
use crate::util::vector::{Vector, Vector3};

const DEPTH_NUM: usize = 72;
const SHIP_IDX_OFS: usize = 5;
const RAD_RATIO: f32 = 1.05;

const DEPTH_CHANGE_RATIO: f32 = 1.15;
const DEPTH_RATIO_MAX: f32 = 80.;

pub struct Tunnel {
    torus: Torus,
    ship_idx: usize,
    ship_ofs: f32,
    ship_y: f32,
    torus_idx: usize,
    point_from: f32,
    slices: TunnelSlices,
    slices_backward: TunnelSlices,
    rings_state: Vec<u32>,
}

struct TunnelSlices {
    slices: Vec<Slice>,
    // TODO use vectors
    ring_indexes: [Option<usize>; DEPTH_NUM],
}

impl Tunnel {
    pub fn new(torus: Torus) -> Self {
        let rings_state = vec![0; torus.rings.len()];
        Tunnel {
            torus,
            ship_idx: 0,
            ship_ofs: 0.,
            ship_y: 0.,
            torus_idx: 0,
            point_from: 0.,
            slices: TunnelSlices {
                slices: std::iter::repeat(Slice::new()).take(DEPTH_NUM).collect(),
                ring_indexes: [None; DEPTH_NUM],
            },
            slices_backward: TunnelSlices {
                slices: std::iter::repeat(Slice::new()).take(DEPTH_NUM).collect(),
                ring_indexes: [None; DEPTH_NUM],
            },
            rings_state,
        }
    }

    pub fn start(&mut self, torus: Torus) {
        let rings_state = vec![0; torus.rings.len()];
        self.torus = torus;
        self.rings_state = rings_state;
        self.torus_idx = 0;
        self.point_from = 0.;
    }

    pub fn set_slices(&mut self) {
        let mut sight_depth = 0.;
        let mut ti = self.torus_idx as f32;
        let mut dr = 1.;
        let slices = &mut self.slices;
        {
            let first_slice = slices.slices.first_mut();
            if let Some(slice) = first_slice {
                slice.set_first(
                    self.point_from,
                    &self.torus.get_slice_state(self.torus_idx),
                    -(self.ship_idx as f32) - self.ship_ofs,
                );
            };
        }
        let slices_len = slices.slices.len();
        for i in 1..slices_len {
            sight_depth += dr;
            let prev_ti = ti as usize;
            ti += dr;
            if ti >= self.torus.slice_num as f32 {
                ti -= self.torus.slice_num as f32;
            }
            let (state, ring_index) = self.torus.get_slice_state_and_ring(ti as usize, prev_ti);
            slices.ring_indexes[i] = ring_index;
            if let Some(ri) = ring_index {
                self.rings_state[ri] += 1;
            }
            let new_slice = slices.slices[i - 1].set_prev_to(
                &state,
                dr,
                sight_depth - self.ship_idx as f32 - self.ship_ofs,
            );
            slices.slices[i].set(&new_slice);
            if i >= slices_len / 2 && dr < DEPTH_RATIO_MAX {
                dr *= DEPTH_CHANGE_RATIO;
            }
        }
    }

    pub fn set_slices_backward(&mut self) {
        let mut sight_depth = 0.;
        let mut ti = self.torus_idx as f32;
        let mut dr = -1.;
        let slices = &mut self.slices_backward;
        {
            let first_slice = slices.slices.first_mut();
            if let Some(slice) = first_slice {
                slice.set_first(
                    self.point_from,
                    &self.torus.get_slice_state(self.torus_idx),
                    -(self.ship_idx as f32) - self.ship_ofs,
                );
            };
        }
        let slices_backward_len = slices.slices.len();
        for i in 1..slices_backward_len {
            sight_depth += dr;
            let prev_ti = ti as usize;
            ti += dr;
            if ti < 0. {
                ti += self.torus.slice_num as f32;
            }
            let (state, ring_index) = self.torus.get_slice_state_and_ring(prev_ti, ti as usize);
            slices.ring_indexes[i] = ring_index;
            if let Some(ri) = ring_index {
                self.rings_state[ri] += 1;
            }
            let new_slice = slices.slices[i - 1].set_prev_to(
                &state,
                dr,
                sight_depth - self.ship_idx as f32 - self.ship_ofs,
            );
            slices.slices[i].set(&new_slice);
            if i >= slices_backward_len / 2 && dr > -DEPTH_RATIO_MAX {
                dr *= DEPTH_CHANGE_RATIO;
            }
        }
    }

    pub fn go_to_next_slice(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.torus_idx += n;
        let slices = &self.slices.slices;
        for i in 0..n {
            self.point_from += slices[i].state.mp;
            self.point_from %= slices[i].state.point_num as f32;
            if self.point_from < 0. {
                self.point_from += slices[i].state.point_num as f32;
            }
        }
        if self.torus_idx >= self.torus.slice_num {
            self.torus_idx -= self.torus.slice_num;
            // That is hard when you reach the end of the torus.
            // Don't do this. -- arnodb
            // self.point_from = 0.;
        }
    }

    pub fn set_ship_pos(&mut self, o: f32, y: f32) {
        self.ship_ofs = o;
        self.ship_y = y;
        self.ship_idx = SHIP_IDX_OFS;
    }

    fn get_pos(&self, d: f32, o: f32, si: usize, rr: f32, slices: &Vec<Slice>) -> Vector3 {
        let nsi = si + 1;
        let r = slices[si].state.rad * (1. - o) + slices[nsi].state.rad * o;
        let d1 = slices[si].d1 * (1. - o) + slices[nsi].d1 * o;
        let d2 = slices[si].d2 * (1. - o) + slices[nsi].d2 * o;
        let mut tpos = Vector3::new_at(0., r * rr, 0.);
        tpos.roll_z(d).roll_y(d1).roll_x(d2);
        tpos += slices[si].center_pos * (1. - o) + slices[nsi].center_pos * o;
        tpos
    }

    pub fn get_pos_v(&self, p: Vector) -> Vector3 {
        if p.y >= -(self.ship_idx as f32) - self.ship_ofs {
            let (si, o) = self.calc_index(p.y);
            self.get_pos(p.x, o, si, 1.0, &self.slices.slices)
        } else {
            let (si, o) = self.calc_index_backward(p.y);
            self.get_pos(p.x, o, si, 1.0, &self.slices_backward.slices)
        }
    }

    pub fn get_pos_v3(&self, p: Vector3) -> Vector3 {
        let (si, o) = self.calc_index(p.y);
        let slices = &self.slices.slices;
        self.get_pos(p.x, o, si, RAD_RATIO - p.z / slices[si].state.rad, slices)
    }

    fn get_center_pos(&self, y: f32) -> (Vector3, f32, f32) {
        let mut mut_y = y - self.ship_y;
        if mut_y < -(self.get_torus_length() as f32) / 2. {
            mut_y += self.get_torus_length() as f32;
        }
        if mut_y >= -(self.ship_idx as f32) - self.ship_ofs {
            let (si, o) = self.calc_index(mut_y);
            let nsi = si + 1;
            let slices = &self.slices.slices;
            (
                slices[si].center_pos * (1. - o) + slices[nsi].center_pos * o,
                slices[si].d1 * (1. - o) + slices[nsi].d1 * o,
                slices[si].d2 * (1. - o) + slices[nsi].d2 * o,
            )
        } else {
            let (si, o) = self.calc_index_backward(mut_y);
            let nsi = si + 1;
            let slices = &self.slices_backward.slices;
            (
                slices[si].center_pos * (1. - o) + slices[nsi].center_pos * o,
                slices[si].d1 * (1. - o) + slices[nsi].d1 * o,
                slices[si].d2 * (1. - o) + slices[nsi].d2 * o,
            )
        }
    }

    pub fn get_slice(&self, y: f32) -> &Slice {
        if y >= -(self.ship_idx as f32) - self.ship_ofs {
            let (si, _o) = self.calc_index(y);
            &self.slices.slices[si]
        } else {
            let (si, _o) = self.calc_index_backward(y);
            &self.slices_backward.slices[si]
        }
    }

    pub fn check_in_course(&self, p: Vector) -> f32 {
        let sl = self.get_slice(p.y);
        if sl.is_nearly_round() {
            return 0.;
        }
        let ld = sl.get_left_edge_deg();
        let rd = sl.get_right_edge_deg();
        let rsl = Tunnel::check_deg_inside(p.x, ld, rd);
        if rsl == 0 {
            0.
        } else {
            let rad = sl.state.rad;
            let mut ofs = if rsl == 1 { p.x - rd } else { ld - p.x };
            if ofs >= std::f32::consts::PI * 2. {
                ofs -= std::f32::consts::PI * 2.;
            } else if ofs < 0. {
                ofs += std::f32::consts::PI * 2.;
            }
            ofs * rad * rsl as f32
        }
    }

    // TODO return -1, 0, 1
    pub fn check_deg_inside(d: f32, ld: f32, rd: f32) -> i32 {
        let mut rsl = 0;
        if rd <= ld {
            if d > rd && d < ld {
                rsl = if d < (rd + ld) / 2. { 1 } else { -1 };
            }
        } else {
            if d < ld || d > rd {
                let mut cd = (ld + rd) / 2. + std::f32::consts::PI;
                if cd >= std::f32::consts::PI * 2. {
                    cd -= std::f32::consts::PI * 2.;
                }
                if cd >= std::f32::consts::PI {
                    rsl = if d < cd && d > rd { 1 } else { -1 };
                } else {
                    rsl = if d > cd && d < ld { -1 } else { 1 };
                }
            }
        }
        rsl
    }

    pub fn get_radius(&self, z: f32) -> f32 {
        let (si, o) = self.calc_index(z);
        let slices = &self.slices.slices;
        slices[si].state.rad * (1.0 - o) + slices[si + 1].state.rad * o
    }

    fn calc_index(&self, z: f32) -> (usize, f32) {
        let slices = &self.slices.slices;
        let mut idx = slices.len() + 99999;
        let mut ofs = 0.;
        for i in 1..slices.len() {
            if z < slices[i].depth {
                idx = i - 1;
                ofs = (z - slices[idx].depth) / (slices[idx + 1].depth - slices[idx].depth);
                break;
            }
        }
        /*XXX if idx < 0 {
            idx = 0;
            ofs = 0.;
        } else */
        if idx >= slices.len() - 1 {
            idx = slices.len() - 2;
            ofs = 0.99;
        }
        if ofs < 0. {
            ofs = 0.;
        } else if ofs >= 1. {
            ofs = 0.99;
        }
        (idx, ofs)
    }

    fn calc_index_backward(&self, z: f32) -> (usize, f32) {
        let slices = &self.slices_backward.slices;
        let mut idx = slices.len() + 99999;
        let mut ofs = 0.;
        for i in 1..slices.len() {
            if z > slices[i].depth {
                idx = i - 1;
                ofs = (z - (slices[idx].depth) - z) / (slices[idx + 1].depth - slices[idx].depth);
                break;
            }
        }
        /*XXX if idx < 0 {
            idx = 0;
            ofs = 0.;
        } else */
        if idx >= slices.len() - 1 {
            idx = slices.len() - 2;
            ofs = 0.99;
        }
        if ofs < 0. {
            ofs = 0.;
        } else if ofs >= 1. {
            ofs = 0.99;
        }
        (idx, ofs)
    }

    pub fn check_in_screen(&self, p: Vector, ship: &Ship) -> bool {
        self.check_in_screen_v_ofs(p, ship, 0.03, 28.)
    }

    fn check_in_screen_v_ofs(&self, p: Vector, ship: &Ship, v: f32, ofs: f32) -> bool {
        let mut xr = f32::abs(p.x - ship.eye_pos().x);
        if xr > std::f32::consts::PI {
            xr = std::f32::consts::PI * 2. - xr;
        }
        xr *= self.get_radius(0.) / DEFAULT_RAD;
        xr <= v * (p.y + ofs)
    }

    pub fn get_torus_length(&self) -> usize {
        self.torus.slice_num
    }

    pub fn draw(&mut self, draw_state: &SliceDrawState, screen: &Screen) {
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
        }
        let mut line_bn = 0.4;
        let mut poly_bn = 0.;
        let mut light_bn = 0.5 - draw_state.dark_line_ratio * 0.2;
        if let Some(slice) = self.slices.slices.last_mut() {
            slice.set_point_pos();
        }
        let slices_len = self.slices.slices.len();
        for i in (1..slices_len).rev() {
            let rtd = match self.slices.ring_indexes[i] {
                Some(ring_index) => {
                    let ring = &self.torus.rings[ring_index];
                    let (p, d1, d2) = self.get_center_pos(ring.idx as f32);
                    Some(RingToDraw {
                        ring,
                        cnt: self.rings_state[ring_index],
                        p,
                        d1,
                        d2,
                    })
                }
                None => None,
            };
            let slices = &mut self.slices.slices;
            slices[i - 1].set_point_pos();
            slices[i].draw(
                &slices[i - 1],
                line_bn,
                poly_bn,
                light_bn,
                rtd,
                draw_state,
                screen,
            );
            line_bn = f32::min(line_bn * 1.02, 1.);
            light_bn = f32::min(light_bn * 1.02, 1.);
            if i < slices_len / 2 {
                if poly_bn <= 0. {
                    poly_bn = 0.2;
                }
                poly_bn = f32::min(poly_bn * 1.03, 1.);
            }
            if (i as f32) < slices_len as f32 * 0.75 {
                line_bn *= 1.0 - draw_state.dark_line_ratio * 0.05;
                light_bn *= 1.0 + draw_state.dark_line_ratio * 0.02;
            }
        }
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
        }
    }

    pub fn draw_backward(&mut self, draw_state: &SliceDrawState, screen: &Screen) {
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
        }
        let mut line_bn = 0.4;
        let mut poly_bn = 0.;
        let mut light_bn = 0.5 - draw_state.dark_line_ratio * 0.2;
        if let Some(slice) = self.slices_backward.slices.last_mut() {
            slice.set_point_pos();
        }
        let slices_backward_len = self.slices_backward.slices.len();
        for i in (1..slices_backward_len).rev() {
            let rtd = match self.slices_backward.ring_indexes[i] {
                Some(ring_index) => {
                    let ring = &self.torus.rings[ring_index];
                    let (p, d1, d2) = self.get_center_pos(ring.idx as f32);
                    Some(RingToDraw {
                        ring,
                        cnt: self.rings_state[ring_index],
                        p,
                        d1,
                        d2,
                    })
                }
                None => None,
            };
            let slices = &mut self.slices_backward.slices;
            slices[i - 1].set_point_pos();
            slices[i].draw(
                &slices[i - 1],
                line_bn,
                poly_bn,
                light_bn,
                rtd,
                draw_state,
                screen,
            );
            line_bn = f32::min(line_bn * 1.02, 1.);
            light_bn = f32::min(light_bn * 1.02, 1.);
            if i < slices_backward_len / 2 {
                if poly_bn <= 0. {
                    poly_bn = 0.2;
                }
                poly_bn = f32::min(poly_bn * 1.03, 1.);
            }
            if (i as f32) < slices_backward_len as f32 * 0.75 {
                line_bn *= 1.0 - draw_state.dark_line_ratio * 0.05;
                light_bn *= 1.0 + draw_state.dark_line_ratio * 0.02;
            }
        }
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
        }
    }
}

pub struct RingToDraw<'a> {
    ring: &'a Ring,
    cnt: u32,
    p: Vector3,
    d1: f32,
    d2: f32,
}

const DEPTH: f32 = 5.;

#[derive(Clone, Copy, Debug)]
pub struct SliceDrawState {
    pub dark_line_ratio: f32,
    pub line: Color,
    pub poly: Color,
}

#[derive(Clone)]
pub struct Slice {
    state: SliceState,
    d1: f32,
    d2: f32,
    point_from: f32,
    center_pos: Vector3,
    point_ratio: f32,
    point_pos: [Vector3; MAX_POINT_NUM],
    depth: f32,
}

impl fmt::Debug for Slice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "( state: {:?}, d1: {:?}, d2: {:?}, point_from: {:?}, center_pos: {:?}, point_ratio: {:?}, point_pos: [", self.state, self.d1, self.d2, self.point_from, self.center_pos, self.point_ratio)?;
        for (i, pos) in self.point_pos.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?})", pos)?;
        }
        write!(f, "], depth: {:?})", self.depth)?;
        Ok(())
    }
}

impl Slice {
    fn new() -> Self {
        Slice {
            state: SliceState::new(),
            d1: 0.,
            d2: 0.,
            point_from: 0.,
            center_pos: Vector3::default(),
            point_ratio: 0.,
            point_pos: [Vector3::default(); MAX_POINT_NUM],
            depth: 0.,
        }
    }

    fn set_first(&mut self, pf: f32, state: &SliceState, dpt: f32) {
        self.center_pos = Vector3::default();
        self.d1 = 0.;
        self.d2 = 0.;
        self.point_from = pf;
        self.state = *state;
        self.depth = dpt;
        self.point_ratio = 1.;
    }

    fn set_prev_to(&self, state: &SliceState, depth_ratio: f32, dpt: f32) -> Slice {
        let d1 = self.d1 + state.md1 * depth_ratio;
        let d2 = self.d2 + state.md2 * depth_ratio;
        let mut center_pos = Vector3::new_at(0., 0., DEPTH * depth_ratio);
        center_pos.roll_y(d1).roll_x(d2);
        center_pos += self.center_pos;
        let point_ratio = 1. + (f32::abs(depth_ratio) - 1.) * 0.02;
        let mut point_from = (self.point_from + state.mp * depth_ratio) % state.point_num as f32;
        if point_from < 0. {
            point_from += state.point_num as f32;
        }
        let state = *state;
        let depth = dpt;
        Slice {
            state,
            d1,
            d2,
            point_from,
            center_pos,
            point_ratio,
            point_pos: [Vector3::default(); MAX_POINT_NUM],
            depth,
        }
    }

    fn set(&mut self, from: &Slice) {
        self.state = from.state;
        self.d1 = from.d1;
        self.d2 = from.d2;
        self.point_from = from.point_from;
        self.center_pos = from.center_pos;
        self.point_ratio = from.point_ratio;
        self.depth = from.depth;
    }

    pub fn draw(
        &self,
        prev_slice: &Slice,
        line_bn: f32,
        poly_bn: f32,
        light_bn: f32,
        ring: Option<RingToDraw>,
        draw_state: &SliceDrawState,
        screen: &Screen,
    ) {
        let mut pi = self.point_from;
        let mut width = self.state.course_width;
        let mut prev_pi = 0.;
        let mut is_first = true;
        let mut poly_first = true;
        let round_slice = self.state.course_width >= self.state.point_num as f32;
        loop {
            if !is_first {
                let ps_pi =
                    (pi * prev_slice.state.point_num as f32 / self.state.point_num as f32) as usize;
                let ps_prev_pi = (prev_pi * prev_slice.state.point_num as f32
                    / self.state.point_num as f32) as usize;
                screen.set_color(draw_state.line * line_bn);
                unsafe {
                    gl::Begin(gl::GL_LINE_STRIP);
                }
                self.point_pos[pi as usize].gl_vertex();
                prev_slice.point_pos[ps_pi].gl_vertex();
                prev_slice.point_pos[ps_prev_pi].gl_vertex();
                unsafe {
                    gl::End();
                }
                if poly_bn > 0. {
                    if round_slice || (!poly_first && width > 0.) {
                        screen.set_alpha_color((draw_state.poly, poly_bn));
                        unsafe {
                            gl::Begin(gl::GL_TRIANGLE_FAN);
                        }
                        Vector3::blend(
                            self.point_pos[prev_pi as usize],
                            prev_slice.point_pos[ps_pi],
                            0.9,
                        )
                        .gl_vertex();
                        Vector3::blend(
                            self.point_pos[pi as usize],
                            prev_slice.point_pos[ps_prev_pi],
                            0.9,
                        )
                        .gl_vertex();
                        screen.set_alpha_color((draw_state.poly, poly_bn / 2.));
                        Vector3::blend(
                            self.point_pos[prev_pi as usize],
                            prev_slice.point_pos[ps_pi],
                            0.1,
                        )
                        .gl_vertex();
                        Vector3::blend(
                            self.point_pos[pi as usize],
                            prev_slice.point_pos[ps_prev_pi],
                            0.1,
                        )
                        .gl_vertex();
                        unsafe {
                            gl::End();
                        }
                    } else {
                        poly_first = false;
                    }
                }
            } else {
                is_first = false;
            }
            prev_pi = pi;
            pi += self.point_ratio;
            while pi >= self.state.point_num as f32 {
                pi -= self.state.point_num as f32;
            }
            if width <= 0. {
                break;
            }
            width -= self.point_ratio;
        }
        if self.state.course_width < self.state.point_num as f32 {
            pi = self.point_from;
            let ps_pi =
                (pi * prev_slice.state.point_num as f32 / self.state.point_num as f32) as usize;
            screen.set_color((line_bn / 3. * 2., line_bn / 3. * 2., line_bn));
            unsafe {
                gl::Begin(gl::GL_LINE_STRIP);
            }
            self.point_pos[pi as usize].gl_vertex();
            prev_slice.point_pos[ps_pi].gl_vertex();
            unsafe {
                gl::End();
            }
        }
        if !round_slice && light_bn > 0.2 {
            self.draw_side_light(self.get_left_edge_deg() - 0.07, light_bn, screen);
            self.draw_side_light(self.get_right_edge_deg() + 0.07, light_bn, screen);
        }
        if let Some(rtd) = ring {
            if light_bn > 0.2 {
                rtd.ring
                    .draw(rtd.cnt, rtd.p, rtd.d1, rtd.d2, light_bn * 0.7, screen);
            }
        }
    }

    fn set_point_pos(&mut self) {
        let mut d = 0.;
        let md = std::f32::consts::PI * 2. / (self.state.point_num as f32 - 1.);
        for pp in self.point_pos.iter_mut() {
            let mut rad_ofs = Vector3::new_at(0., self.state.rad * RAD_RATIO, 0.);
            rad_ofs.roll_z(d).roll_y(self.d1).roll_x(self.d2);
            *pp = rad_ofs + self.center_pos;
            d += md;
        }
    }

    fn draw_side_light(&self, deg: f32, light_bn: f32, screen: &Screen) {
        let mut rad_ofs = Vector3::new_at(0., self.state.rad, 0.);
        rad_ofs.roll_z(deg).roll_y(self.d1).roll_x(self.d2);
        rad_ofs += self.center_pos;
        screen.set_color(Color::from((1., 1., 0.6)) * light_bn);
        unsafe {
            gl::Begin(gl::GL_LINE_LOOP);
            gl::Vertex3f(rad_ofs.x - 0.5, rad_ofs.y - 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x + 0.5, rad_ofs.y - 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x + 0.5, rad_ofs.y + 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x - 0.5, rad_ofs.y + 0.5, rad_ofs.z);
            gl::End();
            gl::Begin(gl::GL_TRIANGLE_FAN);
            screen.set_color(Color::from((0.5, 0.5, 0.3)) * light_bn);
            gl::Vertex3f(rad_ofs.x, rad_ofs.y, rad_ofs.z);
            screen.set_color(Color::from((0.9, 0.9, 0.6)) * light_bn);
            gl::Vertex3f(rad_ofs.x - 0.5, rad_ofs.y - 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x - 0.5, rad_ofs.y + 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x + 0.5, rad_ofs.y + 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x + 0.5, rad_ofs.y - 0.5, rad_ofs.z);
            gl::Vertex3f(rad_ofs.x - 0.5, rad_ofs.y - 0.5, rad_ofs.z);
            gl::End();
        }
    }

    pub fn is_nearly_round(&self) -> bool {
        self.state.course_width >= self.state.point_num as f32 - 1.
    }

    pub fn get_left_edge_deg(&self) -> f32 {
        self.point_from * std::f32::consts::PI * 2. / self.state.point_num as f32
    }

    pub fn get_right_edge_deg(&self) -> f32 {
        let mut rd = (self.point_from + self.state.course_width) * std::f32::consts::PI * 2.
            / self.state.point_num as f32;
        if rd >= std::f32::consts::PI * 2. {
            rd -= std::f32::consts::PI * 2.;
        }
        rd
    }

    pub fn d1(&self) -> f32 {
        self.d1
    }

    pub fn d2(&self) -> f32 {
        self.d2
    }
}

const TORUS_LENGTH: isize = 5000;

pub struct Torus {
    slice_num: usize,
    torus_parts: Vec<TorusPart>,
    rings: Vec<Ring>,
}

impl Torus {
    pub fn new(seed: u64) -> Self {
        let mut rand = Rand::new(seed);
        let mut torus_parts = Vec::<TorusPart>::new();
        let mut rings = Vec::<Ring>::new();
        let mut slice_num = 0;
        let mut tl: isize = TORUS_LENGTH;
        let mut prev = SliceState::new();
        while tl > 0 {
            let lgt = 64 + rand.gen_usize(30);
            let tp = TorusPart::new(&prev, slice_num, lgt, &mut rand);
            prev = tp.slice_state;
            torus_parts.push(tp);
            tl -= lgt as isize;
            slice_num += lgt;
        }
        let mut ri = 5;
        while ri < slice_num - 100 {
            let ss = Torus::get_slice_state_internal(&torus_parts, ri);
            let ring_type = if ri == 5 {
                RingType::Final
            } else {
                RingType::Normal
            };
            rings.push(Ring::new(ri, ss.rad, ring_type));
            ri += 100 + rand.gen_usize(200);
        }
        Torus {
            slice_num,
            torus_parts,
            rings,
        }
    }

    fn get_slice_state(&self, idx: usize) -> SliceState {
        Torus::get_slice_state_internal(&self.torus_parts, idx)
    }

    fn get_slice_state_internal(torus_parts: &[TorusPart], idx: usize) -> SliceState {
        let tp_idx = torus_parts
            .binary_search_by(|tp| {
                use std::cmp::Ordering;
                if idx < tp.slice_idx_from {
                    Ordering::Greater
                } else if idx < tp.slice_idx_to {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            })
            .unwrap();
        let torus_parts_len = torus_parts.len();
        let prev_tp_idx = (tp_idx + torus_parts_len - 1) % torus_parts_len;
        torus_parts[tp_idx].create_blended_slice_state(&torus_parts[prev_tp_idx].slice_state, idx)
    }

    fn get_slice_state_and_ring(&self, idx: usize, pidx: usize) -> (SliceState, Option<usize>) {
        let ss = Torus::get_slice_state_internal(&self.torus_parts, idx);
        for (i, r) in self.rings.iter().enumerate() {
            if idx > pidx {
                if r.idx <= idx && r.idx > pidx {
                    return (ss, Some(i));
                }
            } else {
                if r.idx <= idx || r.idx > pidx {
                    return (ss, Some(i));
                }
            }
        }
        (ss, None)
    }
}

const BLEND_DISTANCE: f32 = 64.;

#[derive(Debug)]
struct TorusPart {
    slice_idx_from: usize,
    slice_idx_to: usize,
    slice_state: SliceState,
}

impl TorusPart {
    fn new(prev_state: &SliceState, slice_idx: usize, slice_num: usize, rand: &mut Rand) -> Self {
        let mut slice_state = *prev_state;
        slice_state.change_deg(rand);
        if f32::abs(prev_state.mp) >= 1. {
            if rand.gen_usize(2) == 0 {
                if prev_state.mp >= 1. {
                    slice_state.change_to_tight_curve_dir(rand, -1.);
                } else {
                    slice_state.change_to_tight_curve_dir(rand, 1.);
                }
            } else {
                slice_state.change_to_straight();
            }
        } else if prev_state.course_width >= prev_state.point_num as f32 || rand.gen_usize(2) == 0 {
            match rand.gen_usize(3) {
                0 => slice_state.change_rad(rand),
                1 => slice_state.change_width(rand),
                2 => slice_state.change_width_to_full(),
                val => panic!("{}", val),
            }
        } else {
            match rand.gen_usize(4) {
                0 => slice_state.change_to_tight_curve(rand),
                2 => slice_state.change_to_easy_curve(rand),
                1 | 3 => slice_state.change_to_straight(),
                val => panic!("{}", val),
            }
        }
        TorusPart {
            slice_idx_from: slice_idx,
            slice_idx_to: slice_idx + slice_num,
            slice_state,
        }
    }

    fn create_blended_slice_state(&self, blendee: &SliceState, idx: usize) -> SliceState {
        let dst = idx - self.slice_idx_from;
        let blend_ratio = dst as f32 / BLEND_DISTANCE;
        if blend_ratio >= 1.0 {
            return self.slice_state;
        }
        SliceState::new_blended(&self.slice_state, blendee, blend_ratio)
    }
}

const MAX_POINT_NUM: usize = 36;
const DEFAULT_POINT_NUM: usize = 24;
pub const DEFAULT_RAD: f32 = 21.;

#[derive(PartialEq, Debug, Clone, Copy)]
struct SliceState {
    md1: f32,
    md2: f32,
    rad: f32,
    point_num: usize,
    course_width: f32,
    mp: f32,
}

impl SliceState {
    fn new() -> Self {
        SliceState {
            md1: 0.,
            md2: 0.,
            rad: DEFAULT_RAD,
            point_num: DEFAULT_POINT_NUM,
            course_width: DEFAULT_POINT_NUM as f32,
            mp: 0.,
        }
    }

    fn new_blended(s1: &SliceState, s2: &SliceState, ratio: f32) -> Self {
        let point_num = (s1.point_num as f32 * ratio + s2.point_num as f32 * (1. - ratio)) as usize;
        let course_width =
            if s1.course_width >= s1.point_num as f32 && s2.course_width >= s2.point_num as f32 {
                point_num as f32
            } else {
                s1.course_width * ratio + s2.course_width * (1. - ratio)
            };
        SliceState {
            md1: s1.md1 * ratio + s2.md1 * (1. - ratio),
            md2: s1.md2 * ratio + s2.md2 * (1. - ratio),
            rad: s1.rad * ratio + s2.rad * (1. - ratio),
            point_num,
            course_width,
            mp: s1.mp,
        }
    }

    fn change_deg(&mut self, rand: &mut Rand) {
        self.md1 = rand.gen_signed_f32(0.005);
        self.md2 = rand.gen_signed_f32(0.005);
    }

    fn change_rad(&mut self, rand: &mut Rand) {
        self.rad = DEFAULT_RAD + rand.gen_signed_f32(DEFAULT_RAD * 0.3);
        let ppn = self.point_num;
        self.point_num = (self.rad * DEFAULT_POINT_NUM as f32 / DEFAULT_RAD) as usize;
        if ppn as f32 <= self.course_width {
            self.change_width_to_full();
        } else {
            self.course_width = self.course_width * self.point_num as f32 / ppn as f32;
        }
    }

    fn change_width(&mut self, rand: &mut Rand) {
        self.course_width =
            rand.gen_usize(self.point_num / 4) as f32 + self.point_num as f32 * 0.36;
    }

    fn change_width_to_full(&mut self) {
        self.course_width = self.point_num as f32;
    }

    fn change_to_straight(&mut self) {
        self.mp = 0.;
    }

    fn change_to_easy_curve(&mut self, rand: &mut Rand) {
        let mp = rand.gen_f32(0.05) + 0.04;
        self.mp = if rand.gen_usize(2) == 0 { -mp } else { mp };
    }

    fn change_to_tight_curve(&mut self, rand: &mut Rand) {
        let dir = rand.gen_usize(2) as f32 * 2. - 1.;
        self.change_to_tight_curve_dir(rand, dir as f32)
    }

    fn change_to_tight_curve_dir(&mut self, rand: &mut Rand, dir: f32) {
        self.mp = (rand.gen_f32(0.04) + 0.1) * dir;
    }
}

const NORMAL_COLOR: (f32, f32, f32) = (0.5, 1., 0.9);
const FINAL_COLOR: (f32, f32, f32) = (1., 0.9, 0.5);

#[derive(Debug)]
struct Ring {
    idx: usize,
    ring_type: RingType,
    display_list: DisplayList,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum RingType {
    Normal,
    Final,
}

impl Ring {
    fn new(idx: usize, slice_rad: f32, ring_type: RingType) -> Self {
        Ring {
            idx,
            ring_type,
            display_list: match ring_type {
                RingType::Normal => Ring::create_normal_ring(slice_rad),
                RingType::Final => Ring::create_final_ring(slice_rad),
            },
        }
    }

    fn create_normal_ring(r: f32) -> DisplayList {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        Ring::draw_ring(r, 1.2, 1.4, 16);
        display_list.end_list();
        display_list
    }

    fn create_final_ring(r: f32) -> DisplayList {
        let mut display_list = DisplayList::new(2);
        display_list.new_list();
        Ring::draw_ring(r, 1.2, 1.5, 14);
        display_list.end_list();
        display_list.new_list();
        Ring::draw_ring(r, 1.6, 1.9, 14);
        display_list.end_list();
        display_list
    }

    fn draw_ring(r: f32, rr1: f32, rr2: f32, num: u32) {
        let mut d = 0.;
        let md = 0.2;
        for _i in 0..num {
            unsafe {
                gl::Begin(gl::GL_LINE_LOOP);
            }
            let p1 = Vector3::new_at(f32::sin(d) * r * rr1, f32::cos(d) * r * rr1, 0.);
            let p2 = Vector3::new_at(f32::sin(d) * r * rr2, f32::cos(d) * r * rr2, 0.);
            let p3 = Vector3::new_at(f32::sin(d + md) * r * rr2, f32::cos(d + md) * r * rr2, 0.);
            let p4 = Vector3::new_at(f32::sin(d + md) * r * rr1, f32::cos(d + md) * r * rr1, 0.);
            let cp = (p1 + p2 + p3 + p4) / 4.;
            let np1 = Vector3::blend(p1, cp, 0.7);
            let np2 = Vector3::blend(p2, cp, 0.7);
            let np3 = Vector3::blend(p3, cp, 0.7);
            let np4 = Vector3::blend(p4, cp, 0.7);
            np1.gl_vertex();
            np2.gl_vertex();
            np3.gl_vertex();
            np4.gl_vertex();
            unsafe {
                gl::End();
            }
            d += md;
        }
    }

    fn draw(&self, cnt: u32, p: Vector3, d1: f32, d2: f32, a: f32, screen: &Screen) {
        unsafe {
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
        }
        let color = match self.ring_type {
            RingType::Normal => NORMAL_COLOR,
            RingType::Final => FINAL_COLOR,
        };
        screen.set_color(Color::from(color) * a);
        unsafe {
            gl::PushMatrix();
            gl::Translatef(p.x, p.y, p.z);
            gl::Rotatef(cnt as f32 * 1.0, 0., 0., 1.);
            gl::Rotatef(d1, 0., 1., 0.);
            gl::Rotatef(d2, 1., 0., 0.);
            self.display_list.call(0);
            gl::PopMatrix();
            if self.ring_type == RingType::Final {
                gl::PushMatrix();
                gl::Translatef(p.x, p.y, p.z);
                gl::Rotatef(cnt as f32 * -1.0, 0., 0., 1.);
                gl::Rotatef(d1, 0., 1., 0.);
                gl::Rotatef(d2, 1., 0., 0.);
                self.display_list.call(1);
                gl::PopMatrix();
            }
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
        }
    }
}
