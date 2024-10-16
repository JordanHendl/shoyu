use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
pub struct SpriteJSONEntry {
    pub name: String,
    pub image_path: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SpriteJSON {
    pub sprites: Vec<SpriteJSONEntry>,        
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SpriteSheetJSONEntry {
    pub name: String,
    pub image_path: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SpriteSheetJSON {
    pub sprite_sheets: Vec<SpriteSheetJSONEntry>,        
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DatabaseJSON {
    pub sprite_cfg: Option<String>,
    pub sprite_sheet_cfg: Option<String>,
}
