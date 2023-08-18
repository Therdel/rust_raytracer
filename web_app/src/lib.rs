use wasm_bindgen::prelude::*;

mod asset_store;
mod color;
mod mesh_fetcher;
mod utils;
mod renderer;
mod gpu_renderer;

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