use bevy::camera::visibility::RenderLayers;
use bevy::ecs::observer::ObservedBy;
use bevy::platform::hash::FixedState;
use bevy::render::Render;
use bevy::window::PrimaryWindow;
use bevy::{platform::collections::HashMap, prelude::*};
use float_ord::FloatOrd;
use std::hash::BuildHasher;

use crate::archipelago::{ConnectedMessage, ReceivedItemMessage, SendItemMessage};
use crate::assets::{ElementAtlas, UiAtlas};
use crate::game::cmd::{AddElementBackground, SpawnElement};
use crate::graph::{Element, Status};
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
                        remove_elements_dropped_in_drawer,
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

/// Source element that creates new copies rather than being moved
#[derive(Component)]
struct ElementSource;

#[derive(Component)]
struct ElementDrawer;

#[derive(Bundle)]
struct ElementBundle {
    pickable: Pickable,
    element: Element,
    transform: Transform,
    sprite: Sprite,
}

fn get_element_icon_idx(id: &Element, name_to_idx: &HashMap<String, usize>) -> usize {
    let key = FixedState::with_seed(42069).hash_one(id);
    key as usize % name_to_idx.len()
}

impl ElementBundle {
    fn build(element: &Element, pos: Vec2, atlas: &ElementAtlas) -> ElementBundle {
        let sprite = Sprite::from_atlas_image(
            atlas.1.clone(),
            TextureAtlas {
                layout: atlas.0.clone(),
                index: get_element_icon_idx(element, &atlas.2),
            },
        );
        ElementBundle {
            pickable: Pickable::default(),
            element: element.clone(),
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
pub struct RecipeGraph(pub Option<ElementGraph>);

impl RecipeGraph {
    /// Get the product resulting from the given ingredients, if it exists.
    /// Lookup is done for every order of ingredients.
    fn get_recipe(&self, el1: &Element, el2: &Element) -> Option<Vec<Element>> {
        self.0.as_ref().and_then(|eg| eg.get(el1, el2))
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
        pub id: Element,
        pub pos: Vec2,
        pub emit_dropped: bool,
    }

    impl Command for SpawnElement {
        fn apply(self, world: &mut World) {
            let atlas = world.get_resource::<ElementAtlas>().unwrap();
            let asset_server = world.get_resource::<AssetServer>().unwrap();
            let font = asset_server.load("fuzzybubbles-bold.ttf");
            let bundle = ElementBundle::build(&self.id, self.pos, atlas);

            let mut commands = world.commands();
            let entity = commands
                .spawn(bundle)
                .with_children(|parent| {
                    parent.spawn((
                        Transform {
                            translation: Vec3::new(0.0, -48.0, 0.0),
                            ..default()
                        },
                        Text2d::new(self.id.to_string()),
                        TextFont {
                            font,
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor::BLACK,
                    ));
                })
                .queue(AddElementBackground)
                .observe(element_drag)
                .observe(element_drag_end)
                .id();

            if self.emit_dropped {
                commands.write_message(ElementDropped(entity));
            }
        }
    }
}

// ================================================================================================
// Observers
// ================================================================================================

fn element_drag_end(
    drag_drop: On<Pointer<DragEnd>>,
    mut dropped_msg: MessageWriter<ElementDropped>,
    mut commands: Commands,
) {
    commands
        .entity(drag_drop.entity)
        .remove_recursive::<Children, RenderLayers>()
        .insert_recursive::<Children>(RenderLayers::layer(0));
    dropped_msg.write(ElementDropped(drag_drop.entity));
}

#[derive(Bundle)]
struct ElementDragAdder {
    sprite: Sprite,
    transform: Transform,
}

fn source_drag_start(
    drag_start: On<Pointer<DragStart>>,
    el_query: Query<&Element, With<ElementSource>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<UiPickingCamera>>,
    mut commands: Commands,
    el_atlas: Res<ElementAtlas>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fuzzybubbles-bold.ttf");

    let Ok(el) = el_query.get(drag_start.entity) else {
        return;
    };

    let (camera, camera_tf) = *camera_query;
    if let Ok(worldpos) =
        camera.viewport_to_world_2d(camera_tf, drag_start.pointer_location.position)
    {
        let new_entity = commands
            .spawn((Pickable::default(), Element(el.0.clone())))
            .observe(source_drag_start)
            .id();

        commands
            .entity(drag_start.entity)
            .move_components::<(ElementSource, Node, ImageNode, ObservedBy, ChildOf)>(new_entity)
            .insert(ElementDragAdder {
                sprite: Sprite::from_atlas_image(
                    el_atlas.1.clone(),
                    TextureAtlas {
                        layout: el_atlas.0.clone(),
                        index: get_element_icon_idx(el.0, &el_atlas.2),
                    },
                ),
                transform: Transform {
                    translation: worldpos.extend(Z_INDEX_DRAG),
                    ..default()
                },
            })
            .with_children(|parent| {
                parent.spawn((
                    Transform {
                        translation: Vec3::new(0.0, -48.0, 0.0),
                        ..default()
                    },
                    Text2d::new(get_element_display_name(el.0)),
                    TextFont {
                        font,
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor::BLACK,
                ));
            })
            .queue(AddElementBackground)
            .remove_recursive::<Children, RenderLayers>()
            .insert_recursive::<Children>(RenderLayers::layer(1))
            .observe(element_drag)
            .observe(element_drag_end);
    }
}

fn element_drag(
    drag: On<Pointer<Drag>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<UiPickingCamera>>,
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

#[allow(clippy::too_many_arguments)]
fn remove_elements_dropped_in_drawer(
    mut commands: Commands,
    mut dropped_msg: MessageReader<ElementDropped>,
    window: Single<&Window, With<PrimaryWindow>>,
    drawer: Single<(&ComputedNode, &UiGlobalTransform), With<ElementDrawer>>,
    camera: Single<(&Camera, &GlobalTransform), With<UiPickingCamera>>,
    assets_atlas: Res<Assets<TextureAtlasLayout>>,
    assets_image: Res<Assets<Image>>,
    element_query: Query<(&GlobalTransform, &Sprite), Without<ElementSource>>,
) {
    let (camera, camera_tf) = *camera;
    dropped_msg.read().for_each(|msg| {
        let Ok((dropped_tf, dropped_sprite)) = element_query.get(msg.0) else {
            return;
        };
        let dropped_bb =
            get_sprite_bounds(dropped_sprite, dropped_tf, &assets_image, &assets_atlas);
        let new_min = camera
            .world_to_viewport(camera_tf, dropped_bb.min.extend(0.0))
            .unwrap();
        let new_max = camera
            .world_to_viewport(camera_tf, dropped_bb.max.extend(0.0))
            .unwrap();

        let dropped_bb = Rect::from_corners(new_min, new_max);

        let drawer_bb = Rect::from_center_size(
            drawer.1.translation / window.scale_factor(),
            drawer.0.size() / window.scale_factor(),
        );

        if !dropped_bb.intersect(drawer_bb).is_empty() {
            commands.entity(msg.0).despawn();
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn merge_elements(
    mut commands: Commands,
    mut dropped_msg: MessageReader<ElementDropped>,
    mut write_send_item: MessageWriter<SendItemMessage>,
    mut write_received_item: MessageWriter<ReceivedItemMessage>,
    recipes: Res<RecipeGraph>,
    assets_atlas: Res<Assets<TextureAtlasLayout>>,
    assets_image: Res<Assets<Image>>,
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
                    .get_recipe(dropped_el, el)
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
                id: r_el.clone(),
                pos: new_pos,
                emit_dropped: false,
            });
            write_send_item.write(SendItemMessage {
                element: r_el.clone(),
            });
            write_received_item.write(ReceivedItemMessage { element: r_el });
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
            flex_direction: FlexDirection::Column,
            column_gap: px(10),
            flex_grow: 1.0,
            overflow: Overflow::scroll_y(),
            padding: UiRect::all(px(10)),
            margin: UiRect::all(px(10)),
            ..default()
        },
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
    let root = (
        Node {
            left: px(0),
            width: percent(25),
            height: percent(100),
            ..default()
        },
        ZIndex(-1),
    );

    commands.spawn((root, children![drawer]));
}

fn on_scroll_handler(
    mut scroll: On<Pointer<Scroll>>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let mut delta = Vec2::new(scroll.x, -scroll.y);
    if matches!(scroll.unit, bevy::input::mouse::MouseScrollUnit::Line) {
        delta *= 24.0;
    }

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
    let eg = recipe_graph.0.as_ref().unwrap();
    let bold_font = asset_server.load("fuzzybubbles-bold.ttf");

    for el in &eg.element_list {
        commands.entity(*drawer).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        height: px(0),
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                        Text::new(el.to_string()),
                        TextFont {
                            font: bold_font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                    parent
                        .spawn((
                            Node {
                                width: px(48),
                                height: px(48),
                                ..default()
                            },
                            el.to_owned(),
                            ElementSource,
                            Pickable::default(),
                            ImageNode::from_atlas_image(
                                el_atlas.1.clone(),
                                TextureAtlas {
                                    layout: el_atlas.0.clone(),
                                    index: get_element_icon_idx(el, &el_atlas.2),
                                },
                            ),
                        ))
                        .observe(source_drag_start);
                });
        });
    }
}

fn on_item_received(
    mut read_item_received: MessageReader<ReceivedItemMessage>,
    src_query: Query<(&Element, &ChildOf), With<ElementSource>>,
    mut vis_query: Query<&mut Visibility>,
    mut node_query: Query<&mut Node>,
) {
    read_item_received.read().for_each(|msg| {
        if let Some((_, parent)) = src_query.into_iter().find(|&(el, _)| *el == msg.element) {
            *vis_query.get_mut(parent.0).unwrap() = Visibility::Inherited;
            node_query.get_mut(parent.0).unwrap().height = auto();
        }
    });
}
