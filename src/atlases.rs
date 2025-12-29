use bevy::prelude::*;

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

auto_atlas_resource! {
    ElementAtlas("element-atlas.png", (48, 48), 1, 13),
    UIAtlas("ui-atlas.png", (32, 32), 4, 4)
}
