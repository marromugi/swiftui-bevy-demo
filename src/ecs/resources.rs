use bevy_ecs::prelude::*;

/// 毎フレームのdelta time (秒)
#[derive(Resource, Default)]
pub struct DeltaTime(pub f32);
