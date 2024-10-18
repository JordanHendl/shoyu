use dashi::Rect2D;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct Glyph {
    pub bounds: Rect2D, // where this glyph is in the atlas
    pub advance: f32,
}

pub struct TTFont {
    pub glyphs: HashMap<char, Glyph>,
    pub atlas: Option<Vec<u8>>, // Cache for the entire glyph atlas as a single image
    pub atlas_width: u32,
    pub atlas_height: u32,
}

impl TTFont {
    /// Creates a new TTFont instance by loading a font from a specified file path.
    pub fn new(file_path: &str, width: u32, height: u32, font_size: f32, range: &[char]) -> Self {
        // Create a new bitmap (initialized to zero)
        let mut bitmap = vec![0u8; (width * height) as usize];

//        let mut file = File::open(file_path).unwrap();
//        let mut font_data = Vec::new();
//        file.read_to_end(&mut font_data).unwrap();
        let font_data = std::fs::read(file_path).unwrap();
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap();

        // Coordinates to keep track of where to draw the next glyph
        let mut cursor_x = 0;
        let mut cursor_y = 0;
        let mut line_height = 0;
        let mut glyph_map = HashMap::new();

        for ch in range {
            let (metrics, bitmap_data) = font.rasterize_indexed(font.lookup_glyph_index(*ch), font_size);
            {
                // If the character doesn't fit in the current row, move to the next row
                if cursor_x + metrics.width >= width as usize {
                    cursor_x = 0;
                    cursor_y += line_height;
                    line_height = 0;
                }

                // If the character doesn't fit in the current bitmap, break
                if cursor_y + metrics.height >= height as usize {
                    break;
                }

                // Draw the glyph into the bitmap
                for y in 0..metrics.height {
                    for x in 0..metrics.width {
                        let bitmap_x = cursor_x + x;
                        let bitmap_y = cursor_y + y;
                        let bitmap_index = bitmap_y * width as usize + bitmap_x;
                        bitmap[bitmap_index as usize] = bitmap_data[y * metrics.width + x];
                    }
                }

                // Store the glyph boundary
                glyph_map.insert(
                    *ch,
                    Glyph {
                        bounds: Rect2D {
                            x: cursor_x as u32,
                            y: cursor_y as u32,
                            w: metrics.width as u32,
                            h: metrics.height as u32,
                        },
                        advance: metrics.advance_width,
                    },
                );

                // Update the cursor and line height
                cursor_x += metrics.width;
                line_height = line_height.max(metrics.height);
            }
        }

        Self {
            glyphs: glyph_map,
            atlas: Some(bitmap),
            atlas_width: width,
            atlas_height: height,
        }
    }
}
