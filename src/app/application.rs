use std::sync::{Arc, Mutex, Weak, MutexGuard};
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::BTreeMap;

use kea;

use crate::glutin::{event_loop::EventLoop, PossiblyCurrent};
// use crate::euclid::{default::Vector2D, vec2};
use crate::pathfinder_geometry::vector::{Vector2F, vec2f};
use crate::renderer::{Renderer, RenderContext, Window, window::LogicalSize, Rect, Color, Renderable, Glyph, TextLine};
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
    // renderer: Renderer,
    /// the main window for this application
    window: Window<PossiblyCurrent>,
    // /// a duplex for communicating with the core thread.
    sender: Sender<Edit>,
    /// is a redraw requested.
    draw_requested: bool,
    /// the current configuration of the application. This should be a global configuration.
    config: Config,
    /// frames this application contains
    frames: BTreeMap<FrameId, Frame>,
    /// how the frames are positioned on the screen
    layout: FrameLayout,
    /// frame actively being interacted with
    active_frame: Option<FrameId>,
	/// the editor core.
    core: KeaCore,
    /// runtime information for the renderer.
    context: RenderContext
}

impl Application {
    pub fn with_config(window: Window<PossiblyCurrent>, sender: Sender<Edit>, font_collection: FontCollection, config: Config) -> Result<Self, super::AppError> {
        let el = EventLoop::<AppEvent>::with_user_event();
        let core = KeaCore::new(&config);

        Ok(Self {
            // renderer: context,
            window,
            sender,
            draw_requested: true,
            config,
            frames: BTreeMap::new(),
            layout: FrameLayout::new(),
            active_frame: None,
            core,
            context: RenderContext::new(font_collection),
        })
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.draw_requested = true;
        // self.renderer.update_perspective(width as i32, height as i32);
        // do what ever is needed here for the rest of the app to update.
    }

    pub fn draw_requested(&self) -> bool {
        self.draw_requested
    }

    pub fn swap_buffer(&self) {
        self.window.swap_buffers()
    }

    pub fn on_init(&mut self) {
        let window_size = self.window.get_size();
        let size = vec2f(window_size.width as f32, window_size.height as f32);
        let origin = Vector2F::zero();

        let metrics = self.context.fonts().default_font()
        	.metrics()
        	.scale_with(self.config.font_size(), self.window.dpi_factor() as f32);

		let mut core = self.core.inner();

		let buffer_id = core.open_file(core::BufferInfo { path: Some(PathBuf::from_str("src/main.rs").unwrap())  }).unwrap();
		let buffer = core.get_buffer(&buffer_id).unwrap();

        let lines = Frame::compute_lines(size.y(), &metrics);
		let mut frame = Frame::new(buffer_id, size, origin, lines);
		frame.update_line_cache(Invalidation::Init, buffer);

		frame.set_active(true);

		let frame_id = frame.id();

		self.active_frame = Some(frame_id);
		self.frames.insert(frame_id, frame);
		self.layout.push_frame(FrameInfo { frame: frame_id });
    }

    fn position_line(line: &Text<TextLine>, font: &Font) -> TextLine {
        let mut glyphs = Vec::new();

        let mut x = 0.0;
        for ch in line.text.chars() {
            if let Some(info) = font.info(ch) {
                let glyph = Glyph {
                    ch,
                    x,
                };

                glyphs.push(glyph);
                x += info.advance.x();
            }
        }

        TextLine::new(glyphs, line.styles.clone())
    }
}

impl Renderable for Application {
    fn render(&mut self, renderer: &mut Renderer) {
        for frame in self.layout.frame_iter() {
            // println!("{:?}", frame);
            if let Some(frame) = self.frames.get_mut(&frame.frame) {
                let width = frame.width();
        		let height = frame.height();
                let origin = frame.origin();
                let font = self.context.fonts().default_font();
                let metrics = font.metrics().scale_with(self.config.font_size(), font.device_pixel_ratio);
                let start_y = height - metrics.ascent as f32;
        		let start_x = 2.0;

        		let x = start_x + origin.x();
        		let mut y = start_y + origin.y();

        		for line in frame.lines_mut() {
        			if let Some(line) = line {

                        if line.assoc.is_none() {
                            // generate glyphs
                            let text_line = Self::position_line(&line, font);
                            line.assoc = Some(text_line);
                        }

                        if let Some(text) = line.assoc.as_ref() {
                            // render glyphs.
                            renderer.render_line(&self.context, text, x, y, self.config.font_size());

                            if !line.cursors.is_empty() {
                                renderer.render_cursors(&self.context, text, line.cursors.as_slice(), y, self.config.font_size());
                            }
                        }
        			}
                    y -= metrics.line_height();
        		}
            }
        }

        self.draw_requested = false;
    }
}
