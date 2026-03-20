use std::error::Error;

#[allow(clippy::missing_errors_doc)]
pub fn process_data(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: config::SystemShufflerConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = crate::web::read_upload(paths, sources)?;

    crate::generators::system_shuffler::process_data(&data_folder, settings.into())
}

pub mod config {
    crate::macros::wasm_newtype! {
        using crate::generators::system_shuffler::config;
        in main =>
        pub SystemShufflerConfig as config::SystemShufflerConfig;
        seed: u32 => u64::from,
        max_presets: u8,
        shuffle_chance: u8,
        fixed_shuffle_days: u8,
        shuffle_once_on_install: bool,
    }
}
