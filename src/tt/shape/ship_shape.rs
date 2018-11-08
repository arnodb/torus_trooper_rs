use std::vec::Vec;

use crate::util::display_list::DisplayList;
use crate::util::rand::Rand;
use crate::util::vector::Vector;
// FIXME we are in util!
use crate::tt::screen::Screen;

use super::structure::{self, Structure};
use super::Drawable;

pub struct ShipShape {
    collision: Vector,
    display_list: DisplayList,
    rocket_x: Vec<f32>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ShipType {
    Small,
    Medium,
    Large,
}

impl ShipShape {
    pub fn new(ship_type: ShipType, damaged: bool, screen: &Screen, seed: u64) -> Self {
        match ship_type {
            ShipType::Small => ShipShape::create_small_type(Rand::new(seed), damaged, screen),
            ShipType::Medium => ShipShape::create_medium_type(Rand::new(seed), damaged, screen),
            ShipType::Large => ShipShape::create_large_type(Rand::new(seed), damaged, screen),
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

    fn create_small_type(mut rand: Rand, damaged: bool, screen: &Screen) -> ShipShape {
        let mut collision = Vector::default();
        let shaft_num = 1 + rand.gen_usize(2);
        println!("{}", shaft_num);
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
        }
    }

    fn create_medium_type(mut rand: Rand, damaged: bool, screen: &Screen) -> ShipShape {
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
        }
    }

    fn create_large_type(mut rand: Rand, damaged: bool, screen: &Screen) -> ShipShape {
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
        }
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
}

impl Drawable for ShipShape {
    fn draw(&self) {
        self.display_list.call(0);
    }
}
