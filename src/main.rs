use bevy::{prelude::*, window::PrimaryWindow};

mod archipelago;
mod atlases;
mod game;
mod graph;
mod input;
mod login;
mod util;

use crate::{
    archipelago::ArchipelagoPlugin,
    atlases::{ElementAtlas, UIAtlas},
    game::{PlayfieldPlugin, Recipes},
    login::LoginScreenPlugin,
};

#[derive(Component)]
pub struct Persistent;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Login,
    InGame,
}

fn setup(mut commands: Commands, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    commands.spawn((Camera2d, Msaa::Off));

    window.resolution.set_scale_factor_override(Some(3.0));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            LoginScreenPlugin,
            PlayfieldPlugin,
            ArchipelagoPlugin,
        ))
        .init_state::<AppState>()
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .insert_resource(Recipes(None))
        .add_systems(Startup, setup)
        .run();
}
