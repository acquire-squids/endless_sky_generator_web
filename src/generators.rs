pub mod chaos;
pub mod full_map;
pub mod system_shuffler;

use crate::import_from_javascript;
use crate::zippy::Zip;
use endless_sky_rw::{self, Data, DataFolder, NodeIndex, SourceIndex};

use std::{error::Error, io, path::PathBuf};

fn zip_root_nodes<P: Into<PathBuf>>(
    archive: &mut Zip,
    path: P,
    data: &Data,
    root_nodes: &[(SourceIndex, NodeIndex)],
) -> Result<(), Box<dyn Error>> {
    let path = P::into(path);

    let mut text = String::new();

    if data.write_root_nodes(&mut text, root_nodes).is_err() {
        return Err(Box::new(io::Error::other(format!(
            "Failed to write `{}` to string :(",
            path.display()
        ))));
    }

    archive.write_file(path, text.trim().as_bytes())?;

    Ok(())
}

fn read_upload(paths: Vec<String>, sources: Vec<String>) -> Result<DataFolder, Box<dyn Error>> {
    match endless_sky_rw::read_upload(paths, sources) {
        Some((data_folder, errors)) => {
            if !errors.is_empty() {
                let error_string = String::from_utf8(errors)?;

                import_from_javascript::error(error_string.as_str());
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
