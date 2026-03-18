use std::error::Error;

#[allow(clippy::missing_errors_doc)]
pub fn process_data(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = crate::web::read_upload(paths, sources)?;

    crate::generators::full_map::process_data(&data_folder)
}
