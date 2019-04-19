use failure::Backtrace;
#[cfg(feature = "glutin_backend")]
use glutin_window::GlutinWindow;
use opengl_graphics::OpenGL;
use piston::window::{OpenGLWindow, Size, WindowSettings};
#[cfg(feature = "sdl_backend")]
use sdl2_window::Sdl2Window;

use crate::gl;

use crate::tt::errors::GameError;
use crate::util::color::{AlphaColor, Color};

#[cfg(feature = "glutin_backend")]
type Window = GlutinWindow;
#[cfg(feature = "sdl_backend")]
type Window = Sdl2Window;

pub struct Screen {
    brightness: f32,
    fullscreen: bool,
    size: Size,
    ortho_size: Size,
    near_plane: f32,
    far_plane: f32,
    window: Option<Window>,
    luminous_screen: Option<LuminousScreen>,
}

impl Screen {
    pub fn new(brightness: f32, luminosity: f32, fullscreen: bool, size: Size) -> Self {
        Screen {
            brightness,
            fullscreen,
            size,
            ortho_size: Screen::physical_size_to_ortho_size(size),
            near_plane: 0.1,
            far_plane: 1000.,
            window: None,
            luminous_screen: if luminosity > 0. {
                Some(LuminousScreen::new(luminosity))
            } else {
                None
            },
        }
    }

    #[cfg(feature = "glutin_backend")]
    pub fn physical_size(&self) -> (f64, f64) {
        let dpi_factor = if let Some(window) = &self.window {
            window.window.get_hidpi_factor()
        } else {
            1.
        };
        (self.size.width * dpi_factor, self.size.height * dpi_factor)
    }

    #[cfg(feature = "sdl_backend")]
    pub fn physical_size(&self) -> (f64, f64) {
        (self.size.width, self.size.height)
    }

    pub fn ortho_size(&self) -> (f64, f64) {
        (self.ortho_size.width, self.ortho_size.height)
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
        let window = self
            .window_settings()
            .build()
            .map_err(|err| GameError::Fatal(err.description().to_string(), Backtrace::new()))?;
        self.init_opengl_internal(window)
    }

    #[cfg(feature = "sdl_backend")]
    pub fn init_opengl_sdl(
        &mut self,
        video_subsystem: sdl2::VideoSubsystem,
    ) -> Result<(), GameError> {
        let window = Sdl2Window::with_subsystem(video_subsystem, &self.window_settings())
            .map_err(|err| GameError::Fatal(err, Backtrace::new()))?;
        self.init_opengl_internal(window)
    }

    fn window_settings(&self) -> WindowSettings {
        let opengl = OpenGL::V2_1;
        WindowSettings::new("Torus Trooper", self.size)
            .opengl(opengl)
            .vsync(true)
            .fullscreen(self.fullscreen)
            .exit_on_esc(false)
            .controllers(false)
    }

    fn init_opengl_internal(&mut self, mut window: Window) -> Result<(), GameError> {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        self.window = Some(window);
        let size = self.size;
        self.resized(size);
        self.init();
        Ok(())
    }

    fn physical_size_to_ortho_size(physical_size: Size) -> Size {
        let ratio_threshold = 480. / 640.;
        let screen_ratio = physical_size.height / physical_size.width;
        if screen_ratio >= ratio_threshold {
            Size {
                width: 640.,
                height: 640. * screen_ratio,
            }
        } else {
            Size {
                width: 480. / screen_ratio,
                height: 480.,
            }
        }
    }

    fn screen_resized(&self) {
        let (p_width, p_height) = self.physical_size();
        unsafe {
            gl::Viewport(0, 0, p_width as i32, p_height as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            //gluPerspective(45.0f, cast(GLfloat) width / cast(GLfloat) height, nearPlane, farPlane);
            let ratio_threshold = 480. / 640.;
            let screen_ratio = p_height / p_width;
            if screen_ratio >= ratio_threshold {
                gl::Frustum(
                    f64::from(-self.near_plane),
                    f64::from(self.near_plane),
                    -f64::from(self.near_plane) * screen_ratio,
                    f64::from(self.near_plane) * screen_ratio,
                    0.1,
                    f64::from(self.far_plane),
                );
            } else {
                // This allows to see at least what can be seen horizontally and vertically
                // with the default ratio -- arnodb
                gl::Frustum(
                    -f64::from(self.near_plane) * ratio_threshold / screen_ratio,
                    f64::from(self.near_plane) * ratio_threshold / screen_ratio,
                    -f64::from(self.near_plane) * ratio_threshold,
                    f64::from(self.near_plane) * ratio_threshold,
                    0.1,
                    f64::from(self.far_plane),
                );
            }
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
    }

    pub fn resized<S: Into<Size>>(&mut self, size: S) {
        self.size = size.into();
        self.ortho_size = Screen::physical_size_to_ortho_size(self.size);
        self.screen_resized();
    }

    pub fn set_color<C: Into<Color>>(&self, color: C) {
        self.set_alpha_color(color.into())
    }
    pub fn set_alpha_color<C: Into<AlphaColor>>(&self, color: C) {
        let color = color.into();
        unsafe {
            gl::Color4f(
                color.red * self.brightness,
                color.green * self.brightness,
                color.blue * self.brightness,
                color.alpha,
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
        if let Some(luminous_screen) = &mut self.luminous_screen {
            luminous_screen.init();
        }
        self.far_plane = 10000.;
        self.screen_resized();
    }

    pub fn view_ortho_fixed(&self) {
        Screen::view_ortho(self.ortho_size.width as f32, self.ortho_size.height as f32);
    }

    pub fn view_ortho(width: f32, height: f32) {
        unsafe {
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::PushMatrix();
            gl::LoadIdentity();
            gl::Ortho(0., f64::from(width), f64::from(height), 0., -1., 1.);
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

    // Luminous

    pub fn start_render_to_luminous_screen(&self) -> bool {
        if let Some(luminous_screen) = &self.luminous_screen {
            luminous_screen.start_render();
            true
        } else {
            false
        }
    }

    pub fn end_render_to_luminous_screen(&self) {
        if let Some(luminous_screen) = &self.luminous_screen {
            luminous_screen.end_render(self);
        }
    }

    pub fn draw_luminous(&self) {
        if let Some(luminous_screen) = &self.luminous_screen {
            luminous_screen.draw(self);
        }
    }
}

const LUMINOUS_TEXTURE_WIDTH: usize = 64;
const LUMINOUS_TEXTURE_HEIGHT: usize = 64;

const LM_OFS: [(f32, f32); 2] = [(-2., -1.), (2., 1.)];
const LM_OFS_BS: f32 = 3.;

pub struct LuminousScreen {
    luminous_texture: u32,
    td: Box<[u32; LUMINOUS_TEXTURE_WIDTH * LUMINOUS_TEXTURE_HEIGHT * 4]>,
    luminosity: f32,
}

impl LuminousScreen {
    fn new(luminosity: f32) -> Self {
        LuminousScreen {
            luminous_texture: 0,
            td: Box::new([0 as u32; LUMINOUS_TEXTURE_WIDTH * LUMINOUS_TEXTURE_HEIGHT * 4]),
            luminosity,
        }
    }

    fn init(&mut self) {
        self.make_kuminous_texture();
    }

    fn make_kuminous_texture(&mut self) {
        unsafe {
            gl::GenTextures(1, &mut self.luminous_texture);
            gl::BindTexture(gl::GL_TEXTURE_2D, self.luminous_texture);
            gl::TexImage2D(
                gl::GL_TEXTURE_2D,
                0,
                4,
                LUMINOUS_TEXTURE_WIDTH as i32,
                LUMINOUS_TEXTURE_HEIGHT as i32,
                0,
                gl::GL_RGBA,
                gl::GL_UNSIGNED_BYTE,
                self.td.as_ptr() as *const std::ffi::c_void,
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
    }

    fn start_render(&self) {
        unsafe {
            gl::Viewport(
                0,
                0,
                LUMINOUS_TEXTURE_WIDTH as i32,
                LUMINOUS_TEXTURE_HEIGHT as i32,
            );
        }
    }

    fn end_render(&self, screen: &Screen) {
        unsafe {
            gl::BindTexture(gl::GL_TEXTURE_2D, self.luminous_texture);
            gl::CopyTexImage2D(
                gl::GL_TEXTURE_2D,
                0,
                gl::GL_RGBA,
                0,
                0,
                LUMINOUS_TEXTURE_WIDTH as i32,
                LUMINOUS_TEXTURE_HEIGHT as i32,
                0,
            );
        }
        let (p_width, p_height) = screen.physical_size();
        unsafe {
            gl::Viewport(0, 0, p_width as i32, p_height as i32);
        }
    }

    fn draw(&self, screen: &Screen) {
        unsafe {
            gl::Enable(gl::GL_TEXTURE_2D);
            gl::BindTexture(gl::GL_TEXTURE_2D, self.luminous_texture);
        }
        let (p_width, p_height) = screen.physical_size();
        Screen::view_ortho(p_width as f32, p_height as f32);
        unsafe {
            gl::Color4f(1., 0.8, 0.9, self.luminosity);
            gl::Begin(gl::GL_QUADS);
            for lm_ofs in &LM_OFS {
                gl::TexCoord2f(0., 1.);
                gl::Vertex2f(lm_ofs.0 * LM_OFS_BS, lm_ofs.1 * LM_OFS_BS);
                gl::TexCoord2f(0., 0.);
                gl::Vertex2f(lm_ofs.0 * LM_OFS_BS, p_height as f32 + lm_ofs.1 * LM_OFS_BS);
                gl::TexCoord2f(1., 0.);
                gl::Vertex2f(
                    p_width as f32 + lm_ofs.0 * LM_OFS_BS,
                    p_height as f32 + lm_ofs.1 * LM_OFS_BS,
                );
                gl::TexCoord2f(1., 1.);
                gl::Vertex2f(p_width as f32 + lm_ofs.0 * LM_OFS_BS, lm_ofs.1 * LM_OFS_BS);
            }
            gl::End();
        }
        Screen::view_perspective();
        unsafe {
            gl::Disable(gl::GL_TEXTURE_2D);
        }
    }
}

impl Drop for LuminousScreen {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.luminous_texture);
        }
    }
}
