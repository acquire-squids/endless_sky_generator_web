use crate::zippy::Zip;
use endless_sky_rw::*;

use std::{error::Error, io};

const PLUGIN_NAME: &str = "Plugin Template";

const PLUGIN_DESCRIPTION: &str = "\
    Template plugin\
";

const PLUGIN_VERSION: &str = "0.1.0";

pub fn process(_paths: Vec<String>, _sources: Vec<String>) -> Result<Vec<u8>, Box<dyn Error>> {
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

        output_data.push_root_node(mission_txt_source, mission);

        let mut mission_txt = String::new();
        let mission_path = "data/replace_with_plugin_data.txt";

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

    archive.finish()?;

    Ok(output)
}
