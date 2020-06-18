use std::convert::From;

// use euclid::default::Vector2D;
use super::Color;
use crate::glutin::dpi::{LogicalPosition, LogicalSize, Pixel};
use crate::pathfinder_geometry::vector::Vector2F;

#[derive(Debug, Clone)]
pub struct Rect {
    pub pos: Vector2F,
    pub width: f32,
    pub height: f32,
    pub bg_color: Color,
}

impl Rect {
    pub fn with_position(pos: Vector2F, width: f32, height: f32) -> Self {
        Self {
            pos,
            width,
            height,
            bg_color: Color::black(),
        }
    }

    pub fn new(width: f32, height: f32) -> Self {
        Self {
            pos: Vector2F::zero(),
            width,
            height,
            bg_color: Color::black(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    pub fn set_color(&mut self, color: Color) {
        self.bg_color = color
    }
}

impl<T> From<LogicalSize<T>> for Rect
where
    T: Pixel,
{
    fn from(val: LogicalSize<T>) -> Self {
        let (w, h) = val.into();
        Self::new(w, h)
    }
}
