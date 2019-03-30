use failure::Backtrace;

#[derive(Fail, Debug)]
pub enum GameError {
    #[fail(display = "Preferences error")]
    Preference(#[cause] preferences::PreferencesError, Backtrace),
    #[fail(display = "Image error")]
    Image(#[cause] image::ImageError, Backtrace),
    #[fail(display = "Sound error")]
    Sound(#[cause] SoundError, Backtrace),
    #[fail(display = "BulletML parse error")]
    BulletML(#[cause] bulletml::parse::Error, Backtrace),
    #[fail(display = "Fatal error")]
    Fatal(String, Backtrace),
}

impl From<preferences::PreferencesError> for GameError {
    fn from(inner: preferences::PreferencesError) -> Self {
        GameError::Preference(inner, Backtrace::new())
    }
}

impl From<image::ImageError> for GameError {
    fn from(inner: image::ImageError) -> Self {
        GameError::Image(inner, Backtrace::new())
    }
}

impl From<SoundError> for GameError {
    fn from(inner: SoundError) -> Self {
        GameError::Sound(inner, Backtrace::new())
    }
}

impl From<bulletml::parse::Error> for GameError {
    fn from(inner: bulletml::parse::Error) -> Self {
        GameError::BulletML(inner, Backtrace::new())
    }
}

#[derive(Fail, Debug)]
pub enum SoundError {
    #[fail(display = "I/O error")]
    InputOutput(#[cause] std::io::Error, Backtrace),
    #[fail(display = "Sdl")]
    Sdl(String, Backtrace),
}

impl From<std::io::Error> for SoundError {
    fn from(inner: std::io::Error) -> Self {
        SoundError::InputOutput(inner, Backtrace::new())
    }
}
