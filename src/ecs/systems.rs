use bevy_ecs::prelude::*;
use glam::Quat;

use crate::{
    ecs::components::{SpinY, Transform3D},
    ecs::resources::DeltaTime
};

// 回転システム：SpinYを持ってるエンティティのTransform3Dを回す
pub fn spin_system_3d(mut q: Query<(&SpinY, &mut Transform3D)>, time: Res<DeltaTime>) {
    for (spin, mut tf) in &mut q {
        let delta_rot = Quat::from_rotation_y(spin.speed * time.0);
        tf.rotation = delta_rot * tf.rotation;
    }
}