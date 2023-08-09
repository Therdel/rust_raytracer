use wasm_bindgen::prelude::*;

mod color;
mod gpu_renderer;
mod mesh_file_store;
mod renderer;
mod utils;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}

#[wasm_bindgen]
pub fn wasm_log_init() {
    // cannot be in init function, somehow
    console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
}