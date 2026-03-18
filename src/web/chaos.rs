use std::error::Error;

use wasm_bindgen::prelude::*;

#[allow(clippy::missing_errors_doc)]
pub fn process_data(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: ChaosConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = crate::web::read_upload(paths, sources)?;

    crate::generators::chaos::process_data(&data_folder, settings.into())
}

impl From<ChaosConfig> for crate::generators::chaos::ChaosConfig {
    fn from(value: ChaosConfig) -> Self {
        Self::new(value.seed)
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct ChaosConfig {
    seed: u32,
}

#[wasm_bindgen]
impl ChaosConfig {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }
}
