use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use kea;
use kea::Ptr;
use kea::comm::{channel, duplex, Duplex, Sender};
use log::{debug, error, info};

// use crate::euclid::{default::Vector2D, vec2};
use crate::core::{self, Core, Edit, KeaCore, Update};
use crate::font::{Font, FontCollection};
use crate::glutin::{
    event::{KeyboardInput, VirtualKeyCode, ModifiersState, ElementState},
    event_loop::EventLoop,
    PossiblyCurrent,
};
use crate::pathfinder_geometry::vector::{vec2f, Vector2F};
use crate::renderer::{
    window::LogicalSize, Color, Glyph, Rect, RenderContext, Renderable, Renderer, TextLine, Window,
};
use crate::ui::*;

use super::{AppError, AppEvent, Config};
use crate::renderer::window::event::WindowEvent::{CursorEntered, CursorMoved};
use std::path::Component::CurDir;

pub struct App(Arc<Mutex<Application>>);

impl App {
    pub fn new(app: Application) -> Self {
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
    pub fn upgrade(&self) -> App {
        match self.0.upgrade() {
            Some(core) => App(core),
            None => panic!("Kea Core was not created"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EditMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy)]
struct EditState {
    mode: EditMode,
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
    core: Core,
    /// runtime information for the renderer.
    context: RenderContext,
    /// state of the editor
    state: EditState,
}

impl Application {
    pub fn with_config(
        window: Window<PossiblyCurrent>,
        sender: Sender<Edit>,
        font_collection: FontCollection,
        config: Config,
    ) -> Result<Self, super::AppError> {
        let el = EventLoop::<AppEvent>::with_user_event();
        let core = Core::new();

        let font_size = config.font_size();
        let dpi_factor = window.dpi_factor();

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
            context: RenderContext::new(font_collection, font_size, dpi_factor as f32),
            state: EditState {
                mode: EditMode::Normal,
            },
        })
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.draw_requested = true;
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

        let metrics = self
            .context
            .fonts()
            .default_font()
            .metrics()
            .scale_with(self.config.font_size(), self.window.dpi_factor() as f32);

        let buffer_id = self
            .core
            .open_file(core::BufferInfo {
                path: Some(PathBuf::from_str("src/main.rs").unwrap()),
            })
            .unwrap();
        let buffer = self.core.get_buffer_ptr(&buffer_id).unwrap();

        let lines = Frame::compute_lines(size.y(), &metrics);
        let mut frame = Frame::new(buffer, size, origin, lines);
        frame.update_line_cache(Invalidation::Init);

        frame.set_active(true);

        let frame_id = frame.id();

        self.active_frame = Some(frame_id);
        self.frames.insert(frame_id, frame);
        self.layout.push_frame(FrameInfo { frame: frame_id });
    }

    pub fn handle_keyboard_input(&mut self, input: KeyboardInput, modifiers: &ModifiersState) {
        // let edit_mode = self.state.mode;
        // if let Some(operation) =
        debug!(
            "Keyboard Input: {:?} Modifier State: {:?}",
            input, modifiers
        );


		// for now I do not want to handle release.
		if input.state == ElementState::Released {
    		return;
		}

        if let Some(key) = input.virtual_keycode {
            match key {
                VirtualKeyCode::Up => {
                    if let Some(frame) = self.active_frame_mut() {
                    }
                }
                VirtualKeyCode::Down => {
                    println!("Handling Down");
                    if let Some(frame) = self.active_frame_mut() {
                    }
                }
                VirtualKeyCode::Left => {
                    if let Some(frame) = self.active_frame_mut() {
                    }
                }
                VirtualKeyCode::Right => {
                    if let Some(frame) = self.active_frame_mut() {
                    }
                }
                _ => {}
            }
        }
    }

    pub fn active_frame(&self) -> Option<&Frame> {
        if let Some(id) = self.active_frame.as_ref() {
            self.frames.get(id)
        } else {
            info!("No Active Frame");
            None
        }
    }

    pub fn active_frame_mut(&mut self) -> Option<&mut Frame> {
        if let Some(id) = self.active_frame.as_ref() {
            self.frames.get_mut(id)
        } else {
            info!("No Active Frame");
            None
        }
    }
}

impl Renderable for Application {
    fn render(&mut self, renderer: &mut Renderer) {
        for frame in self.layout.frame_iter() {
            // println!("{:?}", frame);
            if let Some(frame) = self.frames.get_mut(&frame.frame) {
                renderer.render_frame(&self.context, frame);
            }
        }

        self.draw_requested = false;
    }
}
