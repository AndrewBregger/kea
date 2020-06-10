// Font Character Atlas.
// This is to assist rendering of text. All common characters are preloaded into a set of fixed sized textures
// to allow quick access and render.

use crate::font::{self, FontDesc, GlyphId, RasterizedGlyph, FontMetrics, FontCollection, Font};
use crate::gl::{self, types::*};
use std::collections::HashMap;
use crate::renderer::Vector4D;
// use euclid::{default::Vector2D, vec2};
use crate::pathfinder_geometry::vector::{vec2f, Vector2F, Vector2I};
type AtlasSize = (i32, i32);

const ATLAS_SIZE: AtlasSize = (1024, 1024);
const MAX_ATLASES: usize = 4;

macro_rules! gl_check_impl {
    ($f:expr) => {
        if cfg!(debug_assertions) {
            let err = gl::GetError();
            // println!("Error {:?}", err);
            if err != gl::NO_ERROR {
                let err_str = match err {
                    gl::INVALID_ENUM => "GL_INVALID_ENUM",
                    gl::INVALID_VALUE => "GL_INVALID_VALUE",
                    gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                    gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                    gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                    gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
                    gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                    _ => "unknown error",
                };

                panic!(
                    "{}:{} error {} {}",
                    file!(),
                    line!(),
                    std::stringify!($f),
                    err_str
                );
            }
        }
    };
}

macro_rules! gl_check {
    ($f:expr) => {{
        $f;
        gl_check_impl!($f)
    }};
    () => {{
        gl_check_impl!("no_function")
    }};
}

fn create_texture(size: &AtlasSize, wrap: GLenum, mapping: GLenum) -> u32 {
    let mut handle = 0;

    unsafe {
        gl::GenTextures(1, &mut handle);

        if handle == 0 {
            panic!("Failed to generate texture");
        }

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::BindTexture(gl::TEXTURE_2D, handle);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, mapping as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mapping as GLint);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            size.0,
            size.1,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );

        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    handle
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub tex: u32,
    pub uv: Vector2F,
    pub uv_delta: Vector2F,
    pub size: Vector2F,
    pub advance: Vector2F,
    pub origin: Vector2F,
}

/// Collection of Atlass that make up the font.
/// When an atlas becomes full, a new
pub struct FontAtlas {
    // a list of atlases
    atlases: Vec<Atlas>,
    // font: FontDesc,
    // metrics: FontMetrics,
    // meta data used for constructing the current atlas.

    // the x and y offset into the current texture.
    pub(crate) x_offset: i32,
    pub(crate) y_offset: i32,
    // the largest height of the current row of glyphs.
    pub(crate) max_height: i32,
    dpi_factor: f32,
}

impl FontAtlas {
    pub fn new(dpi_factor: f32) -> Self {
        Self {
            atlases: Vec::new(),
            x_offset: 0,
            y_offset: 0,
            max_height: 0,
            dpi_factor
        }
    }

    pub fn dpi_factor(&self) -> f32 {
        self.dpi_factor
    }

    pub fn from_collection<P>(collection: &FontCollection, loader: P) -> Result<Self, font::FontError>
        where P: FnMut(&mut Self, &Font) -> Result<(), font::FontError> {
        let mut atlas = Self::new(collection.dpi_factor());
        atlas.new_atlas(&ATLAS_SIZE);
        atlas.load_collection(collection, loader)?;
        Ok(atlas)
    }

    fn load_collection<P>(&mut self, collection: &FontCollection, mut loader: P) -> Result<(), font::FontError>
        where P: FnMut(&mut Self, &Font) -> Result<(), font::FontError> {
        for i in 0..collection.num_fonts() {
           match collection.font_at(i) {
               Some(font) => loader(self, font)?,
               _ => {}
           }
        }
       Ok(())
    }

    pub fn atlas(&self) -> &[Atlas] {
        self.atlases.as_slice()
    }

    pub fn add_glyph(&mut self, glyph: &RasterizedGlyph) {
        if self.atlases.is_empty() {
            self.new_atlas(&ATLAS_SIZE);
        }

        // check if room on line
        if !self.room_on_line(glyph) {
            // if not advance to next line
            self.next_line(glyph);
        }

        // check if the
        if !self.fit_vertical(glyph) {
            self.new_atlas(&ATLAS_SIZE)
        }

        // I know this is safe
        let atlas = self.atlases.last_mut().unwrap();

        atlas.bind();

        unsafe {
            gl_check!(gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                self.x_offset,
                self.y_offset,
                glyph.width,
                glyph.height,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                glyph.bitmap.as_ptr() as *const _
            ));
        }

        atlas.unbind();

        let uv = vec2f(
            self.x_offset as f32 / ATLAS_SIZE.0 as f32,
            self.y_offset as f32 / ATLAS_SIZE.1 as f32,
        );

        let uv_delta = vec2f(
            glyph.width as f32 / ATLAS_SIZE.0 as f32,
            (glyph.height + 1) as f32 / ATLAS_SIZE.1 as f32,
        );

        let advance = glyph.advance;
        let origin = glyph.origin;

        let atlas_glyph = GlyphInfo {
            tex: atlas.handle,
            uv,
            uv_delta,
            size: vec2f(glyph.width as f32, (glyph.height + 1) as f32),
            advance,
            origin,
        };


        atlas
            .glyph_infos
            .entry(glyph.glyph.clone())
            .or_insert(atlas_glyph);

        self.x_offset += glyph.width;

        if glyph.height > self.max_height {
            self.max_height = glyph.height;
        }
    }

    fn flip_y(glyph: &RasterizedGlyph) -> Vec<u8> {
        let mut buffer = Vec::with_capacity((glyph.width * glyph.height) as usize);
        let mut buffers = Vec::new();
        for y in 0..glyph.height {
            let (row_start, row_end) = (y as usize * glyph.width as usize, (y + 1) as usize * glyph.width as usize);
            let row = &glyph.bitmap[row_start..row_end];
            buffers.push(row);
        }
        buffers.reverse();
        for buf in buffers {
            buffer.extend_from_slice(buf);
        }
        buffer
        // glyph.bitmap.clone()
    }

    pub fn get_info(&self, glyph: &GlyphId) -> Option<&GlyphInfo> {
        for atlas in &self.atlases {
            match atlas.get_info(glyph) {
                Some(info) => return Some(info),
                None => continue,
            }
        }
        None
    }

    pub fn has_info(&self, glyph: &GlyphId) -> bool {
        for atlas in &self.atlases {
            if atlas.has_info(glyph) {
                return true;
            }
        }
        false
    }

    fn new_atlas(&mut self, size: &AtlasSize) {
        if self.atlases.len() == MAX_ATLASES {
            // maybe this could be changed to a Result.
            panic!("Unable to create anymore atlases for this font");
        }

        self.atlases.push(Atlas::new(size));
        self.x_offset = 0;
        self.y_offset = 0;
        self.max_height = 0;
    }

    fn room_on_line(&self, glyph: &RasterizedGlyph) -> bool {
        self.x_offset + glyph.width < ATLAS_SIZE.0
    }

    fn next_line(&mut self, glyph: &RasterizedGlyph) {
        // move to next line
        self.x_offset = 0;
        self.y_offset += self.max_height;
        // reset the height of the current line.
        self.max_height = 0;
    }

    fn fit_vertical(&self, glyph: &RasterizedGlyph) -> bool {
        self.y_offset + glyph.height < ATLAS_SIZE.1
    }
}

pub struct Atlas {
    // gl texture handle.
    pub(crate) handle: u32,
    // size of the created texture.
    size: AtlasSize,
    // glyph lookup for texture info.
    glyph_infos: HashMap<GlyphId, GlyphInfo>,
}

impl Atlas {
    // Fix, the creatino of of a texture could fail.
    pub(crate) fn new(size: &AtlasSize) -> Self {
        let handle = create_texture(size, gl::CLAMP_TO_BORDER, gl::LINEAR);

        Self {
            handle,
            size: *size,
            glyph_infos: HashMap::new(),
        }
    }

    pub fn get_info(&self, glyph: &GlyphId) -> Option<&GlyphInfo> {
        self.glyph_infos.get(glyph)
    }

    #[inline]
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.handle);
        }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn has_info(&self, glyph: &GlyphId) -> bool {
        self.glyph_infos.contains_key(glyph)
    }
}
