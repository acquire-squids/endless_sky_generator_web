use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/www/export_to_rust.js")]
extern "C" {
    pub fn println(text: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(text: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(text: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(text: &str);
}
