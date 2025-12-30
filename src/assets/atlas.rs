use bevy::{
    math::{URect, UVec2},
    platform::collections::HashMap,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ImageSize {
    pub w: u32,
    pub h: u32,
}

impl Into<UVec2> for ImageSize {
    fn into(self) -> UVec2 {
        UVec2 {
            x: self.w,
            y: self.h,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct ImageRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Into<URect> for ImageRect {
    fn into(self) -> URect {
        URect::new(self.x, self.y, self.x + self.w, self.y + self.h)
    }
}

#[derive(Deserialize)]
pub struct AtlasMeta {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: String,
    pub size: ImageSize,
    pub scale: String, // TODO: parse this string into a float
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasFrame {
    pub frame: ImageRect,
    pub rotated: bool,
    pub trimmed: bool,
    pub sprite_source_size: ImageRect,
    pub source_size: ImageSize,
    pub duration: usize,
}

#[derive(Deserialize)]
pub struct AtlasDef {
    pub meta: AtlasMeta,
    pub frames: HashMap<String, AtlasFrame>,
}

impl AtlasDef {
    pub fn get_sprite_location(&self, name: &str) -> Option<ImageRect> {
        self.frames
            .get(name)
            .and_then(|frame| Some(frame.frame.clone()))
    }
}
