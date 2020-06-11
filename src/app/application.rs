use std::sync::{Arc, Mutex, Weak, MutexGuard};
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::BTreeMap;

use kea;

use crate::glutin::{event_loop::EventLoop, PossiblyCurrent};
// use crate::euclid::{default::Vector2D, vec2};
use crate::pathfinder_geometry::vector::{Vector2F, vec2f};
use crate::renderer::{Renderer, Window, window::LogicalSize, Rect, Color};
use crate::core::{self, KeaCore, Edit, Update};
use crate::ui::*;
use crate::font::{Font, FontCollection};
use kea::comm::{Duplex, duplex, channel, Sender};
use super::{Config, AppEvent, AppError};

use log::{info, debug, error};


pub struct App(Arc<Mutex<Application>>);

impl App {
    pub fn new(app: Application, ) -> Self {
        App(Arc::new(Mutex::new(app)))
    }

    pub fn inner(&self) -> MutexGuard<Application> {
        self.0.lock().unwrap()
    }

    pub fn weak(&self) -> WeakApp {
        WeakApp(Arc::downgrade(&self.0))
    }
}


pub struct WeakApp(Weak<Mutex<Application>>);

impl WeakApp {
    pub fn upgrade(&self) ->  App {
        match self.0.upgrade() {
            Some(core) => App(core),
            None => panic!("Kea Core was not created"),
        }
    }
}

pub struct Application {
    /// an interface for the application to interact with the renderer.
    renderer: Renderer,
    /// the main window for this application
    window: Window<PossiblyCurrent>,
    // /// a duplex for communicating with the core thread.
    sender: Sender<Edit>,
    /// is a redraw requested.
    draw_requested: bool,
    /// the current configuration of the application. This should be a global configuration.
    config: Config,
    /// frames this application contains
    frames: BTreeMap<FrameId, Box<()>>,
    /// how the frames are positioned on the screen
    layout: FrameLayout,
    /// a collection of all of the raw font data needed by the application.
    font_collection: FontCollection,
}

impl Application {
    pub fn with_config(context: Renderer, window: Window<PossiblyCurrent>, sender: Sender<Edit>, font_collection: FontCollection, config: Config) -> Result<Self, super::AppError> {
        let el = EventLoop::<AppEvent>::with_user_event();



        Ok(Self {
            renderer: context,
            window,
            sender,
            draw_requested: true,
            config,
            frames: BTreeMap::new(),
            layout: FrameLayout::new(),
            font_collection
        })
    }

    pub fn test_render(&self) {
    }

    fn render(&mut self) {
    }

    pub fn maybe_render(&mut self) {
        if self.draw_requested {
            self.renderer.clear();

            self.renderer.render_str("\'Hello, World\"", 300f32, 40f32, Color::black(), Color::rgb(0.7, 0.7, 0.7), self.font_collection.default_font(), self.config.font_size());

            self.renderer.flush();
            self.window.swap_buffers();

            // // render the updated state of the screen.
            // self.render();
            // flush the rest of the buffer
            self.draw_requested = false;
        }
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.draw_requested = true;
        self.renderer.update_perspective(width as i32, height as i32);
        // do what ever is needed here for the rest of the app to update.
    }

    pub fn on_init(&mut self) {
        // let window_size = self.window.get_size();
        // let size = vec2f(window_size.width as f32, window_size.height as f32);
        // let origin = Vector2F::zero();

        // let frame_info = FrameInfo {
        //     path: Some(PathBuf::from_str("src/main.rs").unwrap()),
        //     size,
        // }

        // for testing. This might be removed or it can be used to initialize the application.
        // let path = PathBuf::from_str("src/main.rs").unwrap();
        // let edit = Edit::OpenFile(path);
        // self.sender.send(edit).unwrap();

        // let view_info = core::ViewInfo {};
        // let view = Edit::NewView(view_info);
        // self.sender.send(view).unwrap();
    }
}
