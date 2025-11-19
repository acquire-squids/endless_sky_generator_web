use endless_sky_rw::*;

use std::io::{self, Write};

use flate2::{Compression, write::DeflateEncoder};
use rawzip::{self, CompressionMethod, ZipArchiveWriter};
use wasm_bindgen::prelude::*;

const PLUGIN_NAME: &str = "Full Map";

const PLUGIN_DESCRIPTION: &str = "\
    Reveal the entire map via any job board\
";

const PLUGIN_VERSION: &str = "0.1.0";

type ZipBytes<'a> = ZipArchiveWriter<io::Cursor<&'a mut Vec<u8>>>;

fn write_file_to_zip<'a>(
    archive: &mut ZipBytes<'a>,
    path: &str,
    bytes: &[u8],
) -> Result<(), rawzip::Error> {
    let (mut entry, config) = archive
        .new_file(path)
        .compression_method(CompressionMethod::Deflate)
        .start()?;

    let encoder = DeflateEncoder::new(&mut entry, Compression::default());

    let mut writer = config.wrap(encoder);

    writer.write_all(bytes)?;

    let (_, descriptor) = writer.finish()?;

    let _compressed_len = entry.finish(descriptor)?;

    Ok(())
}

fn write_dir_to_zip<'a>(archive: &mut ZipBytes<'a>, path: &str) -> Result<(), rawzip::Error> {
    archive.new_dir(path).create()?;
    Ok(())
}

#[wasm_bindgen(module = "/www/export_to_rust.js")]
extern "C" {
    fn println(text: &str);
}

fn find_named_objects<'a>(
    data: &'a Data,
    source_index: SourceIndex,
    node_index: NodeIndex,
    names: &mut Vec<&'a str>,
) {
    for object in node_path_iter!(data => (source_index, node_index); "object") {
        if let Some(tokens) = data.get_tokens(object)
            && tokens.len() >= 2
            && let Some(name) = data.get_lexeme(source_index, tokens[1])
        {
            names.push(name);
        }

        find_named_objects(data, source_index, object, names);
    }
}

pub fn process(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, rawzip::Error> {
    let data_folder = match read_upload(paths, sources) {
        Some((data_folder, errors)) => {
            if !errors.is_empty() {
                let error_string = String::from_utf8(errors).map_err(|utf8_error| {
                    rawzip::Error::from(rawzip::ErrorKind::InvalidUtf8(utf8_error.utf8_error()))
                })?;

                println(error_string.as_str());
            }

            data_folder
        }
        None => {
            return Err(rawzip::Error::from(rawzip::ErrorKind::InvalidInput { msg: "ERROR: Somehow, everything went wrong while reading the data folder.  You're on your own.".to_owned() }));
        }
    };

    let data = data_folder.data();

    let mut output_data = Data::default();

    let mut output = vec![];

    let mut archive = ZipArchiveWriter::new(io::Cursor::new(&mut output));

    // `plugin.txt`:
    {
        let plugin_txt_source = output_data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut output_data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        output_data.push_root_node(plugin_txt_source, plugin_name);

        let mut plugin_description = vec![];

        for about in PLUGIN_DESCRIPTION.lines() {
            let plugin_about = tree_from_tokens!(
                &mut output_data; plugin_txt_source =>
                : "about", about ;
            );

            plugin_description.push(plugin_about);

            output_data.push_root_node(plugin_txt_source, plugin_about);
        }

        let plugin_version = tree_from_tokens!(
            &mut output_data; plugin_txt_source =>
            : "version", PLUGIN_VERSION ;
        );

        output_data.push_root_node(plugin_txt_source, plugin_version);

        let mut plugin_txt = String::new();

        if output_data
            .write(&mut plugin_txt, plugin_txt_source, plugin_name, 0)
            .is_err()
            || plugin_description.into_iter().any(|n| {
                output_data
                    .write(&mut plugin_txt, plugin_txt_source, n, 0)
                    .is_err()
            })
            || output_data
                .write(&mut plugin_txt, plugin_txt_source, plugin_version, 0)
                .is_err()
        {
            return Err(rawzip::Error::from(rawzip::ErrorKind::IO(
                io::Error::other("Failed to write `plugin.txt` to string :("),
            )));
        }

        write_file_to_zip(&mut archive, "plugin.txt", plugin_txt.as_bytes())?;
    }

    // `data/`
    write_dir_to_zip(&mut archive, "data/")?;

    // `data/full_map_mission.txt`
    {
        let mission_txt_source = output_data.insert_source(String::new());

        let mission = tree_from_tokens!(
            &mut output_data; mission_txt_source =>
            : "mission", format!("Full Map: I know where everything is now") ;
            {
                : "name", "Map Reveal" ;
                : "description", "You can now see every system and planet on the map. Shrouded and hidden systems may disappear again" ;
                : "job" ;
                : "repeat" ;
                : "on", "accept" ;
                {
                    : "event", "Full Map: I know where everything is now", "0" ;
                    : "fail" ;
                }
            }
        );

        output_data.push_root_node(mission_txt_source, mission);

        let mut mission_txt = String::new();

        if output_data
            .write(&mut mission_txt, mission_txt_source, mission, 0)
            .is_err()
        {
            return Err(rawzip::Error::from(rawzip::ErrorKind::IO(
                io::Error::other("Failed to write `data/full_map_mission.txt` to string :("),
            )));
        }

        write_file_to_zip(
            &mut archive,
            "data/full_map_mission.txt",
            mission_txt.as_bytes(),
        )?;
    }

    // `data/full_map_event.txt`
    {
        let event_txt_source = output_data.insert_source(String::new());

        let event = tree_from_tokens!(
            &mut output_data; event_txt_source =>
            : "event", format!("Full Map: I know where everything is now") ;
        );

        let mut planet_names = vec![];

        for (source_index, system) in node_path_iter!(
            data; "system"
        )
        .filter(|(_, node_index)| data.get_tokens(*node_index).unwrap_or_default().len() >= 2)
        {
            let visit_system = tree_from_tokens!(
                &mut output_data; event_txt_source =>
                : "visit", data.get_tokens(system).and_then(|t| data.get_lexeme(source_index, t[1])).unwrap() ;
            );

            output_data.push_child(event, visit_system);

            find_named_objects(data, source_index, system, &mut planet_names);
        }

        for planet_name in planet_names {
            let visit_system = tree_from_tokens!(
                &mut output_data; event_txt_source =>
                : "visit planet", planet_name ;
            );

            output_data.push_child(event, visit_system);
        }

        output_data.push_root_node(event_txt_source, event);

        let mut event_txt = String::new();

        if output_data
            .write(&mut event_txt, event_txt_source, event, 0)
            .is_err()
        {
            return Err(rawzip::Error::from(rawzip::ErrorKind::IO(
                io::Error::other("Failed to write `data/full_map_event.txt` to string :("),
            )));
        }

        write_file_to_zip(
            &mut archive,
            "data/full_map_event.txt",
            event_txt.as_bytes(),
        )?;
    }

    archive.finish()?;

    Ok(output)
}
