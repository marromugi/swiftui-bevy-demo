use glam::{Vec3, Quat};
use bevy_ecs::prelude::*;

/// このエンティティは描画対象のキューブだよ、っていうタグ
#[derive(Component)]
pub struct CubeMeshTag;

/// Y軸回転スピード（ラジアン/秒）
#[derive(Component)]
pub struct SpinY {
    pub speed: f32,
}

/// ワールド空間でのTRS
#[derive(Component)]
pub struct Transform3D {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
