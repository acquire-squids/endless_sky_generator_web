#[cfg(all(target_family = "wasm", target_os = "unknown"))]
const fn main() {}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
fn main() -> std::process::ExitCode {
    use endless_sky_generator_web::{
        config::{self, Value},
        generators::chaos::{self, config::ChaosConfig},
    };

    const FILE_NAME: &str = "chaos.zip";
    const OUTPUT_FOLDER: &str = "output";

    use std::{env, fs, path::PathBuf, process::ExitCode};

    let mut arguments = env::args();
    arguments.next();

    if let Some(file_path) = arguments.next() {
        let file_path = file_path.as_str();
        let path = PathBuf::from(file_path);

        if !path.exists() {
            eprintln!("Config file \"{file_path}\" does not exist!");
            ExitCode::FAILURE
        } else if !path.is_file() {
            eprintln!("Config file \"{file_path}\" is not a file!");
            ExitCode::FAILURE
        } else {
            match fs::read_to_string(path) {
                Ok(source) => {
                    let Some(settings) = config::parse_config!(
                        source.as_str() => ChaosConfig;
                        seed => { int of u64 => seed }
                    ) else {
                        return ExitCode::FAILURE;
                    };

                    endless_sky_rw::read_path("./www/es_stable_data/").map_or(
                        ExitCode::FAILURE,
                        |data_folder| match chaos::process_data(&data_folder, &settings) {
                            Ok(bytes) => {
                                match fs::create_dir_all(OUTPUT_FOLDER).and_then(|()| {
                                    fs::write(format!("{OUTPUT_FOLDER}/{FILE_NAME}"), bytes)
                                }) {
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
                        },
                    )
                }
                Err(error) => {
                    eprintln!("{error}");
                    eprintln!("Failed to read config \"{file_path}\"!");
                    ExitCode::FAILURE
                }
            }
        }
    } else {
        eprintln!("Expected the path to the config!");
        ExitCode::FAILURE
    }
}
