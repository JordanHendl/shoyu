use rusttype::{point, Font, GlyphId, PositionedGlyph, Rect, Scale};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub struct TTFont {
    font: Font<'static>,
    scale: Scale,
    cache: HashMap<GlyphId, Vec<u8>>, // Cache rasterized glyphs as a vector of u8 representing grayscale pixels
    atlas: Option<Vec<u8>>,           // Cache for the entire glyph atlas as a single image
    atlas_width: usize,
    atlas_height: usize,
}

impl TTFont {
    /// Creates a new TTFont instance by loading a font from a specified file path.
    pub fn new(file_path: &str, font_size: f32) -> Self {
        let mut file = File::open(file_path).expect("Could not open font file");
        let mut font_data = Vec::new();
        file.read_to_end(&mut font_data)
            .expect("Could not read font file");
        let font = Font::try_from_vec(font_data).expect("Error constructing Font");

        Self {
            font,
            scale: Scale::uniform(font_size),
            cache: HashMap::new(),
            atlas: None,
            atlas_width: 0,
            atlas_height: 0,
        }
    }

    /// Generates and rasterizes glyphs for a given text string, caching each glyph image.
    pub fn get_or_rasterize_glyph(&mut self, character: char) -> Option<&Vec<u8>> {
        let glyph = self.font.glyph(character).scaled(self.scale);
        let glyph_id = glyph.id();

        // Check if the glyph is already cached
        if !self.cache.contains_key(&glyph_id) {
            // Position the glyph at the origin (since we are only rasterizing)
            let positioned_glyph = glyph.positioned(point(0.0, 0.0));

            // Rasterize the glyph if it has an outline
            if let Some(bounding_box) = positioned_glyph.pixel_bounding_box() {
                let width = bounding_box.width() as usize;
                let height = bounding_box.height() as usize;

                let mut pixel_data = vec![0u8; width * height];
                positioned_glyph.draw(|x, y, v| {
                    let index = (y as usize) * width + (x as usize);
                    pixel_data[index] = (v * 255.0) as u8;
                });

                // Cache the rasterized image
                self.cache.insert(glyph_id, pixel_data);
            } else {
                return None;
            }
        }

        // Return the cached rasterized image
        self.cache.get(&glyph_id)
    }

    /// Generates glyphs for an entire text string and returns the positioned glyphs.
    pub fn generate_glyphs(&self, text: &str) -> Vec<PositionedGlyph> {
        let v_metrics = self.font.v_metrics(self.scale);
        let offset = point(0.0, v_metrics.ascent);
        self.font.layout(text, self.scale, offset).collect()
    }

    /// Rasterizes all glyphs in the given text and creates a single atlas image.
    pub fn rasterize_atlas(&mut self, text: &str) {
        let glyphs: Vec<_> = text
            .chars()
            .map(|c| {
                self.font
                    .glyph(c)
                    .scaled(self.scale)
                    .positioned(point(0.0, 0.0))
            })
            .collect();

        // Calculate atlas dimensions
        let mut width = 0;
        let mut height = 0;
        for glyph in &glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                width += bb.width() as usize;
                height = height.max(bb.height() as usize);
            }
        }

        let mut atlas_data = vec![0u8; width * height];
        let mut x_offset = 0;

        // Rasterize each glyph into the atlas
        for glyph in &glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                let glyph_width = bb.width() as usize;
                let glyph_height = bb.height() as usize;

                glyph.draw(|x, y, v| {
                    let atlas_index = (y as usize) * width + (x as usize + x_offset);
                    atlas_data[atlas_index] = (v * 255.0) as u8;
                });

                x_offset += glyph_width;
            }
        }

        self.atlas = Some(atlas_data);
        self.atlas_width = width;
        self.atlas_height = height;
    }

    /// Returns the atlas image data along with its dimensions.
    pub fn get_atlas(&self) -> Option<(&Vec<u8>, usize, usize)> {
        self.atlas
            .as_ref()
            .map(|atlas| (atlas, self.atlas_width, self.atlas_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache() {
        let mut glyph_cache = TTFont::new("path/to/your/font.ttf", 24.0);
        let character = 'A';
        let glyph_image = glyph_cache.get_or_rasterize_glyph(character);

        assert!(
            glyph_image.is_some(),
            "Glyph image should be rasterized and cached"
        );
        assert!(
            !glyph_image.unwrap().is_empty(),
            "Rasterized glyph image should not be empty"
        );
    }

    #[test]
    fn test_rasterize_atlas() {
        let mut glyph_cache = TTFont::new("path/to/your/font.ttf", 24.0);
        let text = "Hello, world!";
        glyph_cache.rasterize_atlas(text);

        let (atlas, width, height) = glyph_cache.get_atlas().expect("Atlas should be generated");
        assert!(!atlas.is_empty(), "Atlas should not be empty");
        assert!(
            width > 0 && height > 0,
            "Atlas dimensions should be greater than zero"
        );
    }
}
