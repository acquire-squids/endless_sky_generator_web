mod template;
mod wandom;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_template(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, String> {
    self::template::process(paths, sources).map_err(|e| e.to_string())
}
