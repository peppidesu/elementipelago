//! Useful helper functions and traits

use bevy::{
    asset::Assets,
    ecs::message::{Message, MessageReader},
    image::Image,
    math::{Rect, Vec3Swizzles},
    sprite::Sprite,
    transform::components::GlobalTransform,
};

/// Sprite aabb check
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

/// Run condition that triggers when any message of this type is received.
pub fn any_message<T: Message>(mut reader: MessageReader<T>) -> bool {
    reader.read().count() > 0
}
