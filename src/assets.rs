use bevy::{platform::collections::HashMap, prelude::*};

use crate::atlas::AtlasDef;

macro_rules! auto_atlas_resource {
    ($($name:ident ($path:expr)),*) => {
        $(
            #[derive(Resource)]
            pub struct $name(
                #[allow(dead_code)]
                pub Handle<TextureAtlasLayout>,
                #[allow(dead_code)]
                pub Handle<Image>,
                #[allow(dead_code)]
                pub HashMap<String, usize>,
            );

            impl FromWorld for $name {
                fn from_world(world: &mut World) -> Self {
                    let config_str = include_str!(concat!("../assets/", $path));
                    let config: AtlasDef = serde_json::from_str(config_str).expect(concat!("assets/",$path," is invalid json"));
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
        )*
    };
    // TODO: remove this arm once UiAtlas is moved to new one
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

auto_atlas_resource! {
    ElementAtlas("element-atlas.json")
    //UiAtlas("ui-atlas.json")
}

// until ui-atlas.json exists this works
auto_atlas_resource! {
    UiAtlas("ui-atlas.png", (32, 32), 4, 4)
}
