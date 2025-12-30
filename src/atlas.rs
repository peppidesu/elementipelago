use bevy::platform::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
struct ImageSize {
    w: usize,
    h: usize,
}

#[derive(Deserialize, Clone)]
pub struct ImageRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Deserialize)]
struct AtlasMeta {
    app: String,
    version: String,
    image: String,
    format: String,
    size: ImageSize,
    scale: String, // TODO: parse this string into a float
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AtlasFrame {
    frame: ImageRect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: ImageRect,
    source_size: ImageSize,
    duration: usize,
}

#[derive(Deserialize)]
pub struct AtlasDef {
    meta: AtlasMeta,
    frames: HashMap<String, AtlasFrame>,
}

impl AtlasDef {
    pub fn get_sprite_location(&self, name: &str) -> Option<ImageRect> {
        self.frames
            .get(name)
            .and_then(|frame| Some(frame.frame.clone()))
    }
}
