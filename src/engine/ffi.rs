use crate::engine::Engine;
use std::ffi::c_void;

/// Swift から呼ぶ用のブリッジ
#[no_mangle]
pub extern "C" fn engine_init(
    layer_ptr: *mut c_void,
    width: u32,
    height: u32,
) -> *mut Engine {
    let engine = unsafe { Engine::new(layer_ptr, width, height) };
    Box::into_raw(Box::new(engine))
}

#[no_mangle]
pub extern "C" fn engine_frame(engine: *mut Engine, dt_seconds: f32) {
    if engine.is_null() {
        return;
    }
    let eng = unsafe { &mut *engine };
    eng.frame(dt_seconds);
}

#[no_mangle]
pub extern "C" fn engine_free(engine: *mut Engine) {
    if engine.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(engine)); }
}
