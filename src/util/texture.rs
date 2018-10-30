use crate::gl;

pub struct Texture {
    num: u32,
}

impl Texture {
    pub fn create(name: &str) -> Result<Self, image::ImageError> {
        let img = image::open("images/".to_string() + name)?;
        if let image::DynamicImage::ImageRgb8(rgb_img) = img {
            let mut num: u32 = 0;
            unsafe {
                gl::GenTextures(1, &mut num);
                gl::BindTexture(gl::GL_TEXTURE_2D, num);
                gl::TexImage2D(
                    gl::GL_TEXTURE_2D,
                    0,
                    3,
                    rgb_img.dimensions().0 as i32,
                    rgb_img.dimensions().1 as i32,
                    0,
                    gl::GL_RGB,
                    gl::GL_UNSIGNED_BYTE,
                    rgb_img.into_raw().as_ptr() as *const std::ffi::c_void,
                );
                gl::TexParameteri(
                    gl::GL_TEXTURE_2D,
                    gl::GL_TEXTURE_MIN_FILTER,
                    gl::GL_LINEAR as i32,
                );
                gl::TexParameteri(
                    gl::GL_TEXTURE_2D,
                    gl::GL_TEXTURE_MAG_FILTER,
                    gl::GL_LINEAR as i32,
                );
            }
            Ok(Texture { num })
        } else {
            panic!("Image {} should be of type RGB8!", name);
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::GL_TEXTURE_2D, self.num) }
    }
}
