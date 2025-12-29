//! Useful helper functions and traits

use bevy::{
    asset::Assets,
    ecs::message::{Message, MessageReader},
    image::{Image, TextureAtlas, TextureAtlasLayout},
    math::{Rect, Vec2, Vec3Swizzles},
    sprite::Sprite,
    transform::components::GlobalTransform,
};

/// Sprite aabb check
pub fn get_sprite_bounds(
    sprite: &Sprite,
    transform: &GlobalTransform,
    assets_image: &Assets<Image>,
    assets_atlas: &Assets<TextureAtlasLayout>,
) -> Rect {
    let image_size = sprite
        .custom_size
        .or(sprite.texture_atlas.as_ref().map(|a| {
            let size = assets_atlas.get(&a.layout).unwrap().textures[a.index].size();
            Vec2::new(size.x as f32, size.y as f32)
        }))
        .unwrap_or(assets_image.get(&sprite.image).unwrap().size_f32());

    let scaled = image_size * transform.scale().xy();

    Rect::from_center_size(transform.translation().xy(), scaled)
}

/// Run condition that triggers when any message of this type is received.
pub fn any_message<T: Message>(mut reader: MessageReader<T>) -> bool {
    reader.read().count() > 0
}
