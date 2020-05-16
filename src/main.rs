#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// @TODO: remove
#![feature(backtrace)]

extern crate thiserror;
extern crate euclid;
extern crate glutin;
extern crate gl;

mod app;
mod core;
mod ui;
mod renderer;
mod font;

mod kea;

use log::{debug, error, info, trace, warn};
use renderer::{Window, window::LogicalSize};


fn main() {
    let config = app::Config::load_config();
    app::setup_logger(&config).unwrap();
    info!("Loaded Config: {:#?}", config);
    app::run(config).unwrap();
}
