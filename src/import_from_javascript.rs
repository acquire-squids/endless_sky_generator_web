use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/www/export_to_rust.js")]
extern "C" {
    pub fn println(text: &str);
}
