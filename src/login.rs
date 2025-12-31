use bevy::{prelude::*, text::LineHeight};
use bevy_ui_text_input::{
    TextInputBuffer, TextInputMode, TextInputNode, TextInputPlugin, TextInputPrompt,
};

use crate::{
    archipelago::{ArchipelagoState, ConnectedMessage, StartConnect},
    assets::UiAtlas,
};

pub struct LoginScreenPlugin;
impl Plugin for LoginScreenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(TextInputPlugin)
            .init_resource::<UiAtlas>()
            .add_systems(Startup, setup)
            .add_systems(Update, connect_button_system)
            .add_systems(Update, hide_when_connected);
    }
}

mod tag {
    use super::*;

    #[derive(Component)]
    pub struct Root;

    #[derive(Component)]
    pub struct InputAddress;

    #[derive(Component)]
    pub struct InputSlotName;

    #[derive(Component)]
    pub struct InputPassword;

    #[derive(Component)]
    pub struct ButtonConnect;
}

fn input_field(
    ui_tag: impl Component,
    prompt: impl Into<String>,
    atlas: &UiAtlas,
    font: Handle<Font>,
) -> impl Bundle {
    (
        ImageNode::from_atlas_image(
            atlas.1.clone(),
            TextureAtlas {
                layout: atlas.0.clone(),
                index: 4,
            },
        )
        .with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(6.),
            ..default()
        })),
        Node {
            padding: UiRect::all(px(5)),
            ..default()
        },
        children![(
            ui_tag,
            Pickable::default(),
            TextInputNode {
                mode: TextInputMode::SingleLine,
                clear_on_submit: false,
                ..Default::default()
            },
            TextColor(Color::srgb(0., 0., 0.)),
            TextFont {
                font,
                font_size: 18.0,
                line_height: LineHeight::Px(20.0),
                ..default()
            },
            Node {
                width: px(250),
                height: px(20),
                ..default()
            },
            TextInputPrompt::new(prompt),
        )],
    )
}

fn setup(mut commands: Commands, atlas: Res<UiAtlas>, asset_server: Res<AssetServer>) {
    let bold_font = asset_server.load("fuzzybubbles-bold.ttf");
    let regular_font = asset_server.load("fuzzybubbles-regular.ttf");
    let root = (
        tag::Root,
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(1., 1., 1., 0.5)),
    );

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
            margin: UiRect::all(px(50)),
            padding: UiRect::all(px(20)),
            flex_grow: 1.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
        },
    );

    let text = (
        Node { ..default() },
        Text::new("elementipelago"),
        TextFont {
            font: bold_font.clone(),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(0., 0., 0.)),
    );

    let submit_button = (
        ImageNode::from_atlas_image(
            atlas.1.clone(),
            TextureAtlas {
                layout: atlas.0.clone(),
                index: 1,
            },
        )
        .with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(8.),
            ..default()
        })),
        Node {
            padding: UiRect::all(px(5)),
            ..default()
        },
        Button,
        tag::ButtonConnect,
        Name::new("btn_connect"),
        children![(
            Text::new("Connect"),
            TextFont {
                font: bold_font,
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::BLACK),
        )],
    );

    commands.spawn((
        root,
        children![(
            square,
            children![
                text,
                input_field(
                    tag::InputAddress,
                    "archipelago.gg:12345",
                    &atlas,
                    regular_font.clone()
                ),
                input_field(
                    tag::InputSlotName,
                    "slot name",
                    &atlas,
                    regular_font.clone()
                ),
                input_field(tag::InputPassword, "password", &atlas, regular_font.clone()),
                submit_button
            ]
        )],
    ));
}

fn connect_button_system(
    mut commands: Commands,
    mut ap_state: ResMut<ArchipelagoState>,
    interaction: Single<&Interaction, (Changed<Interaction>, With<tag::ButtonConnect>)>,
    input_addr: Single<&TextInputBuffer, With<tag::InputAddress>>,
    input_slot: Single<&TextInputBuffer, With<tag::InputSlotName>>,
    input_pass: Single<&TextInputBuffer, With<tag::InputPassword>>,
) {
    match *interaction {
        Interaction::Pressed => {
            // ap_state.address = input_addr.get_text();
            // ap_state.slot = input_slot.get_text();
            // ap_state.password = input_pass.get_text();
            ap_state.address = "localhost:38281".to_string();
            ap_state.slot = "noa".to_string();
            ap_state.password = "".to_string();

            println!(
                "connecting to {} with {} and {}",
                ap_state.address, ap_state.slot, ap_state.password
            );
            commands.trigger(StartConnect);
        }
        Interaction::Hovered => {}
        Interaction::None => {}
    };
}

fn hide_when_connected(
    mut read_connected: MessageReader<ConnectedMessage>,
    mut root: Single<&mut Visibility, With<tag::Root>>,
) {
    if read_connected.read().count() > 0 {
        **root = Visibility::Hidden;
    }
}
