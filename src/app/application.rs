use std::sync::{Arc, Mutex};

use super::{Config, AppEvent, AppError};
use crate::glutin::{event_loop::EventLoop, PossiblyCurrent};
use crate::euclid::default::Vector2D;
use crate::kea::comm::{Duplex, duplex, channel, Sender};
use crate::renderer::{Renderer, Window, window::LogicalSize, Rect, Color};
use crate::core::KeaCore;
use crate::ui::*;

use log::{info, debug, error};

pub struct Application {
    /// an interface for the application to interact with the renderer.
    renderer: Renderer,
    /// the main window for this application
    window: Window<PossiblyCurrent>,
    /// The editing core engine.
    // This doesn't need to be an arc mutex because the underlining type uses one.
    core: Box<KeaCore>,

    // /// a duplex for communicating with the core thread.
    // duplex: Duplex<(), ()>
    draw_requested: bool,

    config: Config,
}

impl Application {
    pub fn with_config(context: Renderer, window: Window<PossiblyCurrent>, config: Config) -> Result<Self, super::AppError> {
        let el = EventLoop::<AppEvent>::with_user_event();

        Ok(Self {
            renderer: context,
            window,
            core: Box::new(KeaCore::new(&config)),
            // core: Arc::new(Mutex::new(KeaCore::new(&config))),
            // duplex: app_duplex,
            draw_requested: true,
            config,
        })
    }

    pub fn as_arc(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    pub fn test_render(&self) {
    }

    fn render(&mut self, ctx: &mut Renderer) {
    }

    pub fn maybe_render(&mut self) {
        if self.draw_requested {
            self.renderer.clear();
            self.renderer.render_str("Hello, World", 300f32, 10f32, Color::black(), Color::rgb(0.7, 0.7, 0.7), self.config.font_desc(), self.config.font_size());
            self.renderer.flush();
            self.window.swap_buffers();
            // // render the updated state of the screen.
            // self.render(&mut guard);
            // flush the rest of the buffer
            self.draw_requested = false;
        }
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.draw_requested = true;
        self.renderer.update_perspective(width as i32, height as i32);
        // do what ever is needed here for the rest of the app to update.
    }
}