pub use endless_sky_rw::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "www/export_to_rust.js")]
extern "C" {
    fn println(text: &str);
}

#[wasm_bindgen]
pub struct GenerateResult {
    #[wasm_bindgen(readonly, getter_with_clone)]
    pub text: Vec<String>,
    #[wasm_bindgen(readonly, getter_with_clone)]
    pub errors: String,
}

#[wasm_bindgen]
pub fn find_ships(paths: Vec<String>, sources: Vec<String>) -> GenerateResult {
    let Some((data_folder, errors)) = endless_sky_rw::read_upload(paths, sources) else {
        return GenerateResult {
            text: vec![],
            errors: "ERROR: Something went horribly wrong and you get no output :(".to_owned(),
        };
    };

    let data = data_folder.data();

    GenerateResult {
        text: endless_sky_rw::node_path_iter!(
            data; "ship"
        )
        .filter(|(_, node_index)| data.get_tokens(*node_index).unwrap_or_default().len() == 2)
        .map(|(source_index, node_index)| {
            data.get_lexeme(source_index, data.get_tokens(node_index).unwrap()[1])
                .unwrap_or_default()
                .to_owned()
        })
        .collect::<Vec<_>>(),
        errors: match String::from_utf8(errors) {
            Ok(errors) => errors,
            Err(_) => "ERROR: Invalid UTF-8 in error text :(".to_owned(),
        },
    }
}
