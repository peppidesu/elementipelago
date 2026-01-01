use bevy::{camera::visibility::RenderLayers, prelude::*, window::PrimaryWindow};

mod archipelago;
mod assets;
mod game;
mod graph;
mod input;
mod login;
mod util;

use crate::{
    archipelago::ArchipelagoPlugin,
    game::{PlayfieldPlugin, RecipeGraph},
    login::LoginScreenPlugin,
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode as EmbeddedAssetPluginMode};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Login,
    InGame,
}

fn setup(mut commands: Commands, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    commands.spawn((
        Camera2d,
        Msaa::Off,
        IsDefaultUiCamera,
        UiPickingCamera,
        MeshPickingCamera,
    ));
    commands.spawn((
        Camera2d,
        Msaa::Off,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(1),
    ));
    window.resize_constraints.min_width = 640.0;
    window.resize_constraints.min_height = 480.0;

    let base_scale = window.resolution.base_scale_factor();
    window
        .resolution
        .set_scale_factor_override(Some(base_scale * 2.0));
}

fn main() {
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: EmbeddedAssetPluginMode::ReplaceDefault,
            },
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            LoginScreenPlugin,
            PlayfieldPlugin,
            ArchipelagoPlugin,
        ))
        .init_state::<AppState>()
        .insert_resource(UiPickingSettings {
            require_markers: true,
        })
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .insert_resource(RecipeGraph(None))
        .add_systems(Startup, setup)
        .run();
}
