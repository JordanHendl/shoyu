use super::json::*;
use super::load_funcs::*;
use std::collections::HashMap;

pub struct SpriteEntry {
    pub cfg: SpriteJSONEntry,
    pub loaded: Option<ImageInfo<u8>>,
}

impl SpriteEntry {
    pub fn load(&mut self, base_path: &str) {
        self.loaded = Some(
            load_image_rgba8(&format!("{}/{}", base_path, self.cfg.image_path.as_str())).unwrap(),
        );
    }

    pub fn unload(&mut self) {
        self.loaded = None;
    }
}

pub struct SpriteSheetEntry {
    pub cfg: SpriteSheetJSONEntry,
    pub loaded: Option<ImageInfo<u8>>,
}

impl SpriteSheetEntry {
    pub fn load(&mut self, base_path: &str) {
        self.loaded = Some(
            load_image_rgba8(&format!("{}/{}", base_path, self.cfg.image_path.as_str())).unwrap(),
        );
    }

    pub fn unload(&mut self) {
        self.loaded = None;
    }
}

pub fn parse_sprite_sheets(info: SpriteSheetJSON) -> HashMap<String, SpriteSheetEntry> {
    let tup_vec: Vec<(String, SpriteSheetEntry)> = info
        .sprite_sheets
        .into_iter()
        .map(|a| {
            (
                a.name.clone(),
                SpriteSheetEntry {
                    cfg: a.clone(),
                    loaded: None,
                },
            )
        })
        .collect();

    return tup_vec.into_iter().collect();
}

pub fn parse_sprites(info: SpriteJSON) -> HashMap<String, SpriteEntry> {
    let tup_vec: Vec<(String, SpriteEntry)> = info
        .sprites
        .into_iter()
        .map(|a| {
            (
                a.name.clone(),
                SpriteEntry {
                    cfg: a.clone(),
                    loaded: None,
                },
            )
        })
        .collect();

    return tup_vec.into_iter().collect();
}
