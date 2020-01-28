#[cfg(nightly)]
use std::backtrace::Backtrace;

#[derive(Error, Debug, new)]
pub enum GameError {
    #[error("Joystick error")]
    Joystick {
        source: Box<dyn std::error::Error>,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("Preferences error")]
    Preferences {
        #[from]
        source: preferences::PreferencesError,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("Sound initialization error")]
    SoundInit {
        source: Box<dyn std::error::Error>,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("Texture error")]
    Image {
        #[from]
        source: image::ImageError,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },

    #[error("Window initialization error")]
    WindowInit {
        source: Box<dyn std::error::Error>,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("Missing window")]
    MissingWindow(#[cfg(nightly)] Backtrace),

    #[error("Barrage error")]
    Barrage {
        source: Box<dyn std::error::Error>,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("BulletML error")]
    BulletML {
        #[from]
        source: bulletml::errors::ParseError,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },

    #[error("SDL2 initialization error: {error}")]
    Sdl2Init {
        error: String,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("SDL2 video initialization error: {error}")]
    Sdl2VideoInit {
        error: String,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
    #[error("SDL2 audio initialization error: {error}")]
    Sdl2AudioInit {
        error: String,
        #[cfg(nightly)]
        #[new(value = "Backtrace::capture()")]
        backtrace: Backtrace,
    },
}
