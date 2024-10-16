use serde::{Deserialize, Serialize};
pub mod error;
pub use error::*;
pub mod json;
pub use json::*;
use std::collections::HashMap;
use std::fs;
mod load_funcs;
use load_funcs::*;
mod images;
use images::*;

pub struct Database {
    base_path: String,
    sprites: HashMap<String, SpriteEntry>,
    sprite_sheets: HashMap<String, SpriteSheetEntry>,
}

impl Database {
    fn get_sprite_json(path: &str) -> Result<SpriteJSON, Error> {
        let json_data = fs::read_to_string(path)?;
        let info: SpriteJSON = serde_json::from_str(&json_data)?;
        Ok(info)
    }

    fn get_sprite_sheet_json(path: &str) -> Result<SpriteSheetJSON, Error> {
        let json_data = fs::read_to_string(path)?;
        let info: SpriteSheetJSON = serde_json::from_str(&json_data)?;
        Ok(info)
    }

    pub fn new(base_path: &str) -> Result<Self, Error> {
        let json_data = fs::read_to_string(format!("{}/shoyu.json", base_path))?;

        let info: DatabaseJSON = serde_json::from_str(&json_data)?;

        let sprites = if let Some(sprite) = info.sprite_cfg {
            parse_sprites(Database::get_sprite_json(&format!(
                "{}/{}",
                base_path,
                sprite.as_str()
            ))?)
        } else {
            HashMap::new()
        };

        let sprite_sheets = if let Some(sprite) = info.sprite_sheet_cfg {
            parse_sprite_sheets(Database::get_sprite_sheet_json(&format!(
                "{}/{}",
                base_path,
                sprite.as_str()
            ))?)
        } else {
            HashMap::new()
        };

        Ok(Database {
            base_path: base_path.to_string(),
            sprites,
            sprite_sheets,
        })
    }

    pub fn fetch_sprite(&mut self, name: &str) -> Result<&SpriteEntry, Error> {
        // TODO probably async this.
        if let Some(entry) = self.sprites.get_mut(name) {
            if entry.loaded.is_none() {
                entry.load(&self.base_path);
            }

            return Ok(entry);
        }

        return Err(Error::LookupError(LookupError {
            entry: name.to_string(),
        }));
    }

    pub fn fetch_sprite_sheet(&mut self, name: &str) -> Result<&SpriteSheetEntry, Error> {
        // TODO probably async this.
        if let Some(entry) = self.sprite_sheets.get_mut(name) {
            if entry.loaded.is_none() {
                entry.load(&self.base_path);
            }

            return Ok(entry);
        }

        return Err(Error::LookupError(LookupError {
            entry: name.to_string(),
        }));
    }
}

#[test]
fn test_database() {
    let res = Database::new("/wksp/database");
    assert!(res.is_ok());

    let mut db = res.unwrap();
    let sprite = db.fetch_sprite("name");
    assert!(sprite.is_ok());

    let sprite = db.fetch_sprite_sheet("name");
    assert!(sprite.is_ok());
}
