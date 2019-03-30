include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

pub const GL_ONE: u32 = 1;

pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_LOOP: u32 = 0x0002;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_TRIANGLES: u32 = 0x0004;
pub const GL_TRIANGLE_FAN: u32 = 0x0006;
pub const GL_QUADS: u32 = 0x0007;
pub const GL_SRC_ALPHA: u32 = 0x302;
pub const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
pub const GL_LINE_SMOOTH: u32 = 0x0B20;
pub const GL_CULL_FACE: u32 = 0x0B44;
pub const GL_LIGHTING: u32 = 0x0B50;
pub const GL_COLOR_MATERIAL: u32 = 0x0B57;
pub const GL_DEPTH_TEST: u32 = 0x0B71;
pub const GL_BLEND: u32 = 0x0BE2;
pub const GL_TEXTURE_2D: u32 = 0x0DE1;
pub const GL_COMPILE: u32 = 0x1300;
pub const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub const GL_MODELVIEW: u32 = 0x1700;
pub const GL_PROJECTION: u32 = 0x1701;
pub const GL_RGB: u32 = 0x1907;
pub const GL_RGBA: u32 = 0x1908;
pub const GL_LINEAR: u32 = 0x2601;
pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2801;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
