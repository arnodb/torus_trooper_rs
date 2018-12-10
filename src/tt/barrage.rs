use bulletml::parse::BulletMLParser;
use bulletml::{self, BulletML};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use crate::util::rand::Rand;
use crate::util::vector::Vector;

use crate::tt::actor::bullet::{BMLParam, BulletPool};
use crate::tt::actor::PoolActorRef;
use crate::tt::screen::Screen;
use crate::tt::shape::bullet_shape::BulletShape;
use crate::tt::shape::Drawable;

pub struct Barrage {
    rand: Rand,
    bml_params: Rc<Vec<BMLParam>>,
    shape: Rc<Drawable>,
    disap_shape: Rc<Drawable>,
    long_range: bool,
    prev_wait: u32,
    post_wait: u32,
    no_x_reverse: bool,
}

impl Barrage {
    pub fn new(shape: &Rc<Drawable>, disap_shape: &Rc<Drawable>) -> Self {
        Barrage {
            rand: Rand::new(Rand::rand_seed()),
            bml_params: Rc::new(Vec::new()),
            shape: shape.clone(),
            disap_shape: disap_shape.clone(),
            long_range: false,
            prev_wait: 0,
            post_wait: 0,
            no_x_reverse: false,
        }
    }

    pub fn set_wait(&mut self, prev_wait: u32, post_wait: u32) {
        self.prev_wait = prev_wait;
        self.post_wait = post_wait;
    }

    pub fn set_long_range(&mut self, long_range: bool) {
        self.long_range = long_range;
    }

    pub fn set_no_x_reverse(&mut self) {
        self.no_x_reverse = true;
    }

    pub fn add_bml(&mut self, bml: &Rc<BulletML>, r: f32, re: bool, s: f32) {
        Rc::get_mut(&mut self.bml_params).unwrap().push(BMLParam {
            bml: bml.clone(),
            rank: r,
            root_rank_effect: if re { 1. } else { 0. },
            speed: s,
        });
    }

    pub fn add_top_bullet(&mut self, bullets: &mut BulletPool) -> Option<PoolActorRef> {
        let x_reverse = if self.no_x_reverse {
            1.
        } else {
            (self.rand.gen_usize(2) as isize * 2 - 1) as f32
        };
        bullets.add_top_bullet(
            /*TODO*/ &self.bml_params,
            Vector::default(),
            std::f32::consts::PI,
            0.,
            &self.shape,
            &self.disap_shape,
            x_reverse,
            1.,
            self.long_range,
            self.prev_wait,
            self.post_wait,
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BulletShapeType {
    Triangle,
    Square,
    Bar,
}

pub struct BarrageManager {
    bmls: BTreeMap<OsString, BTreeMap<OsString, Rc<BulletML>>>,
    square_bullet_shapes: (Rc<BulletShape>, Rc<BulletShape>),
    triangle_bullet_shapes: (Rc<BulletShape>, Rc<BulletShape>),
    bar_bullet_shapes: (Rc<BulletShape>, Rc<BulletShape>),
}

const BARRAGE_DIR_NAME: &str = "barrage";

impl BarrageManager {
    pub fn load(screen: &Screen) -> Result<Self, bulletml::parse::Error> {
        let mut bmls = BTreeMap::new();
        let dirs = fs::read_dir(BARRAGE_DIR_NAME)?;
        for dir_name in dirs {
            let dir_name = dir_name?;
            if dir_name.file_type()?.is_dir() {
                let files = fs::read_dir(dir_name.path())?;
                for file_name in files {
                    let file_name = file_name?;
                    if file_name.file_type()?.is_file() {
                        if let Some("xml") = file_name.path().extension().and_then(OsStr::to_str) {
                            let entry = bmls
                                .entry(dir_name.file_name().to_os_string())
                                .or_insert(BTreeMap::new());
                            entry.insert(
                                file_name.file_name().to_os_string(),
                                Rc::new(BarrageManager::load_instance(&file_name.path())?),
                            );
                        }
                    }
                }
            }
        }
        Ok(BarrageManager {
            bmls,
            square_bullet_shapes: (
                Rc::new(BulletShape::new_square(false, screen)),
                Rc::new(BulletShape::new_square(true, screen)),
            ),
            triangle_bullet_shapes: (
                Rc::new(BulletShape::new_triangle(false, screen)),
                Rc::new(BulletShape::new_triangle(true, screen)),
            ),
            bar_bullet_shapes: (
                Rc::new(BulletShape::new_bar(false, screen)),
                Rc::new(BulletShape::new_bar(true, screen)),
            ),
        })
    }

    pub fn load_instance(path: &PathBuf) -> Result<BulletML, bulletml::parse::Error> {
        BulletMLParser::parse_file(path.as_path())
    }

    pub fn get_instance(&self, dir_name: &OsStr, file_name: &OsStr) -> &Rc<BulletML> {
        &self.bmls[&dir_name.to_os_string()][&file_name.to_os_string()]
    }

    pub fn get_instance_list(&self, dir_name: &OsStr) -> Vec<(&OsString, &Rc<BulletML>)> {
        let dir_entry = &self.bmls[&dir_name.to_os_string()];
        dir_entry.iter().collect()
    }

    pub fn get_shape(&self, shape_type: BulletShapeType) -> (&Rc<BulletShape>, &Rc<BulletShape>) {
        match shape_type {
            BulletShapeType::Square => (&self.square_bullet_shapes.0, &self.square_bullet_shapes.1),
            BulletShapeType::Triangle => (
                &self.triangle_bullet_shapes.0,
                &self.triangle_bullet_shapes.1,
            ),
            BulletShapeType::Bar => (&self.bar_bullet_shapes.0, &self.bar_bullet_shapes.1),
        }
    }
}

// TODO Drop
