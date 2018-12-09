use failure::Backtrace;

#[derive(Fail, Debug)]
pub enum GameError {
    #[fail(display = "Preferences error")]
    Preference(#[cause] preferences::PreferencesError, Backtrace),
    #[fail(display = "Image error")]
    Image(#[cause] image::ImageError, Backtrace),
    #[fail(display = "BulletML parse error")]
    BulletML(#[cause] bulletml::parse::Error, Backtrace),
    #[fail(display = "String error")]
    String(String, Backtrace),
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

impl From<bulletml::parse::Error> for GameError {
    fn from(inner: bulletml::parse::Error) -> Self {
        GameError::BulletML(inner, Backtrace::new())
    }
}

impl From<String> for GameError {
    fn from(inner: String) -> Self {
        GameError::String(inner, Backtrace::new())
    }
}
