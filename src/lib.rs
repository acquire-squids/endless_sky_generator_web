mod full_map;
mod import_from_javascript;
mod system_shuffler;
mod wandom;
mod zippy;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_full_map(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, String> {
    self::full_map::process(paths, sources).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn generate_system_shuffler(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: self::system_shuffler::SystemShufflerConfig,
) -> Result<Vec<u8>, String> {
    self::system_shuffler::process(paths, sources, settings).map_err(|e| e.to_string())
}
