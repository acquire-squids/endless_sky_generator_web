use crate::import_from_javascript;
use crate::zippy::Zip;
use endless_sky_rw::*;

use std::{error::Error, io};

const PLUGIN_NAME: &str = "Full Map";

const PLUGIN_DESCRIPTION: &str = "\
    Reveal the entire map via any job board\
";

const PLUGIN_VERSION: &str = "0.1.0";

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

pub fn process(paths: Vec<String>, sources: Vec<String>) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = match read_upload(paths, sources) {
        Some((data_folder, errors)) => {
            if !errors.is_empty() {
                let error_string = String::from_utf8(errors)?;

                import_from_javascript::error(error_string.as_str());
            }

            data_folder
        }
        None => {
            return Err(Box::new(rawzip::Error::from(rawzip::ErrorKind::InvalidInput { msg: "ERROR: Somehow, everything went wrong while reading the data folder.  You're on your own.".to_owned() })));
        }
    };

    let data = data_folder.data();

    let mut output_data = Data::default();

    let mut output = vec![];

    let mut archive = Zip::new(&mut output);

    {
        let output_root_node_count = output_data.root_nodes().len();
        let plugin_txt_source = output_data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut output_data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        output_data.push_root_node(plugin_txt_source, plugin_name);

        let mut plugin_description = vec![];

        for about in PLUGIN_DESCRIPTION.lines().map(|t| t.trim()) {
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
        let plugin_path = "plugin.txt";

        if output_data
            .write_root_nodes(
                &mut plugin_txt,
                &output_data.root_nodes()[output_root_node_count..],
            )
            .is_err()
        {
            return Err(Box::new(io::Error::other(format!(
                "Failed to write `{plugin_path}` to string :("
            ))));
        }

        archive.write_file(plugin_path, plugin_txt.trim().as_bytes())?;
    }

    archive.write_dir("data/")?;

    {
        let output_root_node_count = output_data.root_nodes().len();
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
        let mission_path = "data/full_map_mission.txt";

        if output_data
            .write_root_nodes(
                &mut mission_txt,
                &output_data.root_nodes()[output_root_node_count..],
            )
            .is_err()
        {
            return Err(Box::new(io::Error::other(format!(
                "Failed to write `{mission_path}` to string :("
            ))));
        }

        archive.write_file(mission_path, mission_txt.trim().as_bytes())?;
    }

    {
        let output_root_node_count = output_data.root_nodes().len();
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
        let event_path = "data/full_map_event.txt";

        if output_data
            .write_root_nodes(
                &mut event_txt,
                &output_data.root_nodes()[output_root_node_count..],
            )
            .is_err()
        {
            return Err(Box::new(io::Error::other(format!(
                "Failed to write `{event_path}` to string :("
            ))));
        }

        archive.write_file(event_path, event_txt.trim().as_bytes())?;
    }

    archive.finish()?;

    Ok(output)
}
