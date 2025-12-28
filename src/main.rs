use bevy::{platform::collections::HashMap, prelude::*};
use float_ord::FloatOrd;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::util::*;

mod graph;
mod input;
mod util;

const Z_INDEX_PLAYFIELD_MIN: f32 = -2.0;
const Z_INDEX_PLAYFIELD_MAX: f32 = -1.0;
const Z_INDEX_DRAG: f32 = 5.0;

/// Element with a numerical ID
#[derive(Component, Clone)]
struct Element(u64);
/// Source element that creates new copies rather than being moved
#[derive(Component, Clone)]
struct ElementSource;

/// Message indicating an element was just dropped
#[derive(Message)]
struct ElementDropped(Entity);

#[derive(Resource)]
struct ElementSpriteSheet(Handle<TextureAtlasLayout>, Handle<Image>);

impl FromWorld for ElementSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas = TextureAtlasLayout::from_grid(
            (48, 48).into(), // The size of each image
            1,               // The number of columns
            13,              // The number of rows
            None,            // Padding
            None,            // Offset
        );

        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        let image = world
            .get_resource::<AssetServer>()
            .unwrap()
            .load("atlas.png");

        Self(texture_atlas_handle, image)
    }
}

#[derive(Bundle)]
struct ElementBundle {
    pickable: Pickable,
    element: Element,
    transform: Transform,
    sprite: Sprite,
}

impl ElementBundle {
    fn build(id: u64, pos: Vec2, sprite_sheet: &ElementSpriteSheet) -> ElementBundle {
        let element = Element(id);
        let mut rng = SmallRng::seed_from_u64(id);
        let sprite = Sprite::from_atlas_image(
            sprite_sheet.1.clone(),
            TextureAtlas {
                layout: sprite_sheet.0.clone(),
                index: rng.random_range(0..13),
            },
        );
        ElementBundle {
            pickable: Pickable::default(),
            element,
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, Z_INDEX_DRAG),
                scale: Vec3::splat(2.0),
                ..default()
            },
            sprite,
        }
    }
}

pub struct AddElementBackground;
impl EntityCommand for AddElementBackground {
    fn apply(self, mut entity: EntityWorldMut) -> () {
        let asset_server = entity.get_resource::<AssetServer>().unwrap();
        let element_bg = asset_server.load("element-bg.png");

        entity.with_child((
            Sprite::from_image(element_bg),
            Transform {
                translation: Vec3::new(0.0, 0.0, -0.000001),
                ..default()
            },
            Pickable::default(),
        ));
    }
}

pub struct SpawnElement {
    id: u64,
    pos: Vec2,
}

impl Command for SpawnElement {
    fn apply(self, world: &mut World) -> () {
        let sprite_sheet = world.get_resource::<ElementSpriteSheet>().unwrap();
        let bundle = ElementBundle::build(self.id, self.pos, &sprite_sheet);
        world
            .commands()
            .spawn(bundle)
            .queue(AddElementBackground)
            .observe(element_drag)
            .observe(element_drag_end);
    }
}

pub struct SpawnElementSource {
    id: u64,
    pos: Vec2,
}

impl Command for SpawnElementSource {
    fn apply(self, world: &mut World) -> () {
        let sprite_sheet = world.get_resource::<ElementSpriteSheet>().unwrap();
        let bundle = ElementBundle::build(self.id, self.pos, &sprite_sheet);
        world
            .commands()
            .spawn((bundle, ElementSource))
            .queue(AddElementBackground)
            .observe(source_drag_start);
    }
}

pub struct GrabFromSource {
    entity: Entity,
}

impl Command for GrabFromSource {
    fn apply(self, world: &mut World) -> () {
        world
            .commands()
            .entity(self.entity)
            .clone_and_spawn()
            .queue(AddElementBackground)
            .observe(source_drag_start);

        world
            .commands()
            .entity(self.entity)
            .remove::<ElementSource>()
            .observe(element_drag)
            .observe(element_drag_end);
    }
}

/// Mapping of valid recipe ingredients to products
#[derive(Resource)]
struct Recipes(HashMap<(u64, u64), u64>);

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

fn element_drag_end(
    drag_drop: On<Pointer<DragEnd>>,
    mut dropped_msg: MessageWriter<ElementDropped>,
) {
    dropped_msg.write(ElementDropped(drag_drop.entity));
}

fn source_drag_start(
    drag_start: On<Pointer<DragStart>>,
    mut commands: Commands,
    src_query: Query<Entity, With<ElementSource>>,
) {
    let Ok(src_root) = src_query.get(drag_start.entity) else {
        return;
    };

    commands.queue(GrabFromSource { entity: src_root });
}

fn element_drag(
    drag: On<Pointer<Drag>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut tf: Query<&mut Transform>,
) {
    let (camera, camera_tf) = *camera_query;
    if let Ok(worldpos) =
        camera.viewport_to_world_2d(camera_tf, drag.pointer_location.position + drag.delta)
    {
        tf.get_mut(drag.entity).unwrap().translation = worldpos.extend(Z_INDEX_DRAG);
    }
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
        let Ok((dropped_root, dropped_el, dropped_tf, dropped_sprite)) = element_query.get(msg.0)
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
                println!("{:?}", other_bb);
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
        commands.queue(SpawnElement {
            id: result_el,
            pos: new_pos,
        });

        // despawn ingredient elements
        commands.entity(dropped_root).despawn();
        commands.entity(other_root).despawn();
    });
}

fn setup_playfield(mut commands: Commands) {
    let gap = 128.0;
    let count = 15;
    let cols = 3;
    let height = (count / cols) as f32 * gap;
    let offset_y = (height - gap) * 0.5;
    let offset_x = -((cols - 1) as f32 * gap) * 0.5;

    for i in 0..count {
        let x = (i % cols) as f32 * gap + offset_x;
        let y = offset_y - (i / cols) as f32 * gap;

        commands.queue(SpawnElementSource {
            id: i,
            pos: Vec2::new(x, y),
        });
    }
}

pub struct PlayfieldPlugin;
impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ElementDropped>()
            .add_systems(Startup, setup_playfield)
            .add_systems(
                Update,
                (
                    merge_elements,
                    recalculate_element_z_order.run_if(any_message::<ElementDropped>),
                )
                    .chain(),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
}

fn main() {
    let generated_graph = graph::create_graph(5, 5, 2827108, 5, 4).0;
    println!("{:?}", generated_graph);

    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PlayfieldPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .init_resource::<ElementSpriteSheet>()
        .insert_resource(Recipes(generated_graph))
        .add_systems(Startup, setup)
        .run();
}
