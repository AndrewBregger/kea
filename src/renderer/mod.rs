#![allow(dead_code)]

use kea::comm::{Receiver};
pub mod window;
mod rect;
mod color;
pub mod platform;
mod renderer;
pub mod style;

pub use window::Window;
pub use color::Color;
pub use rect::Rect;
use std::sync::{Weak, Mutex, Arc, MutexGuard};
pub use renderer::{Renderer, RenderContext};
use crate::gl::{self, types::*};
use log::{error, info, debug};
// use euclid::default::Transform3D;
use crate::font::{FontCollection, Font};
use pathfinder_geometry::transform3d::Transform4F;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vector4F {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vector4F {
    Vector4F {
        x,
        y,
        z,
        w
    }

}

impl Vector4F {
    pub unsafe fn as_ptr(&self) -> *const f32 {
        &self.x
    }
}

pub trait Renderable {
    fn render(&mut self, renderer: &mut Renderer);
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum RenderError {
    #[error("Failed to crate ibo")]
    IboFailed,
    #[error("Failed to create text renderer")]
    TextInitFailed,
    #[error("Failed to crate rect renderer")]
    RectInitFailed,
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub ch: char,
    pub x: f32,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    glyphs: Vec<Glyph>,
    styles: Vec<style::StyleSpan>
}

impl TextLine {
    pub fn new(glyphs: Vec<Glyph>, styles: Vec<style::StyleSpan>) -> Self {
        Self {
            glyphs,
            styles
        }
    }
}

// An interface to the rendered used by the rest of the system
// pub struct RenderContext(Weak<Mutex<Renderer>>);
// pub struct RenderCore(Arc<Mutex<Renderer>>);

impl Renderer {
    // pub fn main_loop(renderer: RenderCore, receiver: Receiver<RMessage>, window: Window<glutin::NotCurrent>) -> std::thread::JoinHandle<()> {
    //     std::thread::spawn(move || {
    //         let window = window.make_current().unwrap();
    //         assert!(window.is_current());

    //         window.init_gl();
    //         renderer.inner().init().unwrap();

    //         unsafe {
    //             gl::ClearColor(0.6, 0.6, 0.6, 1.0);
    //         }

    //         let (width, height) = window.get_size().into();
    //         renderer.inner().update_perspective(width, height);

    //         loop {
    //             let msg = match receiver.recv() {
    //                 Ok(msg) => msg,
    //                 Err(_) => {
    //                     error!("Renderer channel disconnect while still in use.");
    //                     break
    //                 }
    //             };

    //             match msg {
    //                 RMessage::Flush => {
    //                     renderer.inner().flush();
    //                 }
    //                 RMessage::WindowResize(width, height) => {
    //                     let window_size: (f32, f32) = window.get_size().into();
    //                     renderer.inner().update_perspective(width as i32, height as i32);
    //                 }
    //                 RMessage::Finalize => {
    //                     window.swap_buffers();
    //                 }
    //                 RMessage::Clear => {
    //                     unsafe {
    //                         gl::Clear(gl::COLOR_BUFFER_BIT);
    //                     }
    //                 }
    //                 RMessage::Exit => {
    //                     info!("Closing Render Loop");
    //                     break;
    //                 }
    //             }
    //         }
    //     })
    // }

    pub fn update_perspective(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
        let ortho = Transform4F::from_ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
        self.set_perspective(&ortho);
    }
}
