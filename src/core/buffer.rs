use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::BufReader;

use ropey::Rope;

use super::CoreError;


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferId(usize);

fn map_error(e: std::io::Error, path: &PathBuf) -> CoreError {
    match e.kind() {
        std::io::ErrorKind::NotFound => CoreError::FileNotFound(path.clone()),
        std::io::ErrorKind::PermissionDenied => CoreError::FilePermissions(path.clone()),
        std::io::ErrorKind::Interrupted => panic!("Read interrupted"),
        _ => unimplemented!("Io errors not mapped: {}", e),
    }
}

pub struct Buffer {
    id: BufferId,
    path: Option<PathBuf>,
    content: Rope,
}

pub type BufferResult = ::std::result::Result<BufferId, CoreError>;

impl Buffer {
    pub fn from_path<S: AsRef<Path>>(path: S, id: usize) -> Result<Self, CoreError> {
        let id = BufferId(id);

        let content = File::open(path.as_ref())
                        .and_then(|f| Ok(BufReader::new(f)))
                        .and_then(|r| Rope::from_reader(r))
                        .map_err(|e| map_error(e, &path.as_ref().to_path_buf()))?;
        
        Ok(Self {
            id,
            path: Some(path.as_ref().to_path_buf()),
            content
        })
    }

    pub fn empty(id: usize) -> Result<Self, CoreError> {
        Ok(Self {
            id: BufferId(id),
            path: None,
            content: Rope::new()
        })
    }

    pub fn id(&self) -> BufferId {
        self.id
    }
}
