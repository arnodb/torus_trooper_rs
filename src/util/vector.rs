use crate::gl;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn new() -> Self {
        Vector { x: 0., y: 0. }
    }

    /*TODO
    fn get_element(self, v: Vector) -> Vector {
        let ll = v * v;
        if ll != 0. {
            let mag = self * v;
            Vector {
                x: mag * v.x / ll,
                y: mag * v.y / ll,
            }
        } else {
            Vector::new()
        }
    }
    
    fn check_side(self, pos1: Vector, pos2: Vector) -> f32 {
        self.check_side3(pos1, pos2, Vector::new())
    }
    
    fn check_side3(self, pos1: Vector, pos2: Vector, ofs: Vector) -> f32 {
        let xo = pos2.x - pos1.x;
        let yo = pos2.y - pos1.y;
        let mx = self.x + ofs.x;
        let my = self.y + ofs.y;
        if xo == 0. {
            if yo == 0. {
                return 0.
            }
            let val = mx - pos1.x;
            if yo > 0. {
                return val;
            } else {
                return -val;
            }
        } else if yo == 0. {
            let val = pos1.y - my;
            if xo > 0. {
                return val;
            } else {
                return -val;
            }
        } else {
            let val = (mx - pos1.x) / xo - (my - pos1.y) / yo;
            if xo * yo > 0. {
                return val;
            } else {
                return -val;
            }
        }
    }
    
    fn check_cross(self, p: Vector, p1: Vector, p2: Vector, width: f32) -> bool {
        let ax = if self.x < p.x {
            (self.x - width, p.x + width)
        } else {
            (p.x - width, self.x + width)
        };
        let ay = if self.y < p.y {
            (self.y - width, p.y + width)
        } else {
            (p.y - width, self.y + width)
        };
        let by = if p2.y < p1.y {
            (p2.y - width, p1.y + width)
        } else {
            (p1.y - width, p2.y + width)
        };
        if ay.1 >= by.0 && by.1 >= ay.0 {
            let bx = if p2.x < p1.x {
                (p2.x - width, p1.x + width)
            } else {
                (p1.x - width, p2.x + width)
            };
            if ax.1 >= bx.0 && bx.1 >= ax.0 {
                let a = self.y - p.y;
                let b = p.x - self.x;
                let c = p.x * self.y - p.y * self.x;
                let d = p2.y - p1.y;
                let e = p1.x - p2.x;
                let f = p1.x * p2.y - p1.y * p2.x;
                let dnm = b * d - a * e;
                if dnm != 0. {
                    let x = (b * f - c * e) / dnm;
                    let y = (c * d - a * f) / dnm;
                    if ax.0 <= x && x <= ax.1 && ay.0 <= y && y <= ay.1 &&
                        bx.0 <= x && x <= bx.1 && by.0 <= y && y <= by.1 {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    
    fn check_hit_dist(self, p: Vector, pp: Vector, dist: f32) -> bool {
        let bmvx = pp.x - p.x;
        let bmvy = pp.y - p.y;
        let inaa = bmvx * bmvx + bmvy * bmvy;
        if inaa > 0.00001 {
            let sofsx = self.x - p.x;
            let sofsy = self.y - p.y;
            let inab = bmvx * sofsx + bmvy * sofsy;
            if inab >= 0. && inab <= inaa {
                let hd = sofsx * sofsx + sofsy * sofsy - inab * inab / inaa;
                if hd >= 0. && hd <= dist {
                    return true;
                }
            }
        }
        return false;
    }
    
    fn size(self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y)
    }
    
    fn dist(self, v: Vector) -> f32 {
        let ax = f32::abs(self.x - v.x);
        let ay = f32::abs(self.y - v.y);
        if ax > ay {
            ax + ay / 2.
        } else {
            ay + ax / 2.
        }
    }
    */
}

/* TODO
impl std::ops::Mul for Vector {
    type Output = f32;
    fn mul(self, rhs: Vector) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl std::ops::AddAssign for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Vector) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::MulAssign for Vector {
    fn mul_assign(&mut self, rhs: Vector) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl std::ops::DivAssign for Vector {
    fn div_assign(&mut self, rhs: Vector) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}
*/

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new() -> Self {
        Vector3 {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn new_at(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub fn roll_x(&mut self, d: f32) -> &mut Self {
        let ty = self.y * f32::cos(d) - self.z * f32::sin(d);
        self.z = self.y * f32::sin(d) + self.z * f32::cos(d);
        self.y = ty;
        self
    }

    pub fn roll_y(&mut self, d: f32) -> &mut Self {
        let tx = self.x * f32::cos(d) - self.z * f32::sin(d);
        self.z = self.x * f32::sin(d) + self.z * f32::cos(d);
        self.x = tx;
        self
    }

    pub fn roll_z(&mut self, d: f32) -> &mut Self {
        let tx = self.x * f32::cos(d) - self.y * f32::sin(d);
        self.y = self.x * f32::sin(d) + self.y * f32::cos(d);
        self.x = tx;
        self
    }

    pub fn gl_vertex(&self) {
        unsafe {
            gl::Vertex3f(self.x, self.y, self.z);
        }
    }

    pub fn blend(v1: Vector3, v2: Vector3, ratio: f32) -> Self {
        Vector3 {
            x: v1.x * ratio + v2.x * (1. - ratio),
            y: v1.y * ratio + v2.y * (1. - ratio),
            z: v1.z * ratio + v2.z * (1. - ratio),
        }
    }
}

impl std::ops::Add for Vector3 {
    type Output = Vector3;
    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl std::ops::Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
