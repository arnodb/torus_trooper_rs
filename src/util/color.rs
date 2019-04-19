#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    #[inline]
    fn from_rgb(red: f32, green: f32, blue: f32) -> Self {
        Color { red, green, blue }
    }

    #[inline]
    pub fn with_alpha(&self, alpha: f32) -> AlphaColor {
        AlphaColor::from_color_and_alpha(self, alpha)
    }
}

impl From<(f32, f32, f32)> for Color {
    #[inline]
    fn from(rgb: (f32, f32, f32)) -> Self {
        Color::from_rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Color {
        Color {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct AlphaColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl AlphaColor {
    #[inline]
    fn from_rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::from_rgba(red, green, blue, 1.)
    }

    #[inline]
    fn from_rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        AlphaColor {
            red,
            green,
            blue,
            alpha,
        }
    }

    #[inline]
    fn from_color(color: &Color) -> Self {
        Self::from_rgb(color.red, color.green, color.blue)
    }

    #[inline]
    fn from_color_and_alpha(color: &Color, alpha: f32) -> Self {
        Self::from_rgba(color.red, color.green, color.blue, alpha)
    }
}

impl From<(f32, f32, f32, f32)> for AlphaColor {
    #[inline]
    fn from(rgba: (f32, f32, f32, f32)) -> Self {
        AlphaColor::from_rgba(rgba.0, rgba.1, rgba.2, rgba.3)
    }
}

impl From<Color> for AlphaColor {
    #[inline]
    fn from(color: Color) -> Self {
        AlphaColor::from_color(&color)
    }
}

impl From<(Color, f32)> for AlphaColor {
    #[inline]
    fn from(color_alpha: (Color, f32)) -> Self {
        AlphaColor::from_color_and_alpha(&color_alpha.0, color_alpha.1)
    }
}
