use crate::tt::ship::Ship;
use crate::util::rand::Rand;
use crate::util::vector::Vector3;

const ZOOM_CNT: i32 = 24;

pub struct Camera {
    rand: Rand,
    camera_pos: Vector3,
    camera_trg: Vector3,
    camera_vel: Vector3,
    look_at_pos: Vector3,
    look_at_ofs: Vector3,
    look_at_cnt: i32,
    change_cnt: i32,
    move_cnt: i32,
    deg: f32,
    zoom: f32,
    zoom_trg: f32,
    zoom_min: f32,
    move_type: MoveType,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum MoveType {
    Float,
    Fix,
}

const MOVE_TYPES: [MoveType; 2] = [MoveType::Float, MoveType::Fix];

impl Camera {
    pub fn new() -> Self {
        Camera {
            rand: Rand::new_not_recorded(Rand::rand_seed()),
            camera_pos: Vector3::default(),
            camera_trg: Vector3::default(),
            camera_vel: Vector3::default(),
            look_at_pos: Vector3::default(),
            look_at_ofs: Vector3::default(),
            look_at_cnt: 0,
            change_cnt: 0,
            move_cnt: 0,
            deg: 0.,
            zoom: 0.,
            zoom_trg: 0.,
            zoom_min: 0.,
            move_type: MoveType::Float,
        }
    }

    pub fn start(&mut self) {
        self.change_cnt = 0;
        self.move_cnt = 0;
    }

    pub fn mov(&mut self, ship: &Ship) {
        let ship_rel_pos = ship.rel_pos();
        self.change_cnt -= 1;
        if self.change_cnt < 0 {
            self.move_type = MOVE_TYPES[self.rand.gen_usize(MOVE_TYPES.len())];
            match self.move_type {
                MoveType::Float => {
                    self.change_cnt = 256 + self.rand.gen_usize(150) as i32;
                    self.camera_trg.x = ship_rel_pos.x + self.rand.gen_signed_f32(1.);
                    self.camera_trg.y = ship_rel_pos.y - 12. + self.rand.gen_signed_f32(48.);
                    self.camera_trg.z = self.rand.gen_usize(32) as f32;
                    self.camera_vel.x = (ship_rel_pos.x - self.camera_trg.x)
                        / self.change_cnt as f32
                        * (1. + self.rand.gen_f32(1.));
                    self.camera_vel.y = (ship_rel_pos.y - 12. - self.camera_trg.y)
                        / self.change_cnt as f32
                        * (1.5 + self.rand.gen_f32(0.8));
                    self.camera_vel.z =
                        (16. - self.camera_trg.z) / self.change_cnt as f32 * self.rand.gen_f32(1.);
                    self.zoom_trg = 1.2 + self.rand.gen_f32(0.8);
                    self.zoom = self.zoom_trg;
                }
                MoveType::Fix => {
                    self.change_cnt = 200 + self.rand.gen_usize(100) as i32;
                    self.camera_trg.x = self.rand.gen_signed_f32(0.3);
                    self.camera_trg.y = -8. - self.rand.gen_f32(12.);
                    self.camera_trg.z = 8. + self.rand.gen_usize(16) as f32;
                    self.camera_vel.x = (ship_rel_pos.x - self.camera_trg.x)
                        / self.change_cnt as f32
                        * (1. + self.rand.gen_f32(1.));
                    self.camera_vel.y = self.rand.gen_signed_f32(0.05);
                    self.camera_vel.z =
                        (10. - self.camera_trg.z) / self.change_cnt as f32 * self.rand.gen_f32(0.5);
                    self.zoom_trg = 1.0 + self.rand.gen_signed_f32(0.25);
                    self.zoom = 0.2 + self.rand.gen_f32(0.8);
                }
            }
            self.camera_pos = self.camera_trg;
            self.deg = self.camera_trg.x;
            self.look_at_ofs = Vector3::default();
            self.look_at_cnt = 0;
            self.zoom_min = 1.0 - self.rand.gen_f32(0.9);
        }
        self.look_at_cnt -= 1;
        if self.look_at_cnt == ZOOM_CNT {
            self.look_at_ofs.x = self.rand.gen_signed_f32(0.4);
            self.look_at_ofs.y = self.rand.gen_signed_f32(3.);
            self.look_at_ofs.z = self.rand.gen_signed_f32(10.);
        } else if self.look_at_cnt < 0 {
            self.look_at_cnt = 32 + self.rand.gen_usize(48) as i32;
        }
        self.camera_trg += self.camera_vel;
        let mut co = match self.move_type {
            MoveType::Float => self.camera_trg,
            MoveType::Fix => {
                let mut od = ship_rel_pos.x - self.deg;
                while od >= std::f32::consts::PI {
                    od -= std::f32::consts::PI * 2.;
                }
                while od < -std::f32::consts::PI {
                    od += std::f32::consts::PI * 2.;
                }
                self.deg += od * 0.2;
                Vector3::new_at(
                    self.camera_trg.x + ship_rel_pos.x,
                    self.camera_trg.y + ship_rel_pos.y,
                    self.camera_trg.z,
                )
            }
        };
        co.x -= self.camera_pos.x;
        while co.x >= std::f32::consts::PI {
            co.x -= std::f32::consts::PI * 2.;
        }
        while co.x < -std::f32::consts::PI {
            co.x += std::f32::consts::PI * 2.;
        }
        co.y -= self.camera_pos.y;
        co.z -= self.camera_pos.z;
        self.camera_pos.x += co.x * 0.12;
        self.camera_pos.y += co.y * 0.12;
        self.camera_pos.z += co.z * 0.12;
        let ofs_ratio = if self.look_at_cnt <= ZOOM_CNT {
            1.0 + f32::abs(self.zoom_trg - self.zoom) * 2.5
        } else {
            1.0
        };
        let mut lox = ship_rel_pos.x + self.look_at_ofs.x * ofs_ratio - self.look_at_pos.x;
        while lox >= std::f32::consts::PI {
            lox -= std::f32::consts::PI * 2.;
        }
        while lox < -std::f32::consts::PI {
            lox += std::f32::consts::PI * 2.;
        }
        let loy = ship_rel_pos.y + self.look_at_ofs.y * ofs_ratio - self.look_at_pos.y;
        let loz = self.look_at_ofs.z * ofs_ratio - self.look_at_pos.z;
        if self.look_at_cnt <= ZOOM_CNT {
            self.zoom += (self.zoom_trg - self.zoom) * 0.16;
            self.look_at_pos.x += lox * 0.2;
            self.look_at_pos.y += loy * 0.2;
            self.look_at_pos.z += loz * 0.2;
        } else {
            self.look_at_pos.x += lox * 0.1;
            self.look_at_pos.y += lox * 0.1;
            self.look_at_pos.z += loz * 0.1;
        }
        self.look_at_ofs *= 0.985;
        if f32::abs(self.look_at_ofs.x) < 0.04 {
            self.look_at_ofs.x = 0.;
        }
        if f32::abs(self.look_at_ofs.y) < 0.3 {
            self.look_at_ofs.y = 0.;
        }
        if f32::abs(self.look_at_ofs.z) < 1. {
            self.look_at_ofs.z = 0.;
        }
        self.move_cnt -= 1;
        if self.move_cnt < 0 {
            self.move_cnt = 15 + self.rand.gen_usize(15) as i32;
            let mut lox = f32::abs(self.look_at_pos.x - self.camera_pos.x);
            if lox > std::f32::consts::PI {
                lox = std::f32::consts::PI * 2. - lox;
            }
            let ofs = lox * 3. + f32::abs(self.look_at_pos.y - self.camera_pos.y);
            self.zoom_trg = 3.0 / ofs;
            if self.zoom_trg < self.zoom_min {
                self.zoom_trg = self.zoom_min;
            } else if self.zoom_trg > 2. {
                self.zoom_trg = 2.;
            }
        }
        if self.look_at_pos.x < 0. {
            self.look_at_pos.x += std::f32::consts::PI * 2.;
        } else if self.look_at_pos.x >= std::f32::consts::PI * 2. {
            self.look_at_pos.x -= std::f32::consts::PI * 2.;
        }
    }

    pub fn camera_pos(&self) -> Vector3 {
        self.camera_pos
    }

    pub fn look_at_pos(&self) -> Vector3 {
        self.look_at_pos
    }

    pub fn deg(&self) -> f32 {
        self.deg
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }
}
