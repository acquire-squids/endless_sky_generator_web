use std::{fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let input_path = PathBuf::from("endless-sky/data/");
    let mut paths_list = vec![];

    match read_source(input_path, &mut paths_list) {
        ReadResult::Ok => {
            paths_list.sort_unstable();

            let list_as_text = paths_list
                .into_iter()
                .filter(|path| {
                    // I don't want deprecated data included in the generator's defaults
                    !path
                        .display()
                        .to_string()
                        .starts_with("endless-sky/data/_deprecated")
                })
                .fold(String::new(), |mut accum, path| {
                    // I know, we have perfectly fine `PathBuf`s, we shouldn't be using strings like this
                    accum.push_str(
                        path.display()
                            .to_string()
                            .replacen("endless-sky/data/", "es_stable_data/", 1)
                            .as_str(),
                    );
                    accum.push('\n');
                    accum
                });

            match fs::write("www/es_stable_data_paths.txt", list_as_text) {
                Ok(_) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        ReadResult::Err => ExitCode::FAILURE,
    }
}

const EXTENSION: &str = "txt";

enum ReadResult {
    Ok,
    Err,
}

fn read_source(file_path: PathBuf, paths: &mut Vec<PathBuf>) -> ReadResult {
    if !file_path.exists() {
        eprintln!("File \"{}\" does not exist", file_path.display());
        ReadResult::Err
    } else if file_path.is_dir() {
        let mut all_success = true;

        if let Ok(dir) = fs::read_dir(&file_path) {
            for entry in dir.flatten() {
                let file_path = entry.path();

                all_success &= matches!(read_source(file_path, paths), ReadResult::Ok);
            }

            if all_success {
                ReadResult::Ok
            } else {
                ReadResult::Err
            }
        } else {
            eprintln!("Failed to read directory \"{}\"", file_path.display());
            ReadResult::Err
        }
    } else if file_path.is_file() {
        if matches!(file_path.extension(), Some(ext) if matches!(ext.to_str(), Some(ext) if ext == EXTENSION))
        {
            paths.push(file_path);
        }

        ReadResult::Ok
    } else {
        eprintln!(
            "Path \"{}\" was not a file or a directory",
            file_path.display()
        );

        ReadResult::Err
    }
}
