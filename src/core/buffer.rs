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

struct Line {
    // line number in line space (starts at 1)
    line: usize,
	/// the indices
    indices: LineIndices
}

impl Line {
    fn new(line: usize, start: usize, end: usize) -> Self {
        Self {
			line,
			indices: LineIndices::new(start, end)
        }
    }

    fn start_index(&self) -> usize {
        self.indices.start
    }

	fn end_index(&self) -> usize {
    	self.indices.end
	}
}

struct LineIndices {
	start: usize,
	end: usize,
}

impl LineIndices {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end
        }
    }
}

struct ShallowCache {
	lines: Vec<Line>
}

impl ShallowCache {
    fn new() -> Self {
        Self {
			lines: Vec::new()
        }
    }

    fn push_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    fn insert_line(&mut self, idx: usize, line: Line) {
        self.lines.insert(idx, line);
    }

    fn replace_line(&mut self, idx: usize, line: Line) {
       	if idx < self.lines.len() {
       		self.lines[idx] = line;
       	}
    }

    fn get(&self, idx: usize) -> Option<&Line> {
        self.lines.get(idx)
    }

    fn clear(&mut self) {
        self.lines.clear()
    }

    fn len(&self) -> usize {
        self.lines.len()
    }

    fn slice<R: std::slice::SliceIndex<[Line]>>(&self, r: R) -> Option<&<R as std::slice::SliceIndex<[Line]>>::Output> {
        self.lines.get(r)
    }
}

pub struct Buffer {
    id: BufferId,
    path: Option<PathBuf>,
    content: Rope,
    shallow_cache: ShallowCache,
}

pub type BufferResult = ::std::result::Result<BufferId, CoreError>;

impl Buffer {
    pub fn from_path<S: AsRef<Path>>(path: S, id: usize) -> Result<Self, CoreError> {
        let id = BufferId(id);

        let content = File::open(path.as_ref())
                        .and_then(|f| Ok(BufReader::new(f)))
                        .and_then(|r| Rope::from_reader(r))
                        .map_err(|e| map_error(e, &path.as_ref().to_path_buf()))?;
        
       	let mut buffer = Self {
            id,
            path: Some(path.as_ref().to_path_buf()),
            content,
            shallow_cache: ShallowCache::new(),
        };

        buffer.invalidate_line_cache(0);

        Ok(buffer)
    }

    pub fn empty(id: usize) -> Result<Self, CoreError> {
        Ok(Self {
            id: BufferId(id),
            path: None,
            content: Rope::new(),
            shallow_cache: ShallowCache::new(),
        })
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn content(&self) -> &Rope {
        &self.content
    }

	/// invalids the shallow line cache from start_line to the end of the buffer.
    pub fn invalidate_line_cache(&mut self, start_line: usize) {
		println!("len: {}", self.shallow_cache.len());
        let mut last_line_byte = self.shallow_cache.get(start_line).map_or(0, |l| l.start_index());
        println!("{}|{}", start_line, last_line_byte);
 
        for (idx, line) in self.content.lines().skip(start_line).enumerate() {
            let idx = idx + start_line;
            println!("Checking: {:?}", line);
            let bytes = line.len_bytes();
            if bytes == 0 {
                continue;
            }
 
            let line = Line::new(idx + 1, last_line_byte, last_line_byte + bytes);
            if idx < self.shallow_cache.len() {
                println!("Replacing");
                self.shallow_cache.replace_line(idx, line);
            }
            else {
                println!("Pushing");
                self.shallow_cache.push_line(line);
            }
 
            last_line_byte += bytes;
        }
 
        for line in &self.shallow_cache.lines {
            let start = line.start_index();
            let end = line.end_index();
            let l = self.content.slice(start..end);
            println!("idx: {}, {} {} {}", line.line, start, end, l);
        }
    }

    pub fn request_lines(&self, start: usize, end: usize) -> Vec<String> {
        let mut res = Vec::new();

        if let Some(lines) = self.shallow_cache.slice(start..end) {
    		for line in lines {
				let val = String::from(self.content.slice(line.start_index()..line.end_index()));
				res.push(val);
    		}
        }

        res
    }
}
