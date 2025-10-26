use bevy_ecs::{prelude::*, schedule::Schedule};
use glam::{Vec3, Quat};

use crate::{
    ecs:: {
        systems::spin_system_3d,
        resources::DeltaTime,
        components::{CubeMeshTag, SpinY, Transform3D}
    }
};

pub fn example_setup_world() -> (World, Schedule) {
    // World: ECSの実体
    let mut world = World::new();

    world.spawn((
        CubeMeshTag,
        SpinY { speed: 1.0 },
        Transform3D {
            translation: Vec3::new(0.0, 0.0, 2.0), // カメラ正面ちょい奥
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
        },
    ));

    world.insert_resource(DeltaTime::default());

    // Schedule: 毎フレ実行したいシステムを登録
    let mut schedule = Schedule::default();
    schedule.add_systems(spin_system_3d);

    (world, schedule)
}
