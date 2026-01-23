mod generators;
mod import_from_javascript;
mod wandom;
mod zippy;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(clippy::missing_errors_doc)]
pub fn generate_full_map(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, String> {
    self::generators::full_map::process(paths, sources).map_err(|e| e.to_string())
}

#[wasm_bindgen]
#[allow(clippy::missing_errors_doc)]
pub fn generate_system_shuffler(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: self::generators::system_shuffler::SystemShufflerConfig,
) -> Result<Vec<u8>, String> {
    self::generators::system_shuffler::process(paths, sources, settings).map_err(|e| e.to_string())
}
