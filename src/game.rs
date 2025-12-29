use bevy::{platform::collections::HashMap, prelude::*};
use float_ord::FloatOrd;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::graph::{Element as GElement, Status};
use crate::util::*;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ElementDropped>()
            .init_resource::<ElementSpriteSheet>()
            .insert_resource(Recipes(None))
            .add_systems(Startup, setup)
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

// ================================================================================================
// Constants
// ================================================================================================

const Z_INDEX_PLAYFIELD_MIN: f32 = -2.0;
const Z_INDEX_PLAYFIELD_MAX: f32 = -1.0;
const Z_INDEX_BG_OFFSET: f32 = -0.00001;
const Z_INDEX_DRAG: f32 = 5.0;

// ================================================================================================
// Components
// ================================================================================================

/// Element with a numerical ID
#[derive(Component, Clone)]
struct Element(GElement);
/// Source element that creates new copies rather than being moved
#[derive(Component, Clone)]
struct ElementSource;

#[derive(Bundle)]
struct ElementBundle {
    pickable: Pickable,
    element: Element,
    transform: Transform,
    sprite: Sprite,
}

impl ElementBundle {
    fn build(id: GElement, pos: Vec2, sprite_sheet: &ElementSpriteSheet) -> ElementBundle {
        let element = Element(id);
        let mut rng = SmallRng::seed_from_u64(id.0);
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

// ================================================================================================
// Resources
// ================================================================================================

/// Mapping of valid recipe ingredients to products
#[derive(Resource)]
pub struct Recipes(pub Option<HashMap<(GElement, GElement), Vec<GElement>>>);

impl Recipes {
    /// Get the product resulting from the given ingredients, if it exists.
    /// Lookup is done for every order of ingredients.
    fn get_recipe(&self, el1: GElement, el2: GElement) -> Option<Vec<GElement>> {
        self.0.as_ref().and_then(|map| {
            map.get(&(el1, el2))
                .or_else(|| map.get(&(el2, el1)))
                .cloned()
        })
    }
}

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

// ================================================================================================
// Messages
// ================================================================================================

/// Message indicating an element was just dropped
#[derive(Message)]
struct ElementDropped(Entity);

// ================================================================================================
// Custom commands
// ================================================================================================

pub struct AddElementBackground;
impl EntityCommand for AddElementBackground {
    fn apply(self, mut entity: EntityWorldMut) {
        let asset_server = entity.get_resource::<AssetServer>().unwrap();
        let element_bg = asset_server.load("element-bg.png");

        entity.with_child((
            Sprite::from_image(element_bg),
            Transform {
                translation: Vec2::ZERO.extend(Z_INDEX_BG_OFFSET),
                ..default()
            },
            Pickable::default(),
        ));
    }
}

pub struct SpawnElement {
    id: GElement,
    pos: Vec2,
}

impl Command for SpawnElement {
    fn apply(self, world: &mut World) {
        let sprite_sheet = world.get_resource::<ElementSpriteSheet>().unwrap();
        let bundle = ElementBundle::build(self.id, self.pos, sprite_sheet);
        world
            .commands()
            .spawn(bundle)
            .queue(AddElementBackground)
            .observe(element_drag)
            .observe(element_drag_end);
    }
}

pub struct SpawnElementSource {
    id: GElement,
    pos: Vec2,
}

impl Command for SpawnElementSource {
    fn apply(self, world: &mut World) {
        let sprite_sheet = world.get_resource::<ElementSpriteSheet>().unwrap();
        let bundle = ElementBundle::build(self.id, self.pos, sprite_sheet);
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
    fn apply(self, world: &mut World) {
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

// ================================================================================================
// Run conditions
// ================================================================================================

/// Run condition that triggers when any message of this type is received.
fn any_message<T: Message>(mut reader: MessageReader<T>) -> bool {
    reader.read().count() > 0
}

// ================================================================================================
// Observers
// ================================================================================================

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

// ================================================================================================
// Systems
// ================================================================================================

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
        for r_el in result_el {
            commands.queue(SpawnElement {
                id: r_el,
                pos: new_pos,
            });
        }

        // despawn ingredient elements
        commands.entity(dropped_root).despawn();
        commands.entity(other_root).despawn();
    });
}

fn setup(mut commands: Commands) {
    let gap = 144.0;
    let count = 15;
    let cols = 3;
    let height = (count / cols) as f32 * gap;
    let offset_y = (height - gap) * 0.5;
    let offset_x = -((cols - 1) as f32 * gap) * 0.5;

    for i in 0..count {
        let x = (i % cols) as f32 * gap + offset_x;
        let y = offset_y - (i / cols) as f32 * gap;

        commands.queue(SpawnElementSource {
            id: (i, Status::INPUT),
            pos: Vec2::new(x, y),
        });
    }
}
