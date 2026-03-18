use std::error::Error;

use wasm_bindgen::prelude::*;

#[allow(clippy::missing_errors_doc)]
pub fn process_data(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: SystemShufflerConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = crate::web::read_upload(paths, sources)?;

    crate::generators::system_shuffler::process_data(&data_folder, settings.into())
}

impl From<SystemShufflerConfig> for crate::generators::system_shuffler::SystemShufflerConfig {
    fn from(value: SystemShufflerConfig) -> Self {
        Self::new(
            value.seed,
            value.max_presets,
            value.shuffle_chance,
            value.fixed_shuffle_days,
            value.shuffle_once_on_install,
        )
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct SystemShufflerConfig {
    seed: u32,
    max_presets: u8,
    shuffle_chance: u8,
    fixed_shuffle_days: u8,
    shuffle_once_on_install: bool,
}

#[wasm_bindgen]
impl SystemShufflerConfig {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new(
        seed: u32,
        max_presets: u8,
        shuffle_chance: u8,
        fixed_shuffle_days: u8,
        shuffle_once_on_install: bool,
    ) -> Self {
        Self {
            seed,
            max_presets,
            shuffle_chance,
            fixed_shuffle_days,
            shuffle_once_on_install,
        }
    }
}
