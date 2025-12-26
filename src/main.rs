use bevy::{platform::collections::HashMap, prelude::*, window::PrimaryWindow};
use float_ord::FloatOrd;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::util::*;

mod graph;
mod input;
mod util;

const Z_INDEX_PLAYFIELD_MIN: f32 = -2.0;
const Z_INDEX_PLAYFIELD_MAX: f32 = -1.0;
const Z_INDEX_CURSOR: f32 = 10.0;

/// Element with a numerical ID
#[derive(Component, Clone)]
struct Element(u64);

/// Source element that creates new copies rather than being moved
#[derive(Component, Clone)]
struct ElementSource;

/// Drawer containing source elements
#[derive(Component)]
#[require(Transform, InheritedVisibility)]
struct ElementDrawer;

/// Indicates the current element is being dragged by the cursor
#[derive(Component)]
struct BeingDragged;

/// Message indicating an element was just dropped
#[derive(Message)]
struct ElementDropped(Entity);

/// Cursor-tracking component
#[derive(Component)]
struct Cursor;

/// Mapping of valid recipe ingredients to products
#[derive(Resource)]
struct Recipes(HashMap<(u64, u64), u64>);

#[derive(Bundle)]

struct ElementBundle {
    element: Element,
    transform: Transform,
    sprite: Sprite,
}

impl ElementBundle {
    fn build(id: u64, pos: Vec2) -> ElementBundle {
        let mut rng = SmallRng::seed_from_u64(id);
        let dark = rng.random_bool(0.5);
        let color = if dark {
            Color::hsl(rng.random::<f32>() * 360.0, 1.0, 0.75)
        } else {
            Color::hsl(rng.random::<f32>() * 360.0, 1.0, 0.25)
        };
        ElementBundle {
            element: Element(id),
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, 1.0),
                scale: Vec3::new(64.0, 64.0, 1.0),
                ..default()
            },
            sprite: Sprite::from_color(color, Vec2::ONE),
        }
    }
}

impl Recipes {
    /// Get the product resulting from the given ingredients, if it exists.
    /// Lookup is done for every order of ingredients.
    fn get_recipe(&self, el1: u64, el2: u64) -> Option<u64> {
        self.0
            .get(&(el1, el2))
            .or_else(|| self.0.get(&(el2, el1)))
            .map(|e| *e)
    }
}

/// Run condition that triggers when any message of this type is received.
fn any_message<T: Message>(mut reader: MessageReader<T>) -> bool {
    reader.read().count() > 0
}

fn cursor_move(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut transform: Single<&mut Transform, With<Cursor>>,
) {
    // Camera is required for screen-to-world conversion
    let (camera, camera_transform) = *camera_query;

    if let Some(position) = window.cursor_position()
        && let Ok(worldpos) = camera.viewport_to_world_2d(camera_transform, position)
    {
        transform.translation = worldpos.extend(Z_INDEX_CURSOR);
    }
}

fn drag_end(
    mut commands: Commands,
    mut dropped_msg: MessageWriter<ElementDropped>,
    drag_root: If<Single<Entity, With<BeingDragged>>>,
) {
    commands
        .entity(**drag_root)
        .remove::<BeingDragged>()
        .remove_parent_in_place();

    dropped_msg.write(ElementDropped(**drag_root));
}

fn drag_begin(
    mut commands: Commands,
    assets: Res<Assets<Image>>,
    cursor_query: Single<(Entity, &Transform), With<Cursor>>,
    mut element_query: Query<
        (Entity, &GlobalTransform, &Sprite, Option<&ElementSource>),
        (With<Element>, Without<Cursor>),
    >,
) {
    let (cursor, cursor_transform) = *cursor_query;

    let Some((element_root, _, _, element_is_src)) = element_query
        .iter_mut()
        // Only elements under cursor
        .filter(|(_, tf, sprite, _)| {
            let bounds = get_sprite_bounds(sprite, tf, &assets);
            bounds.contains(cursor_transform.translation.xy())
        })
        // Element with highest z index (top-most)
        .max_by_key(|(_, tf, _, _)| FloatOrd(tf.translation().z))
    else {
        // No candidate elements
        return;
    };

    if element_is_src.is_some() {
        commands.entity(element_root).clone_and_spawn();
        commands.entity(element_root).remove::<ElementSource>();
    }

    commands
        .entity(element_root)
        .set_parent_in_place(cursor)
        .insert(BeingDragged);
}

fn recalculate_element_z_order(
    mut element_query: Query<&mut Transform, (With<Element>, Without<ElementSource>)>,
) {
    let count = element_query.count();

    element_query
        .iter_mut()
        // Sort by z index
        .sort_by::<&Transform>(|tf1, tf2| tf1.translation.z.total_cmp(&tf2.translation.z))
        .enumerate()
        .for_each(|(i, mut tf)| {
            // Calculate new z index
            tf.translation.z = f32::lerp(
                Z_INDEX_PLAYFIELD_MIN,
                Z_INDEX_PLAYFIELD_MAX,
                i as f32 / count as f32,
            );
        });
}

fn merge_elements(
    mut commands: Commands,
    recipes: Res<Recipes>,
    assets: Res<Assets<Image>>,
    mut dropped_msg: MessageReader<ElementDropped>,
    element_query: Query<(Entity, &Element, &GlobalTransform, &Sprite), Without<ElementSource>>,
) {
    dropped_msg.read().for_each(|msg| {
        let Some((dropped_root, dropped_el, dropped_tf, dropped_sprite)) =
            element_query.iter().find(|(root, ..)| *root == msg.0)
        else {
            // Entity has despawned
            return;
        };

        let dropped_bb = get_sprite_bounds(dropped_sprite, dropped_tf, &assets);

        let Some((other_root, other_tf, result_el)) = element_query
            .iter()
            .filter(|(e, _, _, _)| *e != msg.0)
            // Only elements that intersect
            .filter(|(_, _, tf, sprite)| {
                let other_bb = get_sprite_bounds(sprite, tf, &assets);
                let isect = dropped_bb.intersect(other_bb);
                !isect.is_empty()
            })
            // Only elements that can merge with this one
            .filter_map(|(r, el, tf, _)| {
                recipes
                    .get_recipe(dropped_el.0, el.0)
                    .map(|result| (r, tf, result))
            })
            // Element with highest z-order (top-most)
            .max_by_key(|(_, tf, _)| FloatOrd(tf.translation().z))
        else {
            // None -> no candidate element found
            return;
        };

        // New position halfway between others
        let new_pos = dropped_tf
            .translation()
            .xy()
            .interpolate_stable(&other_tf.translation().xy(), 0.5);

        // spawn product element
        commands.spawn(ElementBundle::build(result_el, new_pos));

        // despawn ingredient elements
        commands.entity(dropped_root).despawn();
        commands.entity(other_root).despawn();
    });
}

fn bring_dragged_to_top(
    mut tf: If<Single<(&mut Transform, &GlobalTransform), Added<BeingDragged>>>,
) {
    (*tf).0.translation.z = 0.0;
}

pub struct PlayfieldPlugin;
impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ElementDropped>().add_systems(
            Update,
            (
                (
                    drag_begin.run_if(input::primary_just_pressed),
                    bring_dragged_to_top,
                )
                    .chain(),
                (
                    drag_end.run_if(input::primary_just_released),
                    merge_elements,
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
            ..default()
        },
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

        drawer.with_child((ElementBundle::build(i, Vec2::new(x, y)), ElementSource));
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
