use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct GameError {
    inner: Context<GameErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum GameErrorKind {
    #[fail(display = "Joystick error")]
    Joystick,
    #[fail(display = "Preferences error")]
    Preference,
    #[fail(display = "Sound initialization error")]
    SoundInit,
    #[fail(display = "Texture error")]
    Texture,

    #[fail(display = "Window initialization error")]
    WindowInit,
    #[fail(display = "Missing window")]
    MissingWindow,

    #[fail(display = "Barrage error")]
    Barrage,
    #[fail(display = "BulletML error")]
    BulletML,

    #[fail(display = "SDL2 initialization error")]
    Sdl2Init,
    #[fail(display = "SDL2 video initialization error")]
    Sdl2VideoInit,
    #[fail(display = "SDL2 audio initialization error")]
    Sdl2AudioInit,
}

impl Fail for GameError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl GameError {
    pub fn kind(&self) -> GameErrorKind {
        *self.inner.get_context()
    }
}

impl From<GameErrorKind> for GameError {
    fn from(kind: GameErrorKind) -> GameError {
        GameError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<GameErrorKind>> for GameError {
    fn from(inner: Context<GameErrorKind>) -> GameError {
        GameError { inner: inner }
    }
}
