use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// Rust --> JS glue adapted from [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/examples/import-js.html)
    pub type AssetStore;

    #[wasm_bindgen(method)]
    pub fn get_mesh(this: &AssetStore, name: &str) -> Vec<u8>;

    #[wasm_bindgen(method)]
    pub fn get_scene(this: &AssetStore, name: &str) -> Vec<u8>;

    #[wasm_bindgen(method)]
    // fn get_mesh_during_fn(this: &MeshStore, name: &str, closure: Closure) -> Vec<u8>;
    pub fn get_mesh_during_fn(this: &AssetStore, name: &str) -> Vec<u8>;
    // alternative: Rust mesh cache
        // js->rust_get_mesh_cache
            // if not js->rust_has_mesh(name)
                // js->rust_cache_mesh(&mesh)...
        // js->rust_change_scene
}