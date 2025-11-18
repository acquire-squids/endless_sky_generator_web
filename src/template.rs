use endless_sky_rw::*;

use std::io::{self, Write};

use flate2::{Compression, write::DeflateEncoder};
use rawzip::{self, CompressionMethod, ZipArchiveWriter};
use wasm_bindgen::prelude::*;

const PLUGIN_NAME: &str = "Plugin Template";

const PLUGIN_DESCRIPTION: &str = "\
    Template plugin\
";

const PLUGIN_VERSION: &str = "0.1.0";

#[wasm_bindgen(module = "/www/export_to_rust.js")]
extern "C" {
    fn println(text: &str);
}

pub fn process(_paths: Vec<String>, _sources: Vec<String>) -> Result<Vec<u8>, rawzip::Error> {
    let mut data = Data::default();

    let mut output = vec![];

    let mut archive = ZipArchiveWriter::new(io::Cursor::new(&mut output));

    // `plugin.txt`:
    {
        let (mut entry, config) = archive
            .new_file("plugin.txt")
            .compression_method(CompressionMethod::Deflate)
            .start()?;

        let encoder = DeflateEncoder::new(&mut entry, Compression::default());

        let mut writer = config.wrap(encoder);

        let plugin_txt_source = data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        data.push_root_node(plugin_txt_source, plugin_name);

        let mut plugin_description = vec![];

        for about in PLUGIN_DESCRIPTION.lines() {
            let plugin_about = tree_from_tokens!(
                &mut data; plugin_txt_source =>
                : "about", about ;
            );

            plugin_description.push(plugin_about);

            data.push_root_node(plugin_txt_source, plugin_about);
        }

        let plugin_version = tree_from_tokens!(
            &mut data; plugin_txt_source =>
            : "version", PLUGIN_VERSION ;
        );

        data.push_root_node(plugin_txt_source, plugin_version);

        let mut plugin_txt = String::new();

        if data
            .write(&mut plugin_txt, plugin_txt_source, plugin_name, 0)
            .is_err()
        {
            return Err(rawzip::Error::from(rawzip::ErrorKind::IO(
                io::Error::other("Failed to write `plugin.txt` to string :("),
            )));
        }

        writer.write_all(plugin_txt.as_bytes())?;

        let (_, descriptor) = writer.finish()?;

        let _compressed_len = entry.finish(descriptor)?;
    }

    // `data/`
    archive.new_dir("data/").create()?;

    // `data/replace_with_plugin_data.txt`
    {
        let (mut entry, config) = archive
            .new_file("data/replace_with_plugin_data.txt")
            .compression_method(CompressionMethod::Deflate)
            .start()?;

        let encoder = DeflateEncoder::new(&mut entry, Compression::default());

        let mut writer = config.wrap(encoder);

        let example_txt_source = data.insert_source(String::new());

        let example_mission = tree_from_tokens!(
            &mut data; example_txt_source =>
            : "mission", format!("{PLUGIN_NAME}: forgot to remove example file") ;
            {
                : "name", "YOU FORGOT SOMETHING" ;
                : "description", "The example file was never removed from your template plugin.  Go back and remove it." ;
                : "non-blocking" ;
                : "landing" ;
                : "repeat" ;
                : "offer precedence", "1000000000" ;
                : "on", "offer" ;
                {
                    : "conversation" ;
                    {
                        : "As soon as you land, you realize something is wrong.";
                        : "    You forgot to remove the example file from your template plugin!" ;
                        : "    Everything goes dark, and you die." ;
                        {
                            : "die" ;
                        }
                    }
                }
            }
        );

        data.push_root_node(example_txt_source, example_mission);

        let mut example_txt = String::new();

        if data
            .write(&mut example_txt, example_txt_source, example_mission, 0)
            .is_err()
        {
            return Err(rawzip::Error::from(rawzip::ErrorKind::IO(
                io::Error::other(
                    "Failed to write `data/replace_with_plugin_data.txt` to string :(",
                ),
            )));
        }

        writer.write_all(example_txt.as_bytes())?;

        let (_, descriptor) = writer.finish()?;

        let _compressed_len = entry.finish(descriptor)?;
    }

    archive.finish()?;

    Ok(output)
}
