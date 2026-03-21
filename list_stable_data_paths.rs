use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    let data_path = ["..", "www", "es_stable_data"].iter().collect::<PathBuf>();

    let deprecated_path = PathBuf::from(data_path.as_path().join("_deprecated"));
    let deprecated_path = deprecated_path.as_path();

    let output = ["..", "www", "es_stable_data_paths.txt"]
        .iter()
        .collect::<PathBuf>();

    let mut paths_list = vec![];

    match read_source(data_path, &mut paths_list, &mut |p| {
        // I don't want deprecated data included in the generator's defaults
        p.starts_with(deprecated_path)
    }) {
        ReadResult::Ok => {
            paths_list.sort_unstable();

            let list_as_text = paths_list
                .into_iter()
                .fold(OsString::new(), |mut accum, path| {
                    accum.push(
                        path.components()
                            .skip(2)
                            .collect::<PathBuf>()
                            .as_mut_os_string(),
                    );

                    accum.push("\n");

                    accum
                });

            match fs::write(output, list_as_text.into_encoded_bytes()) {
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

fn read_source<F>(file_path: PathBuf, paths: &mut Vec<PathBuf>, ignore_if: &mut F) -> ReadResult
where
    F: FnMut(&Path) -> bool,
{
    if file_path.exists() {
        if ignore_if(file_path.as_path()) {
            ReadResult::Ok
        } else if file_path.is_dir() {
            let mut all_success = true;

            if let Ok(dir) = fs::read_dir(&file_path) {
                for entry in dir.flatten() {
                    let file_path = entry.path();

                    all_success &=
                        matches!(read_source(file_path, paths, ignore_if), ReadResult::Ok);
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
    } else {
        eprintln!("File \"{}\" does not exist", file_path.display());
        ReadResult::Err
    }
}
