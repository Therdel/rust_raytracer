use wasm_bindgen::prelude::*;

mod color;
mod fake_same_mesh_loader;
mod utils;
mod renderer;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}


pub use wasm_bindgen_rayon::init_thread_pool;
