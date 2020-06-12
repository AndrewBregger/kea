// extern crate euclid;
extern crate font_kit;
use crate::pathfinder_geometry;
// extern crate freetype as ft;
// use ft::{Library};
pub use font_kit::canvas::{Canvas, Format, RasterizationOptions};
pub use font_kit::family_name::FamilyName;
pub use font_kit::font;
pub use font_kit::properties::{Properties, Weight, Style, Stretch};
pub use font_kit::hinting::HintingOptions;

pub use font_kit::error;
use std::{hash::Hasher,
          io::{Read, BufReader},
          sync::Arc,
          collections::HashMap};
use pathfinder_geometry::{vector::{Vector2F, Vector2I, vec2i, vec2f}, transform2d::Transform2F};
use std::str;

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("failed to load font: {:?} | {:?}", font.name, err)]
    FontLoadError {
        font: FontDesc,
        err: error::FontLoadingError,
    },

    #[error("invalid font size metrics: {:?}", font.name)]
    InvalidFontMetrics {
        font: FontDesc
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
    GlyphError {
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

    pub fn line_height(&self) -> f32 {
        self.ascent - self.descent + self.line_gap
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
    pub fn new(scale: f32, x_ppem: f32, y_ppem: f32, ppem: f32, ascent: f32, descent: f32, line_gap: f32) -> Self {
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
    // pub lib: Library,
    fonts: Vec<Font>,
    device_pixel_ratio: f32,
}

#[derive(Clone)]
pub struct Font {
    /// Loaders representation of a font.
    pub(crate) source: font::Font,
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
	pub const DEFAULT_FONT: usize = 0;
	pub const DEFAULT_ITALIC_FONT: usize = 1;
	pub const DEFAULT_BOLD_FONT: usize = 2;

    pub fn new(device_pixel_ratio: f32) -> Result<Self, FontError> {
        Ok(Self {
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

    pub fn add_font_by_name(&mut self, name: &str) -> Result<(), FontError> {
        // add normal,

		let normal = FontDesc::new(name, Properties::new());
		self.add_font(normal)?;

        // add italic,
        let italic_prop = Properties {
            style: Style::Italic,
            weight: Weight::NORMAL,
            stretch: Stretch::NORMAL,
        };
		let italic = FontDesc::new(name, italic_prop);
		self.add_font(italic)?;

        // add bold
        let bold_prop = Properties {
            style: Style::Normal,
            weight: Weight::BOLD,
            stretch: Stretch::NORMAL,
        };
		let bold = FontDesc::new(name, bold_prop);
		self.add_font(bold)?;

		Ok(())
    }

    pub fn add_font(&mut self, desc: FontDesc) -> Result<(), FontError> {
        let font = Font::new(desc, self.device_pixel_ratio)?;
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

    pub fn default_font(&self) -> &Font {
        unsafe { self.fonts.get_unchecked(Self::DEFAULT_FONT) }
    }

    pub fn default_italicfont(&self) -> &Font {
        unsafe { self.fonts.get_unchecked(Self::DEFAULT_ITALIC_FONT) }
    }

    pub fn default_bold_font(&self) -> &Font {
        unsafe { self.fonts.get_unchecked(Self::DEFAULT_BOLD_FONT) }
    }

    pub fn num_fonts(&self) -> usize {
        self.fonts.len()
    }

    pub fn dpi_factor(&self) -> f32 {
        self.device_pixel_ratio
    }

    pub fn add_default(&mut self) {
        let family_name = if cfg!(target_os = "win32") {
          	"Courier New"
        }
        else if cfg!(target_os = "macos") {
            "Menlo"
        }
        else {
          	"Droid Sans Mono"
        };

        self.add_font_by_name(family_name).expect("failed to find default font");
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
    pub origin: Vector2F,
    pub advance: Vector2F,
    pub bitmap: Vec<u8>,
}

impl Font {
    fn new(desc: FontDesc, device_pixel_ratio: f32) -> Result<Self, FontError> {
        let source = font_kit::source::SystemSource::new()
            .select_best_match(&[desc.name.clone()], &desc.properties)
            .map_err(|e| FontError::SelectionError { err: e })?
            .load().map_err(|e| FontError::FontLoadError { font: desc.clone(), err: e})?;

        Ok(Self {
            desc,
            source,
            device_pixel_ratio
        })

    }

    pub fn num_glyphs(&self) -> u32 {
        self.source.glyph_count()
    }

    pub fn desc(&self) -> &FontDesc {
        &self.desc
    }

    pub fn metrics(&self) -> FontMetrics {

        // let metrics = self.source.size_metrics().ok_or(FontError::InvalidFontMetrics { font: self.desc.clone() })?;
        let metrics = self.source.metrics();

        let height  = metrics.x_height as f32;
        let ascent  = metrics.ascent as f32;
        let descent = metrics.descent as f32;
        let ppem    = metrics.units_per_em as f32;


        FontMetrics {
            x_ppem: 0.0,
            y_ppem: 0.0,
            ppem,
            ascent,
            descent,
            line_gap: metrics.line_gap,
        }
    }

    pub fn get_glyph_index(&self, codepoint: char) -> Option<u32> {
        self.source.glyph_for_char(codepoint)
    }

    pub fn supports_codepoint(&self, codepoint: char) -> bool {
        self.get_glyph_index(codepoint).is_some()
    }

    pub fn rasterize_glyph(&self, codepoint: char, height: f32) -> Result<RasterizedGlyph, FontError> {
		//println!("Char: {}", codepoint);
        let glyph = GlyphId::new(codepoint, height, self.desc.clone());
        let height = glyph.scale_size(self.device_pixel_ratio);
        let metrics = self.source.metrics();
        let scale = Self::scale_size(height, self.device_pixel_ratio) / metrics.units_per_em as f32;

        if let Some(glyph_id) = self.get_glyph_index(codepoint) {
            let bounding_box = self.source.raster_bounds(glyph_id, height, Transform2F::default(), HintingOptions::None, RasterizationOptions::SubpixelAa)
            	.unwrap();
			let mut canvas = Canvas::new(bounding_box.size(), Format::Rgb24);

            self.source.rasterize_glyph(
                			&mut canvas,
                			glyph_id,
                			height,
                			Transform2F::from_translation(-bounding_box.origin().to_f32()),
                            // Transform2F::default(),
                            HintingOptions::None,
                			RasterizationOptions::SubpixelAa)
                		.map_err(|err| FontError::GlyphError { ch: codepoint, err })?;
			let advance = self.source.advance(glyph_id).map_err(|err| FontError::GlyphError { ch: codepoint, err })?; //.to_i32();
            let advance = vec2f(advance.x() * scale, advance.y() * scale);

            let mut temp_buffer = Vec::new();

            for i in 0..canvas.size.y() {
                let start = i as usize * canvas.stride;
                let end = (i as usize + 1) * canvas.stride;
                let row = &canvas.pixels[start..end];
                temp_buffer.push(row);
            }
            let origin = if cfg!(target_os = "linux") {
				bounding_box.origin().to_f32()
            }
            else {
            	let origin = self.source.origin(glyph_id).unwrap();
            	vec2f(origin.x() * scale, origin.y() * scale)
            };

            let origin = vec2f(origin.x(), origin.y().abs() - bounding_box.height() as f32);

  //          let origin = vec2f((origin.x() >> 6) as f32, (origin.y() >> 6) as f32);
            // let origin = self.source.og
            println!("Char: {} Bounding Box: {:#?} Origin: {:#?} Size: {:#?} Other Origin: {:#?}", codepoint, bounding_box, bounding_box.origin(), bounding_box.size(), origin);

            Ok(RasterizedGlyph {
                glyph,
                stride: canvas.stride,
                width: canvas.size.x(),
                height: canvas.size.y(),
                origin,
                advance,
                bitmap: temp_buffer.into_iter().rev().flat_map(|e| e.to_vec()).collect(),
            })
        }
        else {
			Err(FontError::UnsupportedGlyph { ch: codepoint })
        }
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
