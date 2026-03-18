use endless_sky_generator_web::generators::full_map;

const FILE_NAME: &str = "full_map.zip";
const OUTPUT_FOLDER: &str = "output";

use std::{fs, process::ExitCode};

fn main() -> ExitCode {
    endless_sky_rw::read_path("./www/es_stable_data/").map_or(ExitCode::FAILURE, |data_folder| {
        match full_map::process_data(&data_folder) {
            Ok(bytes) => {
                match fs::create_dir_all(OUTPUT_FOLDER)
                    .and_then(|()| fs::write(format!("{OUTPUT_FOLDER}/{FILE_NAME}"), bytes))
                {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(error) => {
                        eprintln!("{error}");
                        ExitCode::FAILURE
                    }
                }
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        }
    })
}
