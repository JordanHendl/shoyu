use std::collections::HashMap;

use crate::database::font::*;
use dashi::utils::*;
use dashi::*;

pub struct FontInfo<'a> {
    pub name: &'a str,
    pub db_key: &'a str,
}

pub struct SpriteInfo<'a> {
    pub name: &'a str,
    pub db_key: &'a str,
}

pub struct Sprite {
    pub dim: [u32; 2],
    pub handle: Handle<Image>,
    pub view: Handle<ImageView>,
    pub bg: Handle<BindGroup>,
}

pub struct SpriteSheetInfo<'a> {
    pub name: &'a str,
    pub db_key: &'a str,
}

pub struct SpriteSheet {
    pub dim: [u32; 2],
    pub handle: Handle<Image>,
    pub view: Handle<ImageView>,
    pub bg: Handle<BindGroup>,
    pub sprites: HashMap<u32, FRect2D>,
}

pub struct Font {
    pub dim: [u32; 2],
    pub atlas: Handle<Image>,
    pub atlas_view: Handle<ImageView>,
    pub font: *const TTFont,
    pub bg: Handle<BindGroup>,
}
