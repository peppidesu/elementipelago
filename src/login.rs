use bevy::prelude::*;

use crate::atlases::UIAtlas;

pub struct LoginScreenPlugin;
impl Plugin for LoginScreenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<UIAtlas>().add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, atlas: Res<UIAtlas>) {
    let container = Node {
        width: percent(100),
        height: percent(100),
        justify_content: JustifyContent::Center,
        ..default()
    };

    let square = (
        ImageNode::from_atlas_image(
            atlas.1.clone(),
            TextureAtlas {
                layout: atlas.0.clone(),
                index: 0,
            },
        )
        .with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(6.),
            ..default()
        })),
        Node {
            margin: UiRect::all(px(100)),

            ..default()
        },
    );

    commands.spawn((container, children![square]));
}
