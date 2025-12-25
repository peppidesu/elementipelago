use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{
    math::bounding::{BoundingVolume, IntersectsVolume},
    platform::collections::HashMap,
    prelude::*,
    window::PrimaryWindow,
};
use float_ord::FloatOrd;
use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};

use crate::util::*;

mod graph;
mod util;

#[derive(Component, Clone)]
struct Element(u64);
impl Element {
    fn build(id: u64, pos: Vec2) -> (Element, Transform, Sprite) {
        let mut rng = SmallRng::seed_from_u64(id);
        let dark = rng.random_bool(0.5);
        let color = if dark {
            Color::hsl(rng.random::<f32>() * 360.0, 1.0, 0.75)
        } else {
            Color::hsl(rng.random::<f32>() * 360.0, 1.0, 0.25)
        };
        (
            Element(id),
            Transform {
                translation: Vec3::new(pos.x, pos.y, 1.0),
                scale: Vec3::new(64.0, 64.0, 1.0),
                ..default()
            },
            Sprite::from_color(color, Vec2::ONE),
        )
    }

    fn build_source(id: u64, pos: Vec2) -> (Element, Transform, Sprite, ElementSource) {
        let (el, tf, sp) = Self::build(id, pos);
        (el, tf, sp, ElementSource)
    }
}

#[derive(Component, Clone)]
struct ElementSource;

#[derive(Component)]
struct ElementDrawer;

#[derive(Component)]
struct BeingDragged;

#[derive(Component)]
struct JustDropped;

#[derive(Message)]
struct ElementDropped;

#[derive(Component)]
struct Cursor;

#[derive(Resource)]
struct Recipes(HashMap<(u64, u64), u64>);

impl Recipes {
    fn get_recipe(&self, el1: u64, el2: u64) -> Option<u64> {
        self.0
            .get(&(el1, el2))
            .or_else(|| self.0.get(&(el2, el1)))
            .map(|e| *e)
    }
}

mod input {
    use super::*;
    pub fn primary_start(buttons: Res<ButtonInput<MouseButton>>) -> bool {
        buttons.just_pressed(MouseButton::Left)
    }
    pub fn primary_end(buttons: Res<ButtonInput<MouseButton>>) -> bool {
        buttons.just_released(MouseButton::Left)
    }
}

fn any_message<T: Message>(mut reader: MessageReader<T>) -> bool {
    reader.read().count() > 0
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
        .insert(JustDropped)
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
            tf.into_aabb2d()
                .contains(&cursor_transform.translation.xy().into_aabb2d())
        })
        .max_by_key(|(_, tf, _)| FloatOrd(tf.translation.z))
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
    mut element_query: Query<&mut Transform, (With<Element>, Without<ElementSource>)>,
) {
    let count = element_query.count();

    element_query
        .iter_mut()
        .sort_by::<&Transform>(|tf1, tf2| tf1.translation.z.total_cmp(&tf2.translation.z))
        .enumerate()
        .for_each(|(i, mut tf)| {
            tf.translation.z = -2.0 + ((i as f32) / (count as f32));
        });
}

fn merge_elements(
    mut commands: Commands,
    recipes: Res<Recipes>,
    just_dropped_query: Single<(Entity, &Element, &Transform), With<JustDropped>>,
    element_query: Query<
        (Entity, &Element, &Transform),
        (Without<ElementSource>, Without<JustDropped>),
    >,
) {
    let (dropped_root, dropped_el, dropped_tf) = *just_dropped_query;
    commands.entity(dropped_root).remove::<JustDropped>();

    let dropped_aabb = dropped_tf.into_aabb2d();
    let Some((other_root, other_el, other_tf, result_el)) = element_query
        .iter()
        // Only elements that intersect
        .filter(|(_, _, tf)| dropped_aabb.intersects(&tf.into_aabb2d()))
        // Only elements that can merge with this one
        .filter_map(|(r, el, tf)| {
            recipes
                .get_recipe(dropped_el.0, el.0)
                .map(|result| (r, el, tf, result))
        })
        // Element with highest z-order (top-most)
        .max_by_key(|(_, _, tf, _)| FloatOrd(tf.translation.z))
    else {
        // None -> no candidate element found
        return;
    };

    // New position halfway between others
    let new_pos = dropped_tf
        .translation
        .xy()
        .interpolate_stable(&other_tf.translation.xy(), 0.5);

    // spawn product element
    commands.spawn(Element::build(result_el, new_pos));

    // despawn ingredient elements
    commands.entity(dropped_root).despawn();
    commands.entity(other_root).despawn();
}

fn bring_dragged_to_top(mut tf: If<Single<&mut Transform, Added<BeingDragged>>>) {
    tf.translation.z = -0.1;
}

pub struct PlayfieldPlugin;
impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ElementDropped>().add_systems(
            Update,
            (
                (
                    drag_begin.run_if(input::primary_start),
                    bring_dragged_to_top,
                )
                    .chain(),
                (
                    drag_end.run_if(input::primary_end),
                    merge_elements.run_if(any_message::<ElementDropped>),
                    recalculate_element_z_order.run_if(any_message::<ElementDropped>),
                )
                    .chain(),
            ),
        );
    }
}

fn setup(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
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

    let mut drawer = commands.spawn((
        ElementDrawer,
        Transform {
            translation: Vec3::new(-200.0, 0.0, 0.0),
            ..default()
        },
    ));

    let gap = 72.0;
    let count = 15;
    let cols = 3;
    let height = (count / cols) as f32 * gap;
    let offset_y = (height - gap) * 0.5;
    let offset_x = -((cols - 1) as f32 * gap) * 0.5;

    for i in 0..count {
        let x = (i % cols) as f32 * 72.0 + offset_x;
        let y = offset_y - (i / cols) as f32 * 72.0;

        drawer.with_child(Element::build_source(i, Vec2::new(x, y)));
    }
}

fn main() {
    let generated_graph = graph::create_graph(5, 5, 2827108, 5, 4).0;
    println!("{:?}", generated_graph);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayfieldPlugin)
        .insert_resource(Recipes(generated_graph))
        .add_systems(Startup, setup)
        .add_systems(Update, cursor_move)
        .run();
}
