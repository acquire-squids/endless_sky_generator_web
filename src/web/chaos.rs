use std::error::Error;

#[allow(clippy::missing_errors_doc)]
pub fn process_data(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: ChaosConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = crate::web::read_upload(paths, sources)?;

    crate::generators::chaos::process_data(&data_folder, &settings.into())
}

crate::macros::wasm_newtype! {
    using crate::generators::chaos::config;
    in config =>
    pub ChaosConfig as config::ChaosConfig ;
    seed: u32 => u64::from,
}
