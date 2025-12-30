use bevy::{platform::collections::HashMap, prelude::*};
use float_ord::FloatOrd;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::archipelago::{ConnectedMessage, ReceivedItemMessage, SendItemMessage};
use crate::assets::{ElementAtlas, UiAtlas};
use crate::game::cmd::SpawnElement;
use crate::graph::{Element as GElement, Status};
use crate::util::*;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ElementDropped>()
            .init_resource::<ElementAtlas>()
            .add_systems(Startup, setup_drawer)
            .add_systems(
                Update,
                (
                    (
                        merge_elements,
                        recalculate_element_z_order.run_if(any_message::<ElementDropped>),
                    )
                        .chain(),
                    (
                        populate_drawer.run_if(any_message::<ConnectedMessage>),
                        on_item_received,
                    )
                        .chain(),
                ),
            )
            .add_observer(on_scroll_handler);
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
#[derive(Component)]
struct Element(GElement);
/// Source element that creates new copies rather than being moved
#[derive(Component)]
struct ElementSource {
    unlocked: bool,
}

#[derive(Component)]
struct ElementDrawer;

#[derive(Bundle)]
struct ElementBundle {
    pickable: Pickable,
    element: Element,
    transform: Transform,
    sprite: Sprite,
}

impl ElementBundle {
    fn build(id: GElement, pos: Vec2, atlas: &ElementAtlas) -> ElementBundle {
        let element = Element(id);
        let mut rng = SmallRng::seed_from_u64(id.0);
        let sprite = Sprite::from_atlas_image(
            atlas.1.clone(),
            TextureAtlas {
                layout: atlas.0.clone(),
                index: rng.random_range(0..13),
            },
        );
        ElementBundle {
            pickable: Pickable::default(),
            element,
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, Z_INDEX_DRAG),
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
pub struct RecipeGraph(pub Option<(HashMap<(GElement, GElement), Vec<GElement>>, Vec<GElement>)>);

impl RecipeGraph {
    /// Get the product resulting from the given ingredients, if it exists.
    /// Lookup is done for every order of ingredients.
    fn get_recipe(&self, el1: GElement, el2: GElement) -> Option<Vec<GElement>> {
        self.0.as_ref().and_then(|(map, _)| {
            map.get(&(el1, el2))
                .or_else(|| map.get(&(el2, el1)))
                .cloned()
        })
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

mod cmd {
    use super::*;

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
        pub id: GElement,
        pub pos: Vec2,
        pub from_src_drop: bool,
    }

    impl Command for SpawnElement {
        fn apply(self, world: &mut World) {
            let atlas = world.get_resource::<ElementAtlas>().unwrap();
            let bundle = ElementBundle::build(self.id, self.pos, atlas);

            let mut commands = world.commands();
            let entity = commands
                .spawn(bundle)
                .queue(AddElementBackground)
                .observe(element_drag)
                .observe(element_drag_end)
                .id();

            if self.from_src_drop {
                commands.write_message(ElementDropped(entity));
            }
        }
    }
}

// ================================================================================================
// Run conditions
// ================================================================================================

// ================================================================================================
// Observers
// ================================================================================================

fn element_drag_end(
    drag_drop: On<Pointer<DragEnd>>,
    mut dropped_msg: MessageWriter<ElementDropped>,
) {
    dropped_msg.write(ElementDropped(drag_drop.entity));
}

fn source_drag_end(
    drag_end: On<Pointer<DragEnd>>,
    mut commands: Commands,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut el_query: Query<(&Element, &mut UiTransform), With<ElementSource>>,
) {
    let (camera, camera_tf) = *camera_query;
    let (el, mut tf) = el_query.get_mut(drag_end.entity).unwrap();

    tf.translation = Val2::ZERO;

    if let Ok(worldpos) = camera.viewport_to_world_2d(camera_tf, drag_end.pointer_location.position)
    {
        commands.queue(SpawnElement {
            id: el.0,
            pos: worldpos,
            from_src_drop: true,
        });
    }
}

fn element_drag(
    drag: On<Pointer<Drag>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut tf: Query<&mut Transform>,
) {
    let (camera, camera_tf) = *camera_query;
    let mut tf = tf.get_mut(drag.entity).unwrap();
    if let Ok(zero) = camera.viewport_to_world_2d(camera_tf, Vec2::ZERO)
        && let Ok(delta) = camera.viewport_to_world_2d(camera_tf, drag.delta)
    {
        tf.translation += (delta - zero).extend(0.0);
        tf.translation.z = Z_INDEX_DRAG;
    }
}

fn source_drag(drag: On<Pointer<Drag>>, mut tf: Query<&mut UiTransform>) {
    tf.get_mut(drag.entity).unwrap().translation = Val2::px(drag.distance.x, drag.distance.y);
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
    recipes: Res<RecipeGraph>,
    assets_atlas: Res<Assets<TextureAtlasLayout>>,
    assets_image: Res<Assets<Image>>,
    mut dropped_msg: MessageReader<ElementDropped>,
    mut write_send_item: MessageWriter<SendItemMessage>,
    element_query: Query<(Entity, &Element, &GlobalTransform, &Sprite), Without<ElementSource>>,
) {
    dropped_msg.read().for_each(|msg| {
        let Ok((dropped_root, dropped_el, dropped_tf, dropped_sprite)) = element_query.get(msg.0)
        else {
            // Entity has despawned
            return;
        };

        let dropped_bb =
            get_sprite_bounds(dropped_sprite, dropped_tf, &assets_image, &assets_atlas);
        let Some((other_root, other_tf, result_el)) = element_query
            .iter()
            .filter(|(e, _, _, _)| *e != msg.0)
            // Only elements that intersect
            .filter(|(_, _, tf, sprite)| {
                let other_bb = get_sprite_bounds(sprite, tf, &assets_image, &assets_atlas);
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
        for r_el in result_el {
            commands.queue(cmd::SpawnElement {
                id: r_el,
                pos: new_pos,
                from_src_drop: false,
            });
            write_send_item.write(SendItemMessage { element: r_el });
        }

        // despawn ingredient elements
        commands.entity(dropped_root).despawn();
        commands.entity(other_root).despawn();
    });
}

fn setup_drawer(mut commands: Commands, atlas: Res<UiAtlas>) {
    let drawer = (
        ElementDrawer,
        Node {
            width: percent(25),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            left: px(0),
            height: percent(100),
            ..default()
        },
        ZIndex(-1),
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
    );

    commands.spawn(drawer);
}

fn on_scroll_handler(
    mut scroll: On<Pointer<Scroll>>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };
    let mut delta = Vec2::new(scroll.x, -scroll.y) * 24.0;
    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if (delta.x == 0.0) && (delta.y == 0.0) {
        scroll.propagate(false);
    }
}

fn populate_drawer(
    mut commands: Commands,
    el_atlas: Res<ElementAtlas>,
    recipe_graph: Res<RecipeGraph>,
    asset_server: Res<AssetServer>,
    drawer: Single<Entity, With<ElementDrawer>>,
) {
    let (_, elements) = recipe_graph.0.as_ref().unwrap();
    let bold_font = asset_server.load("fuzzybubbles-bold.ttf");

    for el in elements {
        let el_source = commands
            .spawn((
                Node {
                    width: px(48),
                    height: px(48),
                    ..default()
                },
                Element(*el),
                ElementSource { unlocked: false },
                Pickable::default(),
                ImageNode::from_atlas_image(
                    el_atlas.1.clone(),
                    TextureAtlas {
                        layout: el_atlas.0.clone(),
                        index: 0,
                    },
                ),
            ))
            .observe(source_drag)
            .observe(source_drag_end)
            .id();

        let el_label = commands
            .spawn((
                Node { ..default() },
                Text::new(format!(
                    "{} #{}",
                    match el.1 {
                        Status::INPUT => "Element",
                        Status::INTERMEDIATE => "Intermediate",
                        Status::OUTPUT => "Compound",
                    },
                    el.0
                )),
                TextFont {
                    font: bold_font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ))
            .id();

        let item = commands
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    height: px(0),
                    ..default()
                },
                Visibility::Hidden,
            ))
            .add_children(&[el_label, el_source])
            .id();
        commands.entity(*drawer).add_child(item);
    }
}

fn on_item_received(
    mut read_item_received: MessageReader<ReceivedItemMessage>,
    src_query: Query<(&Element, &ChildOf), With<ElementSource>>,
    mut vis_query: Query<&mut Visibility>,
    mut node_query: Query<&mut Node>,
) {
    read_item_received.read().for_each(|msg| {
        if let Some((_, parent)) = src_query.iter().find(|(el, _)| el.0 == msg.element) {
            *vis_query.get_mut(parent.0).unwrap() = Visibility::Inherited;
            node_query.get_mut(parent.0).unwrap().height = auto();
        }
    });
}
