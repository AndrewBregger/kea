use std::collections::HashMap;

use super::platform::shader::{RectShader, Shader, TextShader};
use super::{Color, Rect, RenderError, vec4, Vector4F, platform, TextLine, Glyph};
use super::style::{self, Style, StyleMap};
use crate::glutin::dpi::{LogicalPosition, LogicalSize};
use crate::font::{self, Font, FontDesc, GlyphId, FontMetrics, FontCollection};

// use crate::euclid::vec2;
use crate::pathfinder_geometry::{vector::vec2f};
use platform::atlas::{Atlas, FontAtlas};
use pathfinder_geometry::transform3d::Transform4F;

use log::{debug, info};



macro_rules! gl_check {
    ($f:expr) => {{
        $f;
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
    }};
}

/// mode for how the renderer is to render data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Text,
    Rect,
    None,
}

/// The result of adding a new element to render buffers.
#[derive(Debug, Clone)]
pub enum SubmitResult {
    /// when the data buffer for the current mode is full then a
    /// flush most be executed.
    BufferFull,
    /// When the render mode changes then currently active buffer
    /// flushed to allow for the mode to change.
    ChangeMode,
    /// Element was added without error.
    Ok
}

const RECT_QUAD: usize = 4096;
const TEXT_QUAD: usize = 65535;
const RECT_VERTEX_SIZE: usize = 13;
const TEXT_VERTEX_SIZE: usize = 12;

pub trait Vertex {}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RectVertex {
    pub vertex: Vector4F,
    pub bg_color: Color,
}

impl RectVertex {
    fn create(vertex: Vector4F, bg_color: Color) -> Self {
        Self {
            vertex,
            bg_color,
        }
    }
}

impl Default for RectVertex {
    fn default() -> Self {
        Self {
            vertex: vec4(0.0, 0.0, 0.0, 0.0),
            bg_color: Color::black(),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TextVertex {
    pub vertex: Vector4F,
    pub fg_color: Color,
    pub tex_info: Vector4F,
    pub texture_id: f32,
}

impl TextVertex {
    fn create(vertex: Vector4F, fg_color: Color, tex_info: Vector4F, texture_id: f32) -> Self {
        Self {
            vertex,
            fg_color,
            tex_info,
            texture_id
        }
    }
}

impl Default for TextVertex {
    fn default() -> Self {
        Self {
            vertex: vec4(0.0, 0.0, 0.0, 0.0),
            fg_color: Color::black(),
            tex_info: vec4(0.0, 0.0, 0.0, 0.0),
            texture_id: 0.0,
        }
    }
}

pub struct Batch<T> {
    pub data: Vec<T>,
}

impl<T: Clone> Batch<T> {
    pub fn new(size: usize) -> Self {
        // asserts that vertices will fit completely into a batch.
        Self {
            data: Vec::with_capacity(size),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }


    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    // returns true if it successed
    // return false if it failed
    pub fn has_room(&self) -> bool {
        self.len() + 1 < self.capacity()
    }

    // adds a vertex data to the current buffer.
    pub fn push(&mut self, data: &T) {
        self.data.push(data.clone());
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

fn generate_indices() -> Vec<u32> {
    vec![0, 1, 2, 0, 2, 3]
}

/// auxiliary render information.
/// From a design standpoint, I wasn't sure exactly where
/// the information stored in this struct should actually be stored
/// to I made this.
pub struct RenderContext {
    style_map: StyleMap,
    font_collection: FontCollection,
}

impl RenderContext {

    pub fn new(collection: FontCollection) -> RenderContext {
        let mut context = Self {
            style_map: StyleMap::new(),
            font_collection: collection,
        };

        // temporary until real color themes are implemented.
        let normal = Style::new(0, Color::black(), Color::white(), false, false);
        let italic = Style::new(1, Color::black(), Color::white(), true, false);
        let bold   = Style::new(2, Color::black(), Color::white(), false, true);

        context.register_style(normal);
        context.register_style(italic);
        context.register_style(bold);

        context
    }

    #[inline]
    pub fn register_style(&mut self, style: Style) {
        self.style_map.register_style(style);
    }

    #[inline]
    pub fn fonts(&self) -> &FontCollection {
        &self.font_collection
    }
}


/// Maintains information needed to render
pub struct Renderer {
    textures: Vec<u32>,
    mode: RenderMode,
    rect_vao: u32,
    rect_vbo: u32,
    text_vao: u32,
    text_vbo: u32,
    ibo: u32,

    rect_batch: Batch<RectVertex>,
    text_batch: Batch<TextVertex>,

    rect_shader: RectShader,
    text_shader: TextShader,

    atlas: FontAtlas,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            textures: Vec::with_capacity(platform::MAX_TEXTURES),
            mode: RenderMode::None,
            rect_vao: 0,
            rect_vbo: 0,
            text_vao: 0,
            text_vbo: 0,

            ibo: 0,
            rect_batch: Batch::new(RECT_QUAD),
            text_batch: Batch::new(TEXT_QUAD),
            rect_shader: RectShader::create(),
            text_shader: TextShader::create(),
            atlas: FontAtlas::new(0.0),
        }
    }


    pub fn init(&mut self) -> Result<(), RenderError> {

        self.rect_shader.init()?;
        self.text_shader.init()?;

        self.init_ibo()?;
        self.init_rect()?;
        self.init_text()?;

        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        }
        Ok(())
    }


    fn init_ibo(&mut self) -> Result<(), RenderError> {
        let indices = generate_indices();

        unsafe {
            gl_check!(gl::GenBuffers(1, &mut self.ibo));

            if self.ibo == 0 {
                return Err(RenderError::IboFailed);
            }

            // create the index buffer
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo));
            gl_check!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (6 * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW
            ));

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        Ok(())
    }

    unsafe fn set_attrib_pointers(layout: &[i32], size: i32) {
        let mut stride = 0;
        for (i, sz) in layout.iter().enumerate() {
            gl_check!(gl::EnableVertexAttribArray(i as u32));
            gl_check!(gl::VertexAttribPointer(
                    i as u32,
                    *sz,
                    gl::FLOAT,
                    gl::FALSE,
                    size,
                    (stride * std::mem::size_of::<f32>()) as *const _
                ));
            gl_check!(gl::VertexAttribDivisor(i as u32, 1));
            stride += *sz as usize;
        }
    }

    fn init_rect(&mut self) -> Result<(), RenderError> {
        let layout_sizes = [4, 4];

        self.rect_shader.bind();
        unsafe {
            gl_check!(gl::GenVertexArrays(1, &mut self.rect_vao));
            gl_check!(gl::GenBuffers(1, &mut self.rect_vbo));

            if self.rect_vao == 0 || self.rect_vbo == 0 {
                self.rect_shader.unbind();
                return Err(RenderError::RectInitFailed);
            }

            gl_check!(gl::BindVertexArray(self.rect_vao));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, self.rect_vbo));
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo));

            gl_check!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (RECT_QUAD * std::mem::size_of::<f32>()) as isize,
                std::ptr::null() as *const _,
                gl::STREAM_DRAW
            ));

            Self::set_attrib_pointers(&layout_sizes, std::mem::size_of::<RectVertex>() as i32);

            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

            gl::BindVertexArray(0);
        }
        self.rect_shader.unbind();

        Ok(())
    }

    fn init_text(&mut self) -> Result<(), RenderError> {
        let layout_sizes = [4, 4, 4, 1];

        self.text_shader.bind();
        unsafe {
            gl_check!(gl::GenVertexArrays(1, &mut self.text_vao));
            gl_check!(gl::GenBuffers(1, &mut self.text_vbo));

            if self.text_vao == 0 || self.text_vbo == 0{
                self.text_shader.unbind();
                return Err(RenderError::TextInitFailed);
            }

            gl_check!(gl::BindVertexArray(self.text_vao));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, self.text_vbo));
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo));

            gl_check!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (TEXT_QUAD * std::mem::size_of::<f32>()) as isize,
                std::ptr::null() as *const _,
                gl::STREAM_DRAW
            ));

            Self::set_attrib_pointers(&layout_sizes, std::mem::size_of::<TextVertex>() as i32);

            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

            gl::BindVertexArray(0);
        }
        self.text_shader.unbind();

        Ok(())
    }

    pub fn set_atlas(&mut self, atlas: FontAtlas) {
        self.atlas = atlas;
        info!("Setting Renderers Font Atlas");

        self.text_shader.bind();

        for (idx, atlas) in self.atlas.atlas().iter().enumerate() {
            let handle = atlas.handle;
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + idx as u32);
                gl::BindTexture(gl::TEXTURE_2D, handle);
            }
        }

        self.text_shader.unbind();
    }

    pub fn set_perspective(&self, perf: &Transform4F) {
        self.rect_shader.bounded(|shader| shader.set_perspective(perf));
        self.text_shader.bounded(|shader| shader.set_perspective(perf));
    }


    /// renders a Rect
    pub fn render_rect(&mut self, _context: &RenderContext, rect: &Rect) {
        let vertex = RectVertex::create(vec4(rect.pos.x(), rect.pos.y(), rect.width, rect.height), rect.bg_color);
        self.submit_rect(&vertex);
    }

    /// renders a string at a specified location. This will layout the string according to new lines.
    /// Each frame the string will be reprocessed.
    //  this should be use for string that change regularly (such as status bar)
    //  uses the atlas for layout font information.
    pub fn render_str(&mut self, context: &RenderContext, s: &str, mut x: f32, y: f32, fg_color: Color, bg_color: Color, size: f32) {
        let font = context.font_collection.default_font();
    //pub fn render_str(&mut self, s: &str, mut x: f32, y: f32, fg_color: Color, bg_color: Color, desc: FontDesc, size: f32) {

        // let metrics = self.font_metrics.get(&desc).expect("requesting metrics for unknown font").scale_with(size, self.atlas.dpi_factor());
        let metrics = font.metrics().scale_with(size, self.atlas.dpi_factor());

        // this will make the vector longer then it needs to be
        let mut glyphs = Vec::with_capacity(s.len());
        let mut lines: u32 = 1;
        let mut width: f32 = 0.0;
        let start_x = x;
        let start_y = y;

        for (idx, c) in s.chars().enumerate() {
            let glyph_id = GlyphId::new(c, size, font.desc().clone());
            if c == '\n' {
                lines += 1;
                continue;
            }

            if let Some(info) = self.atlas.get_info(&glyph_id) {
                let pos = vec2f(x, y) + info.origin;
                let vertex = vec4(pos.x(), pos.y(), info.size.x(), info.size.y());
                let tex_info = vec4(info.uv.x(), info.uv.y(), info.uv_delta.x(), info.uv_delta.y());
                let tex_id = info.tex as f32;
                let vertex = TextVertex::create(vertex, fg_color, tex_info, tex_id);
                glyphs.push(vertex);

                x += info.advance.x();
                width += info.advance.x();
            }
        }
        //
        // let bg_height = lines as f32 * metrics.line_height();
        // let rect= Rect::with_position(vec2f(start_x, start_y + metrics.descent), width, bg_height).with_color(bg_color);
        // self.render_rect(&rect);
        //
        // let rect= Rect::with_position(vec2f(hightlight_start, start_y + metrics.descent), highlight_width, metrics.line_height()).with_color(Color::rgba(1.0, 0.8, 0.0, 0.6));
        // self.render_rect(&rect);

        glyphs.iter().for_each(|v| self.submit_character(v));
    }

    pub fn render_line(&mut self, context: &RenderContext, line: &TextLine, x: f32, y: f32, size: f32) {
        let glyphs = line.glyphs.as_slice();
        for style in line.styles.as_slice() {
            let span = style.span();
            let id = style.style();

            if let Some(style) = context.style_map.style(&id) {
                let font = context.font_collection.font_at(style.font_idx()).unwrap();

                let glyph_span = glyphs.get(span.start..span.end).unwrap();
                for glyph in glyph_span {
                    let glyph_id = GlyphId::new(glyph.ch, size, font.desc.clone());
                    if let Some(info) = self.atlas.get_info(&glyph_id) {
                        let vertex = TextVertex::create(
                            vec4(x + glyph.x + info.origin.x(), y + info.origin.y(), info.size.x(), info.size.y()),
                            style.text_color().clone(),
                            info.tex_info(),
                            info.tex as f32);

                        self.submit_character(&vertex);
                    }
                }
            }
            else { println!("Failed to find style: {:?}", id); }
        }
    }

    pub fn render_cursors(&mut self, context: &RenderContext, line: &TextLine, cursors: &[usize], y: f32, size: f32) {
        let metrics = context.font_collection.default_font().metrics().scale_with(size, context.font_collection.dpi_factor());
        for column in cursors {
            let x = line.glyphs[*column].x;
            self.render_cursor(context, x, y + metrics.descent, metrics.line_height());
        }
    }

    pub fn render_cursor(&mut self, context: &RenderContext, x: f32, y: f32, size: f32) {
        static CURSOR_WIDTH: f32 = 2.5;
        let rect = Rect::with_position(vec2f(x, y), CURSOR_WIDTH, size);
        self.render_rect(context, &rect);
    }

    pub fn submit_rect(&mut self, vertex: &RectVertex) {
        if self.mode != RenderMode::Rect {
            self.flush();
            self.mode = RenderMode::Rect;
        }
        else if !self.rect_batch.has_room() {
            self.flush();
        }
        self.rect_batch.push(vertex);
    }

    pub fn submit_character(&mut self, vertex: &TextVertex) {
        if self.mode != RenderMode::Text {
            self.flush();
            self.mode = RenderMode::Text;
        }
        else if !self.text_batch.has_room() {
            self.flush();
        }
        self.text_batch.push(vertex);
    }

    pub fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn flush_rect(&self) {
        let vertices = self.rect_batch.data.as_slice();
        let len = self.rect_batch.len();

        if len == 0 {
            return;
        }
        self.rect_shader.bind();
        unsafe {
            gl_check!(gl::BindVertexArray(self.rect_vao));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, self.rect_vbo));
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo));

            gl_check!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (len * std::mem::size_of::<RectVertex>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW
            ));


            gl_check!(gl::Enable(gl::BLEND));
            gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ZERO, gl::ONE);

            gl_check!(gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                std::ptr::null() as *const _,
                len as i32
            ));

            gl_check!(gl::Disable(gl::BLEND));

            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_check!(gl::BindVertexArray(0));
        }
        self.rect_shader.unbind();
    }

    fn flush_text(&self) {
        let vertices = self.text_batch.data.as_slice();
        let len = self.text_batch.len();
        if len == 0 {
            return;
        }

        self.text_shader.bind();
        unsafe {
            gl_check!(gl::BindVertexArray(self.text_vao));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, self.text_vbo));
            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo));

            gl_check!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (len * std::mem::size_of::<TextVertex>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW
            ));

            gl_check!(gl::Enable(gl::BLEND));
            gl::BlendFuncSeparate(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR, gl::ZERO, gl::ONE);

            gl_check!(gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                std::ptr::null() as *const _,
                len as i32
            ));

            gl_check!(gl::Disable(gl::BLEND));

            gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_check!(gl::BindVertexArray(0));
        }
        self.text_shader.unbind();
    }


    pub fn flush(&mut self) {
        use RenderMode::*;
        match self.mode {
            Rect => {
                self.flush_rect();
                self.rect_batch.clear();
            }
            Text => {
                self.flush_text();
                self.text_batch.clear();
            }
            None => {
            }
        }
    }
}
