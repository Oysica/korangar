use std::io::Cursor;
use std::sync::Arc;

use cosmic_text::FontSystem;
use cosmic_text::fontdb::{ID, Source};
use hashbrown::HashMap;
use image::{ImageFormat, ImageReader, RgbaImage};
#[cfg(feature = "debug")]
use korangar_debug::logging::{Colorize, Timer, print_debug};
use korangar_loaders::FileLoader;

use crate::loaders::GameFileLoader;
use crate::loaders::font::GlyphCoordinate;
use crate::loaders::font::font_map_descriptor::parse_glyphs;

const FONT_FOLDER_PATH: &str = "data\\font";

/// Format of the first column in the atlas CSV produced by msdf-atlas-gen.
#[derive(Clone, Copy)]
pub enum CsvFormat {
    /// `-allglyphs` / `-glyphset` output: column 1 is the font's internal glyph
    /// index.
    GlyphId,
    /// `-charset` output: column 1 is the Unicode codepoint of each glyph.
    /// The loader remaps these to glyph ids using the font's cmap so renderers
    /// that work in glyph-id space (e.g. cosmic-text) can find them.
    Codepoint,
}

pub(crate) struct FontFile {
    pub(crate) ids: Vec<ID>,
    pub(crate) font_map: RgbaImage,
    pub(crate) glyphs: Arc<HashMap<u16, GlyphCoordinate>>,
}

impl FontFile {
    pub(crate) fn new(
        name: &str,
        csv_format: CsvFormat,
        game_file_loader: &GameFileLoader,
        font_system: &mut FontSystem,
    ) -> Option<Self> {
        #[cfg(feature = "debug")]
        let timer = Timer::new_dynamic(format!("load font: {}", name.magenta()));

        let font_base_path = format!("{}\\{}", FONT_FOLDER_PATH, name);
        let ttf_file_path = format!("{}.ttf", font_base_path);
        let map_file_path = format!("{}.png", font_base_path);
        let map_description_file_path = format!("{}.csv", font_base_path);

        let Ok(font_data) = game_file_loader.get(&ttf_file_path) else {
            #[cfg(feature = "debug")]
            print_debug!("[{}] failed to load font file '{}'", "error".red(), ttf_file_path.magenta());
            return None;
        };

        let font_data = Arc::new(font_data);
        let ids = font_system.db_mut().load_font_source(Source::Binary(font_data.clone()));

        let Ok(font_map_data) = game_file_loader.get(&map_file_path) else {
            #[cfg(feature = "debug")]
            print_debug!("[{}] failed to load font map file '{}'", "error".red(), map_file_path.magenta());
            return None;
        };

        let font_map_reader = ImageReader::with_format(Cursor::new(font_map_data), ImageFormat::Png);

        let Ok(font_map_decoder) = font_map_reader.decode() else {
            #[cfg(feature = "debug")]
            print_debug!("[{}] failed to decode font map '{}'", "error".red(), map_file_path.magenta());
            return None;
        };

        let font_map_rgba_image = font_map_decoder.into_rgba8();
        let font_map_width = font_map_rgba_image.width();
        let font_map_height = font_map_rgba_image.height();

        let Ok(font_description_data) = game_file_loader.get(&map_description_file_path) else {
            #[cfg(feature = "debug")]
            print_debug!(
                "[{}] failed to load font map description file '{}'",
                "error".red(),
                map_description_file_path.magenta()
            );
            return None;
        };

        let Ok(font_description_content) = String::from_utf8(font_description_data) else {
            #[cfg(feature = "debug")]
            print_debug!(
                "[{}] invalid UTF-8 text data found in font map description file '{}'",
                "error".red(),
                map_description_file_path.magenta()
            );
            return None;
        };

        let glyphs = parse_glyphs(font_description_content, font_map_width, font_map_height);

        let glyphs = match csv_format {
            CsvFormat::GlyphId => glyphs,
            CsvFormat::Codepoint => {
                let Ok(face) = ttf_parser::Face::parse(font_data.as_ref(), 0) else {
                    #[cfg(feature = "debug")]
                    print_debug!("[{}] failed to parse font face for '{}'", "error".red(), ttf_file_path.magenta());
                    return None;
                };
                glyphs
                    .into_iter()
                    .filter_map(|(codepoint, coordinate)| {
                        let character = char::from_u32(u32::from(codepoint))?;
                        let glyph_id = face.glyph_index(character)?;
                        Some((glyph_id.0, coordinate))
                    })
                    .collect()
            }
        };

        #[cfg(feature = "debug")]
        timer.stop();

        Some(Self {
            ids: Vec::from_iter(ids),
            font_map: font_map_rgba_image,
            glyphs: Arc::new(glyphs),
        })
    }
}
