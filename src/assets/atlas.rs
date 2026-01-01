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

impl From<ImageSize> for UVec2 {
    fn from(val: ImageSize) -> Self {
        UVec2 {
            x: val.w,
            y: val.h,
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

impl From<ImageRect> for URect {
    fn from(val: ImageRect) -> Self {
        URect::new(val.x, val.y, val.x + val.w, val.y + val.h)
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct AtlasMeta {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: String,
    pub size: ImageSize,
    pub scale: String, // TODO: parse this string into a float
}

#[allow(dead_code)]
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
