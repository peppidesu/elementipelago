//! Input run conditions (bindings)

use bevy::{
    ecs::system::Res,
    input::{ButtonInput, mouse::MouseButton},
};

pub fn primary_just_pressed(buttons: Res<ButtonInput<MouseButton>>) -> bool {
    buttons.just_pressed(MouseButton::Left)
}
pub fn primary_just_released(buttons: Res<ButtonInput<MouseButton>>) -> bool {
    buttons.just_released(MouseButton::Left)
}
