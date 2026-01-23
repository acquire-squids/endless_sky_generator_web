use crate::generators;
use crate::zippy::Zip;

use endless_sky_rw::{
    Data, Node, NodeIndex, SourceIndex, Span, Token, TokenKind, node_path_iter, tree_from_tokens,
};

use std::{error::Error, path::PathBuf};

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
    let data_folder = generators::read_upload(paths, sources)?;

    let data = data_folder.data();

    let mut output = vec![];

    let mut generator = FullMap {
        archive: Zip::new(&mut output),
        output_data: Data::default(),
    };

    generator.description()?;

    generator.archive.write_dir("data/")?;

    generator.main_mission()?;

    generator.main_event(data)?;

    generator.archive.finish()?;

    Ok(output)
}

struct FullMap<'a> {
    archive: Zip<'a>,
    output_data: Data,
}

impl FullMap<'_> {
    fn zip_root_nodes<P: Into<PathBuf>>(
        &mut self,
        path: P,
        from: usize,
    ) -> Result<(), Box<dyn Error>> {
        generators::zip_root_nodes(
            &mut self.archive,
            path,
            &self.output_data,
            &self.output_data.root_nodes()[from..],
        )
    }

    fn description(&mut self) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();
        let plugin_txt_source = self.output_data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut self.output_data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        self.output_data
            .push_root_node(plugin_txt_source, plugin_name);

        for about in PLUGIN_DESCRIPTION.lines().map(str::trim) {
            let plugin_about = tree_from_tokens!(
                &mut self.output_data; plugin_txt_source =>
                : "about", about ;
            );

            self.output_data
                .push_root_node(plugin_txt_source, plugin_about);
        }

        let plugin_version = tree_from_tokens!(
            &mut self.output_data; plugin_txt_source =>
            : "version", PLUGIN_VERSION ;
        );

        self.output_data
            .push_root_node(plugin_txt_source, plugin_version);

        self.zip_root_nodes("plugin.txt", output_root_node_count)
    }

    fn main_mission(&mut self) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();
        let mission_txt_source = self.output_data.insert_source(String::new());

        let mission = tree_from_tokens!(
            &mut self.output_data; mission_txt_source =>
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

        self.output_data.push_root_node(mission_txt_source, mission);

        self.zip_root_nodes("data/full_map_mission.txt", output_root_node_count)
    }

    fn main_event(&mut self, data: &Data) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();
        let event_txt_source = self.output_data.insert_source(String::new());

        let event = tree_from_tokens!(
            &mut self.output_data; event_txt_source =>
            : "event", format!("Full Map: I know where everything is now") ;
        );

        let mut system_names = vec![];
        let mut planet_names = vec![];

        for (source_index, system) in node_path_iter!(
            data; "system"
        )
        .filter(|(_, node_index)| data.get_tokens(*node_index).unwrap_or_default().len() >= 2)
        {
            system_names.push(
                data
                    .get_tokens(system)
                    .and_then(|tokens| tokens.get(1))
                    .and_then(|token| data.get_lexeme(source_index, *token))
                    .expect("The iterator should have a filter applied such that only nodes with two or more tokens are allowed")
            );

            find_named_objects(data, source_index, system, &mut planet_names);
        }

        system_names.sort_unstable();
        planet_names.sort_unstable();

        for system_name in system_names {
            let visit_system = tree_from_tokens!(
                &mut self.output_data; event_txt_source =>
                : "visit", system_name ;
            );

            self.output_data.push_child(event, visit_system);
        }

        for planet_name in planet_names {
            let visit_system = tree_from_tokens!(
                &mut self.output_data; event_txt_source =>
                : "visit planet", planet_name ;
            );

            self.output_data.push_child(event, visit_system);
        }

        self.output_data.push_root_node(event_txt_source, event);

        self.zip_root_nodes("data/full_map_event.txt", output_root_node_count)
    }
}
