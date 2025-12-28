use bevy::prelude::*;

mod archipelago;
mod game;
mod graph;
mod input;
mod util;

use crate::{archipelago::ArchipelagoPlugin, game::PlayfieldPlugin};

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PlayfieldPlugin,
            ArchipelagoPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .add_systems(Startup, setup)
        .run();
}
