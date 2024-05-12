use wasm_bindgen::prelude::*;

mod color;
mod mesh_file_store;
mod renderer;
mod utils;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}
