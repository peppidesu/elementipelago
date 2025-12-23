use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseButtonInput, MouseMotion},
    math::{
        VectorSpace,
        bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    },
    prelude::*,
    window::PrimaryWindow,
};
#[derive(Component)]
struct Element(u32);

#[derive(Component)]
struct ElementSource;

#[derive(Component)]
struct ElementDrawer {
    page: u32,
}

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct BeingDragged;

#[derive(Event)]
struct ElementDropped;

mod input {
    use super::*;
    pub fn primary_start(buttons: Res<ButtonInput<MouseButton>>) -> bool {
        buttons.just_pressed(MouseButton::Left) && buttons.pressed(MouseButton::Left)
    }
    pub fn primary_end(buttons: Res<ButtonInput<MouseButton>>) -> bool {
        buttons.just_released(MouseButton::Left) && !buttons.pressed(MouseButton::Left)
    }
}

fn cursor_move(
    window: Single<&Window, With<PrimaryWindow>>,
    mut transform: Single<&mut Transform, With<Cursor>>,
) {
    if let Some(position) = window.cursor_position() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn drag_end(
    mut commands: Commands,
    mut drag_query: If<Single<(Entity, &mut Transform), With<BeingDragged>>>,
) {
    commands
        .entity((*drag_query).0)
        .remove::<BeingDragged>()
        .remove_parent_in_place();

    (*drag_query).1.translation.z = 0.0;
    commands.trigger(ElementDropped);
}

fn drag_begin(
    mut commands: Commands,
    cursor_query: Single<(Entity, &Transform), With<Cursor>>,
    mut element_query: Query<(Entity, &mut Transform), With<Element>>,
) {
    let (cursor, cursor_transform) = *cursor_query;

    let Some((drag_candidate, mut drag_candidate_tf)) = element_query
        .iter_mut()
        .filter(|(_, tf)| {
            let aabb = Aabb2d::new(tf.translation.xy(), tf.scale.xy() / 2.0);
            let cursor_aabb = Aabb2d::new(cursor_transform.translation.xy(), Vec2::ZERO);

            aabb.contains(&cursor_aabb)
        })
        .min_by(|(_, tf1), (_, tf2)| tf1.translation.z.total_cmp(&tf2.translation.z))
    else {
        return;
    };

    commands.entity(drag_candidate).set_parent_in_place(cursor);

    drag_candidate_tf.translation.z = -1.0;
}

fn recalculate_element_z_order(
    _event: On<ElementDropped>,
    mut element_query: Query<&mut Transform, With<Element>>,
) {
    let count = element_query.count();
    element_query
        .iter_mut()
        .sort_by::<&Transform>(|tf1, tf2| tf1.translation.z.total_cmp(&tf2.translation.z))
        .enumerate()
        .for_each(|(i, mut tf)| tf.translation.z = -2.0 + ((i as f32) / (count as f32)));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, cursor_move)
        .add_systems(Update, drag_begin.run_if(input::primary_start))
        .add_systems(Update, drag_end.run_if(input::primary_end))
        .add_observer(recalculate_element_z_order)
        .run();
}
