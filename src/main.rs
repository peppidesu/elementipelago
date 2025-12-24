use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseButtonInput, MouseMotion},
    math::{
        VectorSpace,
        bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    },
    prelude::*,
    window::PrimaryWindow,
};

mod graph;

#[derive(Component, Clone)]
struct Element(u32);

#[derive(Component, Clone)]
struct ElementSource;

#[derive(Component)]
struct ElementDrawer {
    page: u32,
}

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct BeingDragged;

#[derive(Message)]
struct ElementDropped;

mod input {
    use super::*;
    pub fn primary_start(buttons: Res<ButtonInput<MouseButton>>) -> bool {
        buttons.just_pressed(MouseButton::Left)
    }
    pub fn primary_end(buttons: Res<ButtonInput<MouseButton>>) -> bool {
        buttons.just_released(MouseButton::Left)
    }
}

fn cursor_move(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut transform: Single<&mut Transform, With<Cursor>>,
) {
    let (camera, camera_transform) = *camera_query;
    if let Some(position) = window.cursor_position()
        && let Ok(worldpos) = camera.viewport_to_world_2d(camera_transform, position)
    {
        transform.translation.x = worldpos.x;
        transform.translation.y = worldpos.y;
    }
}

fn drag_end(
    mut commands: Commands,
    mut dropped_msg: MessageWriter<ElementDropped>,
    drag_query: If<Single<Entity, With<BeingDragged>>>,
) {
    commands
        .entity(**drag_query)
        .remove::<BeingDragged>()
        .remove_parent_in_place();

    dropped_msg.write(ElementDropped);
}

fn drag_begin(
    mut commands: Commands,
    cursor_query: Single<(Entity, &Transform), With<Cursor>>,
    mut element_query: Query<
        (Entity, &mut Transform, Option<&ElementSource>),
        (With<Element>, Without<Cursor>),
    >,
) {
    let (cursor, cursor_transform) = *cursor_query;

    let Some((drag_candidate, mut drag_candidate_tf, drag_is_source)) = element_query
        .iter_mut()
        .filter(|(_, tf, _)| {
            let aabb = Aabb2d::new(tf.translation.xy(), tf.scale.xy() / 2.0);
            let cursor_aabb = Aabb2d::new(cursor_transform.translation.xy(), Vec2::ZERO);

            aabb.contains(&cursor_aabb)
        })
        .max_by(|(_, tf1, _), (_, tf2, _)| tf1.translation.z.total_cmp(&tf2.translation.z))
    else {
        return;
    };
    if drag_is_source.is_some() {
        commands.entity(drag_candidate).clone_and_spawn();
        commands.entity(drag_candidate).remove::<ElementSource>();
    }
    commands
        .entity(drag_candidate)
        .set_parent_in_place(cursor)
        .insert(BeingDragged);
}

fn recalculate_element_z_order(
    mut dropped_msg: MessageReader<ElementDropped>,
    mut element_query: Query<&mut Transform, (With<Element>, Without<ElementSource>)>,
) {
    if dropped_msg.read().count() == 0 {
        return;
    }
    let count = element_query.count();

    element_query
        .iter_mut()
        .sort_by::<&Transform>(|tf1, tf2| tf1.translation.z.total_cmp(&tf2.translation.z))
        .enumerate()
        .for_each(|(i, mut tf)| {
            tf.translation.z = -2.0 + ((i as f32) / (count as f32));
        });
}
fn bring_dragged_to_top(mut tf: If<Single<&mut Transform, Added<BeingDragged>>>) {
    tf.translation.z = -0.1;
}

pub struct PlayfieldPlugin;
impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                drag_begin.run_if(input::primary_start),
                drag_end.run_if(input::primary_end),
                bring_dragged_to_top,
            )
                .chain(),
        )
        .add_systems(PostUpdate, recalculate_element_z_order);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Cursor,
        Transform {
            translation: Vec3::new(1.0, 1.0, 10.0),
            scale: Vec3::new(25.0, 25.0, 1.0),
            ..default()
        },
        Sprite::from_color(Color::srgb(0.0, 1.0, 0.0), Vec2::ONE),
    ));

    commands.spawn((
        Element(0),
        ElementSource,
        Transform {
            translation: Vec3::new(-150.0, 0.0, 1.0),
            scale: Vec3::new(100.0, 100.0, 1.0),
            ..default()
        },
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::ONE),
    ));

    commands.spawn((
        Element(1),
        ElementSource,
        Transform {
            translation: Vec3::new(150.0, 0.0, 1.0),
            scale: Vec3::new(100.0, 100.0, 1.0),
            ..default()
        },
        Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), Vec2::ONE),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayfieldPlugin)
        .add_message::<ElementDropped>()
        .add_systems(Startup, setup)
        .add_systems(Update, cursor_move)
        .run();
}
