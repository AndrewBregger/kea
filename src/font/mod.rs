extern crate euclid;
extern crate font_kit;
extern crate freetype as ft;

use ft::{Library};
use euclid::{Point2D, Size2D};
pub use font_kit::{family_name::FamilyName, font, properties::Properties};
use std::{hash::Hasher,
          io::{Read, BufReader},
          sync::Arc,
          collections::HashMap};
use euclid::default::Scale;
use std::str;
use euclid::{default::Vector2D, vec2};

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("failed to load font: {:?} | {:?}", font.name, err)]
    FontLoadError {
        font: FontDesc,
        err: ft::error::Error,
    },

    #[error("invalid font size metrics: {:?}", font.name)]
    InvalidFontMetrics {
        font: FontDesc
    },

    #[error("failed to failed to initialize library: {:?}", err)]
    LibraryInit {
        err: ft::error::Error
    },

    #[error("failed to find font: {:?}", font.name)]
    InvalidFont {
        font: FontDesc
    },

    #[error("attempting to load an invalid glyph: '{}'", ch)]
    InvalidGlyph {
        ch: char
    },

    #[error("loaded fonts do not support glyph: '{}'", ch)]
    UnsupportedGlyph {
        ch: char
    },

    #[error("failed to load glyph: {} | {}", ch, err)]
    GLyphError {
        ch: char,
        err: font_kit::error::GlyphLoadingError,
    },

    #[error("selection error: {}", err)]
    SelectionError {
        err: font_kit::error::SelectionError,
    },
}

/// used to describe what font is to be rendered and in what style.
#[derive(Debug, Clone)]
pub struct FontDesc {
    name: FamilyName,
    properties: Properties,
}

impl FontDesc {
    pub fn new(name: &str, properties: Properties) -> Self {
        Self {
            name: FamilyName::Title(name.to_string()),
            properties,
        }
    }
}

impl std::cmp::PartialEq for FontDesc {
    fn eq(&self, other: &Self) -> bool {
        match (&self.name, &other.name) {
            (FamilyName::Title(title), FamilyName::Title(title2)) => title.eq(title2),
            (FamilyName::Serif, FamilyName::Serif)
            | (FamilyName::SansSerif, FamilyName::SansSerif)
            | (FamilyName::Monospace, FamilyName::Monospace)
            | (FamilyName::Cursive, FamilyName::Cursive)
            | (FamilyName::Fantasy, FamilyName::Fantasy) => true,
            _ => false,
        }
    }
}

impl std::cmp::Eq for FontDesc {}

impl std::hash::Hash for FontDesc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let FamilyName::Title(ref title) = &self.name {
            title.hash(state);
        } else {
            panic!("Font Family name not handled in hasher: {:?}", self.name);
        }

        state.write_i8(match self.properties.style {
            font_kit::properties::Style::Normal => 1,
            font_kit::properties::Style::Italic => 2,
            font_kit::properties::Style::Oblique => 3,
        });
    }
}

#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub x_ppem: f32,
    pub y_ppem: f32,
    pub ppem: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
}

impl FontMetrics {
    pub fn new(x_ppem: f32, y_ppem: f32, ppem: f32, ascent: f32, descent: f32, line_gap: f32) -> Self {
        Self {
            x_ppem,
            y_ppem,
            ppem,
            ascent,
            descent,
            line_gap
        }
    }

    pub fn scale(&self, scale: f32) -> ScaledFontMetrics {
        ScaledFontMetrics::new(scale, self.x_ppem, self.ppem, self.y_ppem, self.ascent * scale, self.descent * scale, self.line_gap * scale)
    }

    pub fn scale_with(&self, height: f32, dpi: f32) -> ScaledFontMetrics {
        let scale = Font::scale_size(height, dpi) / self.ppem;
        self.scale(scale)
    }
}

#[derive(Debug, Clone)]
pub struct ScaledFontMetrics {
    pub scale: f32,
    pub x_ppem: f32,
    pub y_ppem: f32,
    pub ppem: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
}

impl ScaledFontMetrics {
    pub fn new(x_ppem: f32, y_ppem: f32, ppem: f32, scale: f32, ascent: f32, descent: f32, line_gap: f32) -> Self {
        Self {
            scale,
            x_ppem,
            y_ppem,
            ppem,
            ascent,
            descent,
            line_gap
        }
    }

    pub fn unscale(&self) -> FontMetrics {
        FontMetrics::new(self.x_ppem, self.y_ppem, self.ppem, self.ascent / self.scale, self.descent / self.scale, self.line_gap / self.scale)
    }


    pub fn line_height(&self) -> f32 {
        self.ascent - self.descent + self.line_gap
    }
}

/// collection of fonts used by the application
pub struct FontCollection {
    pub lib: Library,
    fonts: Vec<Font>,
    device_pixel_ratio: f32,
}

#[derive(Clone)]
pub struct Font {
    /// Loaders representation of a font.
    pub(crate) source: ft::Face,
    /// description of what font is to be loaded.
    pub(crate) desc: FontDesc,
    pub(crate) device_pixel_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct GlyphId {
    pub glyph: char,
    pub size: f32,
    pub font: FontDesc,
}

impl FontCollection {
    pub fn new(device_pixel_ratio: f32) -> Result<Self, FontError> {
        Ok(Self {
            lib: Library::init().map_err(|e| FontError::LibraryInit {err: e })?,
            fonts: Vec::new(),
            device_pixel_ratio,
        })
    }

    pub fn from_font(desc: FontDesc, device_pixel_ration: f32) -> Result<Self, FontError> {
        let mut collection = Self::new(device_pixel_ration)?;
        collection.add_font(desc)?;
        Ok(collection)
    }

    pub fn find_font(&self, desc: FontDesc) -> Option<&Font> {
        self.fonts.iter().find(|&font| font.desc == desc)
    }

    pub fn add_font(&mut self, desc: FontDesc) -> Result<(), FontError> {
        let font = Font::new(&self.lib, desc, self.device_pixel_ratio)?;
        println!("Num GLyphs: {}", font.num_glyphs());
        self.fonts.push(font);
        Ok(())
    }

    pub fn find_codepoint_font(&self, codepoint: char) -> Option<usize> {
        for (i, font) in self.fonts.iter().enumerate() {
            if font.supports_codepoint(codepoint) {
                return Some(i)
            }
        }
        None
    }

    pub fn font_at(&self, index: usize) -> Option<&Font> {
        if index < self.fonts.len() {
            Some(&self.fonts[index])
        }
        else {
            None
        }
    }

    pub fn num_fonts(&self) -> usize {
        self.fonts.len()
    }

    pub fn dpi_factor(&self) -> f32 {
        self.device_pixel_ratio
    }
}

impl GlyphId {
    pub fn new(glyph: char, size: f32, font: FontDesc) -> Self {
        Self { glyph, size, font }
    }
    pub fn scale_size(&self, dpi: f32) -> f32 {
        Font::scale_size(self.size, dpi)
    }
}

impl std::cmp::PartialEq for GlyphId {
    fn eq(&self, other: &Self) -> bool {
        self.glyph.eq(&other.glyph)
            && self.size as i32 == other.size as i32
            && self.font.eq(&other.font)
    }
}

impl std::cmp::Eq for GlyphId {}

impl std::hash::Hash for GlyphId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.glyph.hash(state);
        (self.size as i32).hash(state);
        self.font.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    pub glyph: GlyphId,
    pub stride: usize,
    pub width: i32,
    pub height: i32,
    pub origin: Vector2D<f32>,
    pub advance: Vector2D<f32>,
    pub bitmap: Vec<u8>,
}

impl Font {
    fn new(lib: &Library, desc: FontDesc, device_pixel_ratio: f32) -> Result<Self, FontError> {
        let font = font_kit::source::SystemSource::new()
            .select_best_match(&[desc.name.clone()], &desc.properties)
            .map_err(|e| FontError::SelectionError { err: e })?;
        let (buffer, index) = match font {
            font_kit::handle::Handle::Path {
                path,
                font_index,
            } => {
                println!("Path: {}", path.display());
                let buffer = std::fs::File::open(path)
                             .and_then(|f| Ok(BufReader::new(f)))
                             .and_then(|b| Ok(b.bytes().map(|e| e.unwrap()).collect()))
                             .map_err(|e| FontError::InvalidFont { font: desc.clone()})?;
                // let file = std::fs::File::open(path).map_err(|e| )?;
                // let buf_reader = BufReader::new(file);
                // let buffer: Vec<u8> = buf_reader.bytes().map(|e| e.unwrap()).collect();
                (buffer, font_index)
            }
            font_kit::handle::Handle::Memory {
                bytes,
                font_index
            } => {
                let buffer = Arc::try_unwrap(bytes).expect("Expected only one strong reference");
                println!("Memory FontIndex: {}", font_index);
                (buffer, font_index)
            }
        };

        let face = lib.new_memory_face(buffer, index as isize).map_err(|e| FontError::FontLoadError { font: desc.clone(), err: e})?;

        Ok(Self {
            source: face,
            desc,
            device_pixel_ratio
        })

    }

    pub fn num_glyphs(&self) -> u32 {
        // unsafe {
            self.source.raw().num_glyphs as u32
        // }
    }

    pub fn desc(&self) -> &FontDesc {
        &self.desc
    }

    pub fn font_metrics(&self) -> Result<FontMetrics, FontError> {

        // let metrics = self.source.size_metrics().ok_or(FontError::InvalidFontMetrics { font: self.desc.clone() })?;

        let height  = self.source.height() as f32;
        let ascent  = self.source.ascender() as f32;
        let descent = self.source.descender() as f32;

        let ppem = self.source.em_size() as f32;


        Ok(FontMetrics {
            x_ppem: 0.0,
            y_ppem: 0.0,
            ppem,
            ascent,
            descent,
            line_gap: height + descent - ascent,
        })
    }

    pub fn get_glyph_index(&self, codepoint: char) -> u32 {
        self.source.get_char_index(codepoint as usize)
    }

    pub fn supports_codepoint(&self, codepoint: char) -> bool {
        self.source.get_char_index(codepoint as usize) != 0
    }

    pub fn rasterize_glyph(&self, codepoint: char, height: f32) -> Result<RasterizedGlyph, FontError> {
        let glyph = GlyphId::new(codepoint, height, self.desc.clone());
        let height = glyph.scale_size(self.device_pixel_ratio);

        let height = to_freetype_26_6(height);

        self.source.set_char_size(0, height, 0, 0).unwrap();
        self.source.load_char(glyph.glyph as usize, ft::face::LoadFlag::RENDER).unwrap();
        let rasterized_glyph = self.source.glyph();

        rasterized_glyph.render_glyph(ft::RenderMode::Normal).unwrap();
        let buffer = rasterized_glyph.bitmap();
        let metrics = rasterized_glyph.metrics();
        let x = rasterized_glyph.bitmap_left();
        let y = rasterized_glyph.bitmap_top() - buffer.rows();

        let origin = vec2(x as f32, y as f32);

        // let mut bitmap = Vec::new();
        let pitch = buffer.pitch() as usize;
        let buf = buffer.buffer();

        let mut rows = Vec::new();
        match buffer.pixel_mode().unwrap() {
            ft::bitmap::PixelMode::Gray => {
                for i in 0..buffer.rows() {
                    let start = (i as usize) * pitch;
                    let stop = start + pitch; // as usize;
                    rows.push(Vec::new());
                    for byte in &buf[start..stop] {
                        rows.last_mut().unwrap().extend_from_slice(&[*byte, *byte, *byte]);
                    }
                }
            }
            e => unimplemented!("Unhandled pixel mode: {:?}", e)
        }

        let stride = buffer.width() as usize * 3;
        rows.reverse();
        // let bitmap: Vec<u8> = buffer.buffer().to_owned();
        let bitmap: Vec<u8> = rows.iter().flatten().map(|val| *val).collect();

        Ok(RasterizedGlyph {
            glyph,
            stride,
            width: buffer.width(),
            height: buffer.rows(),
            origin,
            advance: vec2((metrics.horiAdvance >> 6) as f32, (metrics.vertAdvance >> 6) as f32),
            bitmap,
        })
    }

    #[inline]
    pub fn scale_size(height: f32, dpi: f32) -> f32 {
        height * dpi * 96.0 / 72.0
   }
}

#[inline]
pub fn to_freetype_26_6(f: f32) -> isize {
    ((1i32 << 6) as f32 * f) as isize
}

#[inline]
pub fn from_freetype_26_6(f: f32) -> isize {
    ((1i32 >> 6) as f32 * f) as isize
}
