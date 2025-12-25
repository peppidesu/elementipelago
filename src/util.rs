use bevy::{
    math::{Vec2, Vec3Swizzles, bounding::Aabb2d},
    transform::components::Transform,
};

pub trait IntoAabb2dExt {
    fn into_aabb2d(self) -> Aabb2d;
}

impl IntoAabb2dExt for Vec2 {
    fn into_aabb2d(self) -> Aabb2d {
        Aabb2d::new(self, Vec2::ZERO)
    }
}

impl IntoAabb2dExt for Transform {
    fn into_aabb2d(self) -> Aabb2d {
        Aabb2d::new(self.translation.xy(), self.scale.xy() * 0.5)
    }
}
