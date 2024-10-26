use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ParticleSystemJSONAnimation {
    pub name: String,
    pub id: u32,
    pub sprites: Vec<dashi::Rect2D>,
    pub time_per_frame_ms: f32
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ParticleSystemJSONAutoGen {
    pub name: String,
    pub bounds: dashi::Rect2D,
    pub stride: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ParticleSystemJSONEntry {
    pub name: String,
    pub id: u32,
    pub image_path: String,
    pub animations: Vec<ParticleSystemJSONAnimation>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ParticleSystemJSON {
    pub particles: Vec<ParticleSystemJSONEntry>,
}
