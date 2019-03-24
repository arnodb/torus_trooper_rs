use crate::gl;

#[derive(PartialEq, Default, Clone, Copy, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn new_at(x: f32, y: f32) -> Self {
        Vector { x, y }
    }

    pub fn dist(self, v: Vector) -> f32 {
        let ax = f32::abs(self.x - v.x);
        let ay = f32::abs(self.y - v.y);
        if ax > ay {
            ax + ay / 2.
        } else {
            ay + ax / 2.
        }
    }
}

impl std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Vector) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, rhs: f32) -> Vector {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::MulAssign<f32> for Vector {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

#[derive(PartialEq, Default, Clone, Copy, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new_at(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub fn new_at_tuple(t: (f32, f32, f32)) -> Self {
        Vector3 {
            x: t.0,
            y: t.1,
            z: t.2,
        }
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

    pub fn gl_translate(&self) {
        unsafe {
            gl::Translatef(self.x, self.y, self.z);
        }
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

impl std::ops::Mul<Vector3> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
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

impl std::ops::DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}
