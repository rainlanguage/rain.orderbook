use wasm_bindgen_utils::prelude::*;

#[wasm_bindgen(raw_module = "../js_api/index.js")]
extern "C" {
    async fn init_wasm();
}

#[wasm_bindgen(start)]
pub async fn compile_wasm_async() {
    init_wasm().await;
}
