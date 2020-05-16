extern crate ropey;
use ropey::{Rope, RopeSlice};

use crate::app::Config;
use std::sync::{Arc, Mutex};

pub enum KeaCore {
    Waiting,
    Running(Arc<Mutex<()>>)
}

impl KeaCore {
    pub fn new(_config: &Config) -> Self {
        KeaCore::Waiting
    }
}

// pub struct RenderContext(Weak<Mutex<Renderer>>);
// pub struct RenderCore(Arc<Mutex<Renderer>>);
// impl RenderCore {
//     pub fn new() ->RenderCore  {
//         RenderCore(Arc::new(Mutex::new(Renderer::new())))
//     }

//     pub fn inner(&self) -> MutexGuard<Renderer> {
//         self.0.lock().unwrap()
//     }

//     pub fn weak(&self) -> RenderContext {
//         RenderContext(Arc::downgrade(&self.0))
//     }
// }

// impl RenderContext {
//     pub fn upgrade(&self) ->  RenderCore {
//         match self.0.upgrade() {
//             Some(ctx) => RenderCore(ctx),
//             None => panic!("Renderer was not created"),
//         }
//     }
// }