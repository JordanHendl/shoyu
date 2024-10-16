use super::types::*;
use crate::database::*;
use dashi::utils::*;
use dashi::*;
use std::collections::HashMap;
pub struct ResourceManager {
    database: Database,
    sprites: Pool<Sprite>,
    sprite_sheets: Pool<SpriteSheet>,
}

impl ResourceManager {
    pub fn new(ctx: *mut Context, database: Database) -> Self {
        todo!()
    }
    
    pub fn fetch_sprite(& mut self, handle: Handle<Sprite>) -> & mut Sprite {
        todo!()
    }

    pub fn fetch_sprite_sheet(& mut self, handle: Handle<SpriteSheet>) -> & mut SpriteSheet {
        todo!()
    }

    pub fn make_sprite(&mut self, info: &SpriteInfo) -> Handle<Sprite> {
        todo!()
    }

    pub fn make_sprite_sheet(&mut self, info: &SpriteSheetInfo) -> Handle<SpriteSheet> {
        todo!()
    }

    pub fn release_sprite(&mut self, handle: Handle<Sprite>) {}

    pub fn release_sprite_sheet(&mut self, handle: Handle<SpriteSheet>) {}
}
