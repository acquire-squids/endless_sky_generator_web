cfg_select! {
    all(target_family = "wasm", target_os = "unknown") => {
        const fn main() {}
    }
    _ => {
        fn main() -> std::process::ExitCode {
            use endless_sky_generator_web::generators::full_map;

            const FILE_NAME: &str = "full_map.zip";
            const OUTPUT_FOLDER: &str = "output";

            use std::{fs, path::PathBuf, process::ExitCode};

            let data_path = ["www", "es_stable_data"].iter().collect::<PathBuf>();
            let data_path = data_path.as_path();

            endless_sky_rw::read_path_and_ignore_if(data_path, |p| {
                p.starts_with(data_path.join("_deprecated"))
            })
            .map_or(
                ExitCode::FAILURE,
                |data_folder| match full_map::process_data(&data_folder) {
                    Ok(bytes) => {
                        match fs::create_dir_all(OUTPUT_FOLDER)
                            .and_then(|()| fs::write(PathBuf::from(OUTPUT_FOLDER).join(FILE_NAME), bytes))
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
                },
            )
        }
    }
}
