use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Response, WorkerGlobalScope};

fn get_worker_global_scope() -> WorkerGlobalScope {
    let global = js_sys::global();
    global.dyn_into().unwrap()
}

#[wasm_bindgen]
extern "C" {
    pub fn fetch_into_array(resource: &str) -> Box<[u8]>;
    pub async fn fetch_scene() -> JsValue;
}

/// taken from the wasm_bindgen [documentation](https://rustwasm.github.io/docs/wasm-bindgen/examples/fetch.html)
pub async fn fetch(resource: &str) -> Result<String, JsValue> {
    let scope = get_worker_global_scope();
    let fetch_promise = scope.fetch_with_str(resource);
    let response = JsFuture::from(fetch_promise).await?;
    let response: Response = response.dyn_into()?;
    //response.

    todo!()
}
