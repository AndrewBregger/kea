
mod config;
mod application;
mod event_handler;

pub use config::{Config};
use kea::{self, utils::log_file_path, comm::Receiver};
use super::renderer::{self, Renderer, window::{Window, LogicalSize}};
use application::{Application, App, WeakApp};
use event_handler::{EventHandler};
use std::sync::{Arc, Mutex, Weak};
use crate::font::{FontCollection, GlyphId, Font, FontMetrics};
use crate::renderer::platform::atlas::FontAtlas;
use crate::core::{self, KeaCore, Edit, Update};

use glutin::event_loop::EventLoop;

use log::{error, info};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Exit,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    WindowError(crate::renderer::window::WindowError),
    #[error("{0}")]
    FontError(crate::font::FontError),
    #[error("{0}")]
    RenderError(crate::renderer::RenderError),
}

pub fn run(config: Config) -> Result<(), AppError> {
    let event_loop = EventLoop::<AppEvent>::with_user_event();
    let window = Window::<glutin::NotCurrent>::new(&event_loop, LogicalSize::new(500 as _, 400 as _), "kea", &config).unwrap();
    let (app_duplex, core_duplex) = kea::comm::duplex::<Edit, Update>();
    let (app_sender, app_receiver) = app_duplex.decompose();

    let dpi = window.dpi_factor() as f32;

    let mut font_collection = FontCollection::new(dpi).unwrap();
    let font_desc = config.font_desc();
    let font_size = config.font_size();

    match font_collection.add_font_by_name(config.font_name()) {
        Ok(_) => {}
        Err(e) => {
            // add status message
            println!("{}", e);
            font_collection.add_default();
        }
    }

    // the font needs to be loaded on this thread because it cannot be between across thread boundaries.
    let window = window.make_current().map_err(AppError::WindowError)?;
    window.init_gl();

    let font_atlas = FontAtlas::from_collection(&font_collection, |atlas, font| {
        // println!("Loading Font: {:?}", font.desc());
        for c in 32 as u8 .. 127 as u8 {
            let rglyph = font.rasterize_glyph(c as char, font_size)?;
            atlas.add_glyph(&rglyph);
        }
        Ok(())
    }).map_err(AppError::FontError)?;

    let mut renderer = Renderer::new();
    renderer.init().map_err(|e| {
        error!("Failed to initialize render: {:?}", e);
        AppError::RenderError(e)
    })?;

    renderer.set_atlas(font_atlas);

    let (width, height) = window.get_size().into();
    renderer.update_perspective(width, height);

    // let core = KeaCore::new(&config);

    let elp = event_loop.create_proxy();
    let mut event_handler = EventHandler::new(elp);

    let app = Application::with_config(renderer, window, app_sender, font_collection, config)?;
    let app = App::new(app);
    // let _update = application_update_thread(app.weak(), app_receiver);

    // let _core = core::main_loop(core, core_duplex);
    // app.start();
    event_handler.run(app, event_loop, app_receiver);

    // TODO: handle the join.
    // core.join().unwrap();
    Ok(())
}

fn application_update_thread(app: WeakApp, receiver: Receiver<Update>) -> std::thread::JoinHandle<()> {
    unimplemented!();
    // kea::utils::spawn_thread("app update", move || {
    //     loop {
    //         let update = match receiver.recv()  {
    //             Ok(update) => update,
    //             _ => panic!("Channel disconnected"),
    //         };

    //         app.upgrade().inner().apply_update(update);
    //     }
    // })
}

pub fn setup_logger(config: &Config) -> Result<(), fern::InitError> {
    extern crate chrono;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file_path())?)
        .apply()?;
    Ok(())
}
