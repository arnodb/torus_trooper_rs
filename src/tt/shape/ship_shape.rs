use std::vec::Vec;

use crate::tt::actor::particle::{ParticlePool, ParticleSpec};
use crate::tt::screen::Screen;
use crate::tt::tunnel::Tunnel;
use crate::util::display_list::DisplayList;
use crate::util::rand::Rand;
use crate::util::vector::Vector;

use super::structure::{self, Structure};
use super::{Collidable, Drawable};

pub struct ShipShape {
    collision: Vector,
    display_list: DisplayList,
    rocket_x: Vec<f32>,
    color: usize,
}

impl ShipShape {
    pub fn new_small(damaged: bool, screen: &Screen, seed: u64) -> Self {
        let mut rand = Rand::new(seed);
        let mut collision = Vector::default();
        let shaft_num = 1 + rand.gen_usize(2);
        let sx = (0.25 + rand.gen_f32(0.1)) * 1.5;
        let so = (0.5 + rand.gen_f32(0.3)) * 1.5;
        let sl = (0.7 + rand.gen_f32(0.9)) * 1.5;
        let sw = (1.5 + rand.gen_f32(0.7)) * 1.5;
        let sd1 = rand.gen_f32(1.) * std::f32::consts::PI / 3. + std::f32::consts::PI / 4.;
        let sd2 = rand.gen_f32(1.) * std::f32::consts::PI / 10.;
        let cl = rand.gen_usize(structure::COLOR_RGB.len() - 2) + 2;
        let shp = structure::SHIP_SHAPES[rand.gen_usize(structure::SHIP_SHAPES.len())];
        let mut rocket_x = vec![];
        let mut structure = vec![];
        match shaft_num {
            1 => {
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    0.,
                    0.,
                    so,
                    sd1,
                    sl,
                    2,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                collision.x = so / 2. + sw;
                collision.y = sl / 2.;
                rocket_x.push(0.);
            }
            2 => {
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    true,
                    damaged,
                ));
                collision.x = sx + so / 2. + sw;
                collision.y = sl / 2.;
                rocket_x.push(sx * 0.05);
                rocket_x.push(-sx * 0.05);
            }
            val => panic!("{} is out of range", val),
        }
        collision.x *= 0.1;
        collision.y *= 1.2;
        let display_list = ShipShape::create_display_list(&structure, screen);
        ShipShape {
            collision,
            display_list,
            rocket_x,
            color: cl,
        }
    }

    pub fn new_medium(damaged: bool, screen: &Screen, seed: u64) -> Self {
        let mut rand = Rand::new(seed);
        let mut collision = Vector::default();
        let shaft_num = 3 + rand.gen_usize(2);
        let sx = (1.0 + rand.gen_f32(0.7)) * 1.6;
        let so = (0.9 + rand.gen_f32(0.6)) * 1.6;
        let sl = (1.5 + rand.gen_f32(2.0)) * 1.6;
        let sw = (2.5 + rand.gen_f32(1.4)) * 1.6;
        let sd1 = rand.gen_f32(1.) * std::f32::consts::PI / 3. + std::f32::consts::PI / 4.;
        let sd2 = rand.gen_f32(1.) * std::f32::consts::PI / 10.;
        let cl = rand.gen_usize(structure::COLOR_RGB.len() - 2) + 2;
        let shp = structure::SHIP_SHAPES[rand.gen_usize(structure::SHIP_SHAPES.len())];
        let mut rocket_x = vec![];
        let mut structure = vec![];
        match shaft_num {
            3 => {
                let cshp = structure::SHIP_SHAPES[rand.gen_usize(structure::SHIP_SHAPES.len())];
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    0.,
                    0.,
                    so * 0.5,
                    sd1,
                    sl,
                    2,
                    sw,
                    sd1,
                    sd2,
                    cl,
                    cshp,
                    8,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl * 0.8,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl * 0.8,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    true,
                    damaged,
                ));
                collision.x = sx + so / 2. + sw;
                collision.y = sl / 2.;
                rocket_x.push(0.);
                rocket_x.push(sx * 0.05);
                rocket_x.push(-sx * 0.05);
            }
            4 => {
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx / 3.,
                    -sx / 2.,
                    so,
                    sd1,
                    sl * 0.7,
                    1,
                    sw * 0.6,
                    sd1 / 3.,
                    sd2 / 2.,
                    cl,
                    shp,
                    5,
                    false,
                    false,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx / 3.,
                    -sx / 2.,
                    so,
                    sd1,
                    sl * 0.7,
                    1,
                    sw * 0.6,
                    sd1 / 3.,
                    sd2 / 2.,
                    cl,
                    shp,
                    5,
                    true,
                    false,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    true,
                    damaged,
                ));
                collision.x = sx + so / 2. + sw;
                collision.y = sl / 2.;
                rocket_x.push(sx * 0.025);
                rocket_x.push(-sx * 0.025);
                rocket_x.push(sx * 0.05);
                rocket_x.push(-sx * 0.05);
            }
            val => panic!("{} is out of range", val),
        }
        collision.x *= 0.1;
        collision.y *= 1.2;
        let display_list = ShipShape::create_display_list(&structure, screen);
        ShipShape {
            collision,
            display_list,
            rocket_x,
            color: cl,
        }
    }

    pub fn new_large(damaged: bool, screen: &Screen, seed: u64) -> Self {
        let mut rand = Rand::new(seed);
        let mut collision = Vector::default();
        let shaft_num = 5 + rand.gen_usize(2);
        let sx = (3.0 + rand.gen_f32(2.2)) * 1.6;
        let so = (1.5 + rand.gen_f32(1.0)) * 1.6;
        let sl = (3.0 + rand.gen_f32(4.0)) * 1.6;
        let sw = (5.0 + rand.gen_f32(2.5)) * 1.6;
        let sd1 = rand.gen_f32(1.) * std::f32::consts::PI / 3. + std::f32::consts::PI / 4.;
        let sd2 = rand.gen_f32(1.) * std::f32::consts::PI / 10.;
        let cl = rand.gen_usize(structure::COLOR_RGB.len() - 2) + 2;
        let shp = structure::SHIP_SHAPES[rand.gen_usize(structure::SHIP_SHAPES.len())];
        let mut rocket_x = vec![];
        let mut structure = vec![];
        match shaft_num {
            5 => {
                let cshp = structure::SHIP_SHAPES[rand.gen_usize(structure::SHIP_SHAPES.len())];
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    0.,
                    0.,
                    so * 0.5,
                    sd1,
                    sl,
                    2,
                    sw,
                    sd1,
                    sd2,
                    cl,
                    cshp,
                    8,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx * 0.6,
                    0.,
                    so,
                    sd1,
                    sl * 0.6,
                    1,
                    sw,
                    sd1 / 3.,
                    sd2 / 2.,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx * 0.6,
                    0.,
                    so,
                    sd1,
                    sl * 0.6,
                    1,
                    sw,
                    sd1 / 3.,
                    sd2 / 2.,
                    cl,
                    shp,
                    5,
                    true,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl * 0.9,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl * 0.9,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    true,
                    damaged,
                ));
                collision.x = sx + so / 2. + sw;
                collision.y = sl / 2.;
                rocket_x.push(0.);
                rocket_x.push(sx * 0.03);
                rocket_x.push(-sx * 0.03);
                rocket_x.push(sx * 0.05);
                rocket_x.push(-sx * 0.05);
            }
            6 => {
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx / 4.,
                    -sx / 2.,
                    so,
                    sd1,
                    sl * 0.6,
                    1,
                    sw * 0.6,
                    sd1 / 3.,
                    sd2 / 2.,
                    cl,
                    shp,
                    5,
                    false,
                    false,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx / 4.,
                    -sx / 2.,
                    so,
                    sd1,
                    sl * 0.6,
                    1,
                    sw * 0.6,
                    sd1 / 3.,
                    sd2 / 2.,
                    cl,
                    shp,
                    5,
                    true,
                    false,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx / 2.,
                    -sx / 3. * 2.,
                    so,
                    sd1,
                    sl * 0.8,
                    1,
                    sw * 0.8,
                    sd1 / 3.,
                    sd2 / 3. * 2.,
                    cl,
                    shp,
                    5,
                    false,
                    false,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx / 2.,
                    -sx / 3. * 2.,
                    so,
                    sd1,
                    sl * 0.8,
                    1,
                    sw * 0.8,
                    sd1 / 3.,
                    sd2 / 3. * 2.,
                    cl,
                    shp,
                    5,
                    true,
                    false,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    false,
                    damaged,
                ));
                structure.extend(ShipShape::create_shaft(
                    &mut rand,
                    sx,
                    0.,
                    so,
                    sd1,
                    sl,
                    1,
                    sw,
                    sd1 / 2.,
                    sd2,
                    cl,
                    shp,
                    5,
                    true,
                    damaged,
                ));
                collision.x = sx + so / 2. + sw;
                collision.y = sl / 2.;
                rocket_x.push(sx * 0.0125);
                rocket_x.push(-sx * 0.0125);
                rocket_x.push(sx * 0.025);
                rocket_x.push(-sx * 0.025);
                rocket_x.push(sx * 0.05);
                rocket_x.push(-sx * 0.05);
            }
            val => panic!("{} is out of range", val),
        }
        collision.x *= 0.1;
        collision.y *= 1.2;
        let display_list = ShipShape::create_display_list(&structure, screen);
        ShipShape {
            collision,
            display_list,
            rocket_x,
            color: cl,
        }
    }

    fn create_display_list(structure: &Vec<Structure>, screen: &Screen) -> DisplayList {
        let mut display_list = DisplayList::new(1);
        display_list.new_list();
        for st in structure {
            st.create_display_list(screen);
        }
        display_list.end_list();
        display_list
    }

    fn create_shaft(
        rand: &mut Rand,
        ox: f32,
        oy: f32,
        offset: f32,
        od1: f32,
        rocket_length: f32,
        wing_num: i32,
        wing_width: f32,
        wing_d1: f32,
        wing_d2: f32,
        color: usize,
        shp: structure::Shape,
        div_num: usize,
        rev: bool,
        damaged: bool,
    ) -> Vec<Structure> {
        let mut sts = vec![];
        let mut pos_x = ox;
        let pos_y = oy;
        if rev {
            pos_x = -pos_x;
        }
        let st = Structure::new(
            Vector { x: pos_x, y: pos_y },
            0.,
            0.,
            rocket_length * 0.15,
            rocket_length,
            structure::Shape::Rocket,
            1.,
            match damaged {
                false => 1,
                true => 0,
            },
            0,
        );
        sts.push(st);
        let wofs = offset;
        let whgt = rocket_length * (rand.gen_f32(0.5) + 1.5);
        for i in 0..wing_num {
            let inverse = (((i % 2) * 2) - 1)
                * match rev {
                    false => 1,
                    true => -1,
                }
                == 1;
            let mut pos_x = ox + f32::sin(od1) * wofs;
            let pos_y = oy + f32::cos(od1) * wofs;
            let mut d1 = wing_d1 * 180. / std::f32::consts::PI;
            let d2 = wing_d2 * 180. / std::f32::consts::PI;
            let mut shape_x_reverse = 1.;
            if inverse {
                pos_x = -pos_x;
                d1 = -d1;
                shape_x_reverse = -shape_x_reverse;
            }
            let st = Structure::new(
                Vector { x: pos_x, y: pos_y },
                d1,
                d2,
                wing_width,
                whgt,
                shp,
                shape_x_reverse,
                match damaged {
                    false => color,
                    true => 0,
                },
                div_num,
            );
            sts.push(st);
        }
        sts
    }

    pub fn add_particles(&self, pos: Vector, tunnel: &Tunnel, particles: &mut ParticlePool) {
        for rx in &self.rocket_x {
            let got_pt = particles.get_instance_and(|pt, particles_rand| {
                let rocket_pos = Vector::new_at(pos.x + rx, pos.y - 0.15);
                pt.set(
                    &ParticleSpec::Jet,
                    rocket_pos,
                    1.,
                    std::f32::consts::PI,
                    0.,
                    0.2,
                    (0.3, 0.4, 1.0).into(),
                    16,
                    tunnel,
                    particles_rand,
                );
            });
            if !got_pt {
                break;
            }
        }
    }

    // TODO this is not the correct Rand instance
    pub fn add_fragments(
        &self,
        pos: Vector,
        tunnel: &Tunnel,
        particles: &mut ParticlePool,
        rand: &mut Rand,
    ) {
        if self.collision.x < 0.5 {
            return;
        }
        for _ in 0..(self.collision.x * 40.) as usize {
            let wb = self.collision.x;
            let hb = self.collision.y;
            let got_pt = particles.get_instance_and(|pt, particles_rand| {
                pt.set(
                    &ParticleSpec::Fragment {
                        d1: 0.,
                        d2: 0.,
                        md1: 0.,
                        md2: 0.,
                        width: wb + rand.gen_f32(wb),
                        height: hb + rand.gen_f32(hb),
                    },
                    pos,
                    1.,
                    rand.gen_signed_f32(0.1),
                    1. + rand.gen_signed_f32(1.),
                    0.2 + rand.gen_f32(0.2),
                    structure::COLOR_RGB[self.color].into(),
                    (32 + rand.gen_usize(16)) as i32,
                    tunnel,
                    particles_rand,
                );
            });
            if !got_pt {
                break;
            }
        }
    }
}

impl Drawable for ShipShape {
    fn draw(&self) {
        self.display_list.call(0);
    }
}

impl Collidable for ShipShape {
    fn collision(&self) -> Vector {
        self.collision
    }
}
