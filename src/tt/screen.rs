#[cfg(feature = "glutin_backend")]
use glutin_window::GlutinWindow as Window;
use opengl_graphics::OpenGL;
use piston::window::{OpenGLWindow, Size, WindowSettings};
#[cfg(feature = "sdl_backend")]
use sdl2_window::Sdl2Window as Window;

use crate::gl;

use crate::tt::errors::GameError;

pub struct Screen {
    brightness: f32,
    size: Size,
    near_plane: f32,
    far_plane: f32,
    window: Option<Window>,
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            brightness: 1.,
            size: [640, 480].into(),
            near_plane: 0.1,
            far_plane: 1000.,
            window: None,
        }
    }

    #[cfg(feature = "glutin_backend")]
    pub fn physical_size(&self) -> (u32, u32) {
        let dpi_factor = if let Some(window) = &self.window {
            window.window.get_hidpi_factor()
        } else {
            1.
        };
        ((self.size.width as f64 * dpi_factor) as u32, (self.size.height as f64 * dpi_factor) as u32)
    }

    #[cfg(feature = "sdl_backend")]
    pub fn physical_size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn window_mut(&mut self) -> Option<&mut Window> {
        self.window.as_mut()
    }

    // Screen3D

    pub fn init_opengl(&mut self) -> Result<(), GameError> {
        let opengl = OpenGL::V2_1;
        let mut window: Window = WindowSettings::new("Torus Trooper", self.size)
            .opengl(opengl)
            .vsync(true)
            .exit_on_esc(false)
            .build()?;

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        self.window = Some(window);
        let size = self.size;
        self.resized(size);
        self.init();
        Ok(())
    }

    fn screen_resized(&self) {
        let p_size = self.physical_size();
        unsafe {
            gl::Viewport(0, 0, p_size.0 as i32, p_size.1 as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            //gluPerspective(45.0f, cast(GLfloat) width / cast(GLfloat) height, nearPlane, farPlane);
            let ratio_threshold = 480. / 640.;
            let screen_ratio = p_size.1 as f32 / p_size.0 as f32;
            if screen_ratio >= ratio_threshold {
                gl::Frustum(
                    -self.near_plane as f64,
                    self.near_plane as f64,
                    (-self.near_plane * screen_ratio) as f64,
                    (self.near_plane * screen_ratio) as f64,
                    0.1,
                    self.far_plane as f64,
                );
            } else {
                // This allows to see at least what can be seen horizontally and vertically
                // with the default ratio -- arnodb
                gl::Frustum(
                    (-self.near_plane * ratio_threshold / screen_ratio) as f64,
                    (self.near_plane * ratio_threshold / screen_ratio) as f64,
                    (-self.near_plane * ratio_threshold) as f64,
                    (self.near_plane * ratio_threshold) as f64,
                    0.1,
                    self.far_plane as f64,
                );
            }
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
    }

    pub fn resized<S: Into<Size>>(&mut self, size: S) {
        self.size = size.into();
        self.screen_resized();
    }

    pub fn set_color_rgb(&self, r: f32, g: f32, b: f32) {
        self.set_color_rgba(r, g, b, 1.)
    }
    pub fn set_color_rgba(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::Color4f(
                r * self.brightness,
                g * self.brightness,
                b * self.brightness,
                a,
            );
        }
    }

    pub fn set_clear_color_rgb(&self, r: f32, g: f32, b: f32) {
        self.set_clear_color_rgba(r, g, b, 1.)
    }
    pub fn set_clear_color_rgba(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(
                r * self.brightness,
                g * self.brightness,
                b * self.brightness,
                a,
            );
        }
    }

    // Screen

    pub fn clear() {
        unsafe {
            gl::Clear(gl::GL_COLOR_BUFFER_BIT);
        }
    }

    fn init(&mut self) {
        unsafe {
            gl::LineWidth(1.);
            gl::BlendFunc(gl::GL_SRC_ALPHA, gl::GL_ONE);
            gl::Enable(gl::GL_LINE_SMOOTH);
            gl::Enable(gl::GL_BLEND);
            gl::Disable(gl::GL_COLOR_MATERIAL);
            gl::Disable(gl::GL_CULL_FACE);
            gl::Disable(gl::GL_DEPTH_TEST);
            gl::Disable(gl::GL_LIGHTING);
            gl::Disable(gl::GL_TEXTURE_2D);
        }
        self.set_clear_color_rgba(0., 0., 0., 1.);
        self.far_plane = 10000.;
        self.screen_resized();
    }

    pub fn view_ortho_fixed() {
        unsafe {
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::PushMatrix();
            gl::LoadIdentity();
            gl::Ortho(0., 640., 480., 0., -1., 1.);
            gl::MatrixMode(gl::GL_MODELVIEW);
            gl::PushMatrix();
            gl::LoadIdentity();
        }
    }

    pub fn view_perspective() {
        unsafe {
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::PopMatrix();
            gl::MatrixMode(gl::GL_MODELVIEW);
            gl::PopMatrix();
        }
    }
}
