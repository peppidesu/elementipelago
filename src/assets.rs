use bevy::{platform::collections::HashMap, prelude::*};

use crate::atlas::AtlasDef;

macro_rules! auto_atlas_resource {
    ($($name:ident ($path:expr, $size:expr, $cols:expr, $rows:expr)),*) => {
        $(
            #[derive(Resource)]
            pub struct $name(
                #[allow(dead_code)]
                pub Handle<TextureAtlasLayout>,
                #[allow(dead_code)]
                pub Handle<Image>
            );

            impl FromWorld for $name {
                fn from_world(world: &mut World) -> Self {
                    let texture_atlas = TextureAtlasLayout::from_grid(
                        $size.into(), // The size of each image
                        $cols,        // The number of columns
                        $rows,        // The number of rows
                        None,         // Padding
                        None,         // Offset
                    );

                    let mut texture_atlases = world
                        .get_resource_mut::<Assets<TextureAtlasLayout>>()
                        .unwrap();
                    let texture_atlas_handle = texture_atlases.add(texture_atlas);

                    let image = world.get_resource::<AssetServer>().unwrap().load($path);

                    Self(texture_atlas_handle, image)
                }
            }
        )*
    };
}

#[derive(Resource)]
pub struct ElementAtlas(
    pub Handle<TextureAtlasLayout>,
    pub Handle<Image>,
    pub HashMap<String, usize>,
);

impl FromWorld for ElementAtlas {
    fn from_world(world: &mut World) -> Self {
        let config_str = include_str!("../assets/element-atlas.json");
        let config: AtlasDef =
            serde_json::from_str(config_str).expect("assets/element-atlas.json is invalid json");
        let image_path = config.meta.image;

        let mut texture_atlas = TextureAtlasLayout::new_empty(config.meta.size.clone().into());
        let mut texture_indices = HashMap::with_capacity(config.frames.len());

        for (name, frame) in config.frames {
            let rect = frame.frame;
            let urect: URect = rect.into();
            debug_assert!(
                urect.max.x <= config.meta.size.w,
                "frame with name {name} goes out of bounds (in x)"
            );
            debug_assert!(
                urect.max.y <= config.meta.size.h,
                "frame with name {name} goes out of bounds (in y)"
            );
            texture_indices.insert(name, texture_atlas.add_texture(urect));
        }

        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        let image = world
            .get_resource::<AssetServer>()
            .unwrap()
            .load(image_path);

        Self(texture_atlas_handle, image, texture_indices)
    }
}

auto_atlas_resource! {
    // ElementAtlas("element-atlas.png", (48, 48), 5, 5),
    UiAtlas("ui-atlas.png", (32, 32), 4, 4)
}
