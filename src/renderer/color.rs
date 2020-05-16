#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[inline]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    #[inline]
    pub fn red() -> Color {
        Color::rgb(1.0f32, 0f32, 0f32)
    }

    #[inline]
    pub fn green() -> Color {
        Color::rgb(0.0f32, 1f32, 0f32)
    }

    #[inline]
    pub fn blue() -> Color {
        Color::rgb(0.0f32, 0f32, 1f32)
    }

    #[inline]
    pub fn grey() -> Color {
        Color::rgb(0.5f32, 0.5f32, 0.5f32)
    }

    #[inline]
    pub fn white() -> Color {
        Color::rgb(1.0f32, 1f32, 1f32)
    }

    #[inline]
    pub fn black() -> Color {
        Color::rgb(0.0f32, 0.0f32, 0.0f32)
    }

    pub fn uniform(ch: f32) -> Color { Color::rgb(ch, ch, ch) }
}
