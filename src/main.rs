#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// @TODO: remove
// #![feature(backtrace)]

extern crate pathfinder_geometry;
extern crate thiserror;
// extern crate euclid;
extern crate gl;
extern crate glutin;

mod app;
mod core;
mod font;
mod renderer;
mod ui;

use log::{debug, error, info, trace, warn};
use renderer::{window::LogicalSize, Window};

fn main() {
    let config = app::Config::load_config();
    app::setup_logger(&config).unwrap();
    info!("Loaded Config: {:#?}", config);
    app::run(config).unwrap();
}
