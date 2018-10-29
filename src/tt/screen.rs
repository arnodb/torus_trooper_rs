use glutin_window::GlutinWindow as Window;
use opengl_graphics::OpenGL;
use piston::window::WindowSettings;

use crate::gl;

use crate::tt::errors::GameError;

const CAPTION: &str = "Torus Trooper";

pub struct Screen {
    brightness: f32,
    width: u32,
    height: u32,
    near_plane: f32,
    far_plane: f32,
    window: Option<Window>,
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            brightness: 1.,
            width: 640,
            height: 480,
            near_plane: 0.1,
            far_plane: 1000.,
            window: None,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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
        let window: Window = WindowSettings::new("Torus Trooper", [self.width, self.height])
            .opengl(opengl)
            .vsync(true)
            .exit_on_esc(false)
            .build()?;
        let gl_window = &window.window;
        use glutin::GlContext;

        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        self.window = Some(window);

        self.resized(self.width, self.height);
        self.init();
        Ok(())
    }

    fn screen_resized(&self) {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            //gluPerspective(45.0f, cast(GLfloat) width / cast(GLfloat) height, nearPlane, farPlane);
            gl::Frustum(
                -self.near_plane as f64,
                self.near_plane as f64,
                (-self.near_plane * self.height as f32 / self.width as f32) as f64,
                (self.near_plane * self.height as f32 / self.width as f32) as f64,
                0.1,
                self.far_plane as f64,
            );
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.screen_resized();
    }

    pub fn set_caption(&self, _name: &str) {
        // TODO
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
        self.set_caption(CAPTION);
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
