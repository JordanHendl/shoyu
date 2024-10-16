use dashi::utils::*;
use dashi::*;

pub struct SpriteInfo<'a> {
    pub name: &'a str,
    pub db_key: &'a str,
}

pub struct Sprite {
    pub dim: [u32; 2],
    pub handle: Handle<Image>,
    pub bg: Handle<BindGroup>,
}

pub struct SpriteSheetInfo<'a> {
    pub name: &'a str,
    pub db_key: &'a str,
}

pub struct SpriteSheet {
    pub dim: [u32; 2],
    pub handle: Handle<Image>,
    pub bg: Handle<BindGroup>,
}