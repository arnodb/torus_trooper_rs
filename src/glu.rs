use crate::gl;

fn make_identityf(m: &mut [f32; 16]) {
    *m = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];
}

fn normalize(v: &mut [f32; 3]) {
    let r = f32::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    if r == 0.0 {
        return;
    }
    v[0] /= r;
    v[1] /= r;
    v[2] /= r;
}

fn cross(v1: [f32; 3], v2: [f32; 3], result: &mut [f32; 3]) {
    result[0] = v1[1] * v2[2] - v1[2] * v2[1];
    result[1] = v1[2] * v2[0] - v1[0] * v2[2];
    result[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

#[allow(clippy::too_many_arguments)]
pub fn look_at(
    eyex: f64,
    eyey: f64,
    eyez: f64,
    centerx: f64,
    centery: f64,
    centerz: f64,
    upx: f64,
    upy: f64,
    upz: f64,
) {
    let mut forward = [
        (centerx - eyex) as f32,
        (centery - eyey) as f32,
        (centerz - eyez) as f32,
    ];
    let mut up = [upx as f32, upy as f32, upz as f32];
    let mut side = [0.; 3];
    normalize(&mut forward);
    /* Side = forward x up */
    cross(forward, up, &mut side);
    normalize(&mut side);
    /* Recompute up as: up = side x forward */
    cross(side, forward, &mut up);
    let mut m = [0.; 4 * 4];
    make_identityf(&mut m);
    m[0] = side[0];
    m[4] = side[1];
    m[8] = side[2];
    m[1] = up[0];
    m[5] = up[1];
    m[9] = up[2];
    m[2] = -forward[0];
    m[6] = -forward[1];
    m[10] = -forward[2];
    unsafe {
        gl::MultMatrixf(m.as_ptr());
        gl::Translated(-eyex, -eyey, -eyez);
    }
}
