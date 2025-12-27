//! Useful helper functions and traits

use bevy::{
    asset::Assets,
    image::Image,
    math::{Rect, Vec2, Vec3Swizzles, bounding::Aabb2d},
    sprite::Sprite,
    transform::components::{GlobalTransform, Transform},
};

pub fn get_sprite_bounds(
    sprite: &Sprite,
    transform: &GlobalTransform,
    assets: &Assets<Image>,
) -> Rect {
    let image_size = sprite
        .custom_size
        .unwrap_or(assets.get(&sprite.image).unwrap().size_f32());
    let scaled = image_size * transform.scale().xy();

    Rect::from_center_size(transform.translation().xy(), scaled)
}
