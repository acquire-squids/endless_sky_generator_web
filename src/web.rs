use endless_sky_rw::DataFolder;

use std::{error::Error, io};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(clippy::missing_errors_doc)]
pub fn generate_full_map(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, String> {
    read_upload(paths, sources)
        .and_then(|data_folder| crate::generators::full_map::process_data(&data_folder))
        .map_err(|error| error.to_string())
}

#[wasm_bindgen]
#[allow(clippy::missing_errors_doc)]
pub fn generate_system_shuffler(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: crate::generators::system_shuffler::config::SystemShufflerConfig,
) -> Result<Vec<u8>, String> {
    read_upload(paths, sources)
        .and_then(|data_folder| {
            crate::generators::system_shuffler::process_data(&data_folder, settings)
        })
        .map_err(|error| error.to_string())
}

#[wasm_bindgen]
#[allow(clippy::missing_errors_doc)]
pub fn generate_chaos(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: &crate::generators::chaos::config::ChaosConfig,
) -> Result<Vec<u8>, String> {
    read_upload(paths, sources)
        .and_then(|data_folder| crate::generators::chaos::process_data(&data_folder, settings))
        .map_err(|error| error.to_string())
}

fn read_upload(paths: Vec<String>, sources: Vec<String>) -> Result<DataFolder, Box<dyn Error>> {
    match endless_sky_rw::read_upload(paths, sources) {
        Some((data_folder, errors)) => {
            if !errors.is_empty() {
                let error_string = String::from_utf8(errors)?;

                self::import_from_javascript::error(error_string.as_str());
            }

            Ok(data_folder)
        }
        None => {
            Err(Box::new(
                io::Error::other(
                    "ERROR: Somehow, everything went wrong while reading the data folder. You're on your own.".to_owned()
                )
            ))
        }
    }
}

mod import_from_javascript {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    unsafe extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(text: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn warn(text: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn error(text: &str);
    }
}
