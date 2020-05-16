
mod config;
mod application;
mod event_handler;

pub use config::{Config};
use super::kea::{self, utils::log_file_path};
use super::renderer::{self, Renderer, window::{Window, LogicalSize}};
use application::{Application};
use event_handler::{EventHandler};
use std::sync::{Arc, Mutex};
use crate::font::{FontCollection, GlyphId, Font, FontMetrics};

use crate::renderer::platform::atlas::FontAtlas;

use glutin::event_loop::EventLoop;

use log::{error, info};

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
    // let (app_duplex, core_duplex) = kea::comm::duplex::<(), ()>();

    let dpi = window.dpi_factor() as f32;

    let mut font_collection = FontCollection::new(dpi).unwrap();
    let font_desc = config.font_desc();
    let font_size = config.font_size();

    font_collection.add_font(font_desc).unwrap();

    // the font needs to be loaded on this thread because it cannot be between across thread boundaries.
    let window = window.make_current().map_err(AppError::WindowError)?;
    window.init_gl();
    
    let font_atlas = FontAtlas::from_collection(&font_collection, |atlas, font| {
        for c in 32 as u8 .. 128 as u8 {
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
    for idx in 0..font_collection.num_fonts() {
        if let Some(font) = font_collection.font_at(idx) {
            renderer.add_font_metric(font.desc().clone(), font.font_metrics().unwrap_or(FontMetrics::new(0., 0., 0., 0., 0., 0.)))
        }
    }

    let (width, height) = window.get_size().into();
    renderer.update_perspective(width, height);

    let app = Application::with_config(renderer, window, config)?.as_arc();

    let elp = event_loop.create_proxy();
    let mut event_handler = EventHandler::new(elp);

    // app.start();
    event_handler.run(app, event_loop);

    // TODO: handle the join.
    // render.join().unwrap();
    Ok(())
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