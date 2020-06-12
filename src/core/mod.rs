extern crate ropey;

use ropey::{Rope, RopeSlice};

use std::sync::{Arc, Mutex, Weak, MutexGuard};
use std::path::PathBuf;

use crate::app::Config;

use log::{error};
use kea;

mod edit;
mod buffer;
mod view;

use edit::{Core};
pub use edit::{Update, Edit, BufferInfo};

pub use buffer::{Buffer, BufferId};
use view::{View};
pub use view::{ViewInfo, ViewId};

#[derive(thiserror::Error, Debug, Clone)]
pub enum CoreError  {
    #[error("file not found: '{0}'")]
    FileNotFound(PathBuf),
    #[error("do not have permission to open: '{0}'")]
    FilePermissions(PathBuf),
}


// pub enum KeaCore {
//     Waiting,
//     Running(Arc<Mutex<()>>)
// }

pub struct KeaCore(Arc<Mutex<Core>>);

impl KeaCore {
    pub fn new(_config: &Config) -> Self {
        KeaCore(Arc::new(Mutex::new(Core::new())))
    }

    pub fn inner(&self) -> MutexGuard<Core> {
        self.0.lock().unwrap()
    }

    pub fn weak(&self) -> WeakCore {
        WeakCore(Arc::downgrade(&self.0))
    }
}


pub struct WeakCore(Weak<Mutex<Core>>);

impl WeakCore {
    pub fn upgrade(&self) ->  KeaCore {
        match self.0.upgrade() {
            Some(core) => KeaCore(core),
            None => panic!("Kea Core was not created"),
        }
    }
}

// pub fn main_loop(core: KeaCore, duplex: kea::comm::Duplex<Update, Edit>) -> std::thread::JoinHandle<()> {
//     kea::utils::spawn_thread("core", move || {
//         loop  {
//             let edit_operation = match duplex.recv() {
//                 Ok(msg) => msg,
//                 Err(e) => {
//                     error!("Core Channel Disconnected: {}", e);
//                     panic!();
//                 }
//             };
// 
//             use Edit::*;
// 
//             match edit_operation {
//                 OpenFile(view_info) => {
//                     let result = core.inner().open_file(view_info);
//                     match result {
//                         Ok(view) => {
//                             let id = view.view;
//                             duplex.send(Update::View(view)).unwrap();
// 
//                             let update = core.inner().update_view(id);
//                             duplex.send(update).unwrap();
//                         }
//                         Err(err) => {
//                             duplex.send(err.into()).unwrap();
//                         }
//                     }
//                 },
//                 Close => { break; }
//             }
//         }
//     })
// }
