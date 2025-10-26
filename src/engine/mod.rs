use std::ffi::c_void;

use bevy_ecs::{
    prelude::*,
    schedule::{Schedule},
};

use crate::{
    ecs::{example_setup_world, DeltaTime}, 
    graphics::Graphics
};


/// Engine = ゲームループをまとめたもの
pub struct Engine {
    world: World,
    schedule: Schedule,
    graphics: Graphics,
}

impl Engine {
    pub unsafe fn new(layer_ptr: *mut c_void, width: u32, height: u32) -> Self {
        let (world, schedule) = example_setup_world();
        let graphics = Graphics::new_from_layer(layer_ptr, width, height);
        Self { world, schedule, graphics }
    }

    pub fn frame(&mut self, dt_seconds: f32) {
        // 1. DeltaTime 更新
        {
            let mut dt = self.world.resource_mut::<DeltaTime>();
            dt.0 = dt_seconds;
        }

        // 2. ECSシステム実行（回転とか）
        self.schedule.run(&mut self.world);

        // 3. レンダリング（Worldの状態を読んで描く）
        self.graphics.draw_world(&mut self.world);
    }
}

pub mod ffi;