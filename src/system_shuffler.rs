use crate::import_from_javascript;
use crate::wandom::ShuffleIndex;
use crate::zippy::Zip;
use endless_sky_rw::*;

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io,
    path::PathBuf,
};

use wasm_bindgen::prelude::*;

const PLUGIN_NAME: &str = "System Shuffler";

const PLUGIN_VERSION: &str = "0.3.8";

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct SystemShufflerConfig {
    seed: usize,
    max_presets: u8,
    shuffle_chance: u8,
    fixed_shuffle_days: u8,
    shuffle_once_on_install: bool,
}

#[wasm_bindgen]
impl SystemShufflerConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: usize,
        max_presets: u8,
        shuffle_chance: u8,
        fixed_shuffle_days: u8,
        shuffle_once_on_install: bool,
    ) -> Self {
        Self {
            seed,
            max_presets,
            shuffle_chance,
            fixed_shuffle_days,
            shuffle_once_on_install,
        }
    }
}

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

fn copy_node(
    data: &Data,
    (source_index, node_index): (SourceIndex, NodeIndex),
    output_data: &mut Data,
    output_source: SourceIndex,
    allow_object: bool,
) -> Option<NodeIndex> {
    let tokens = data.get_tokens(node_index)?;

    if tokens.is_empty() {
        return None;
    }

    let output_node = output_data.insert_node(Node::Some { tokens: vec![] });

    for token in tokens {
        if let Some(lexeme) = data.get_lexeme(source_index, *token)
            && let Some((span_start, span_end)) = output_data.push_source(output_source, lexeme)
        {
            output_data.push_token(
                output_node,
                Token::new(TokenKind::Symbol, Span::new(span_start, span_end)),
            );
        }
    }

    if let Some(children) = data.get_children(node_index) {
        for child in children {
            if (allow_object
                || data
                    .get_tokens(*child)
                    .and_then(|tokens| tokens.first())
                    .and_then(|t| data.get_lexeme(source_index, *t))
                    .unwrap_or_default()
                    != "object")
                && let Some(output_child) = copy_node(
                    data,
                    (source_index, *child),
                    output_data,
                    output_source,
                    allow_object,
                )
            {
                output_data.push_child(output_node, output_child);
            }
        }
    }

    Some(output_node)
}

pub fn process(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: SystemShufflerConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = match read_upload(paths, sources) {
        Some((data_folder, errors)) => {
            if !errors.is_empty() {
                let error_string = String::from_utf8(errors)?;

                import_from_javascript::error(error_string.as_str());
            }

            data_folder
        }
        None => {
            return Err(Box::new(
                io::Error::other(
                    "ERROR: Somehow, everything went wrong while reading the data folder. You're on your own.".to_owned()
                )
            ));
        }
    };

    let data = data_folder.data();

    let mut output_data = Data::default();

    let mut output = vec![];

    let mut archive = Zip::new(&mut output);

    {
        let plugin_description_txt = format!(
            "\
            An Endless Sky \"no logic\" location randomizer.\n\
            \n\n\
            \
            {}\
            - PRNG seed: {}\n\
            - {} possible universe presets\n\
            {}\
            {}
            ",
            if settings.shuffle_once_on_install {
                "In addition to shuffling once immediately upon installation, this plugin was generated with the following settings:\n"
            } else {
                "This plugin was generated with the following settings:\n"
            },
            settings.seed,
            settings.max_presets,
            if settings.shuffle_chance > 0 {
                format!(
                    "- A {}% chance to shuffle to a different preset every time you land\n",
                    settings.shuffle_chance
                )
            } else {
                "".to_owned()
            },
            if settings.fixed_shuffle_days > 0 {
                format!(
                    "- A guaranteed shuffle roughly once every {} days\n",
                    settings.fixed_shuffle_days
                )
            } else {
                "".to_owned()
            },
        );

        let output_root_node_count = output_data.root_nodes().len();
        let plugin_txt_source = output_data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut output_data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        output_data.push_root_node(plugin_txt_source, plugin_name);

        let mut plugin_description = vec![];

        for about in plugin_description_txt.lines().map(|t| t.trim()) {
            if !about.is_empty() {
                let plugin_about = tree_from_tokens!(
                    &mut output_data; plugin_txt_source =>
                    : "about", about ;
                );

                plugin_description.push(plugin_about);

                output_data.push_root_node(plugin_txt_source, plugin_about);
            }
        }

        let plugin_version = tree_from_tokens!(
            &mut output_data; plugin_txt_source =>
            : "version", PLUGIN_VERSION ;
        );

        output_data.push_root_node(plugin_txt_source, plugin_version);

        zip_root_nodes(
            &mut archive,
            "plugin.txt",
            &output_data,
            &output_data.root_nodes()[output_root_node_count..],
        )?;
    }

    let mut shuffle_storage = Data::default();

    let shuffle_storage_source = shuffle_storage.insert_source(String::new());

    let mut system_names = HashSet::new();

    let mut persistent_system_nodes = HashMap::new();

    let mut planets = HashMap::new();
    let mut wormholes = HashSet::new();

    find_wormholes_from_planets(data, &mut wormholes);

    data_from_system(
        data,
        node_path_iter!(&data; "system").chain(node_path_iter!(&data; "wormhole")),
        &mut system_names,
        (&mut planets, &mut wormholes),
        &mut shuffle_storage,
        shuffle_storage_source,
        &mut persistent_system_nodes,
    );

    let mut persistent_event_system_nodes = HashMap::new();

    for (source_index, node_index) in
        node_path_iter!(&data; "event").filter(|(source_index, node_index)| {
            data.get_tokens(*node_index).unwrap_or_default().len() >= 2
                && !data
                    .get_children(*node_index)
                    .unwrap_or_default()
                    .is_empty()
                && (node_path_iter!(&data => (*source_index, *node_index); "system")
                    .next()
                    .is_some()
                    || node_path_iter!(&data => (*source_index, *node_index); "wormhole")
                        .next()
                        .is_some())
        })
    {
        let event_name = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
            .unwrap();

        let mut event_map = HashMap::new();

        data_from_system(
            data,
            node_path_iter!(&data => (source_index, node_index); "system")
                .chain(node_path_iter!(&data => (source_index, node_index); "wormhole"))
                .map(|n| (source_index, n)),
            &mut system_names,
            (&mut planets, &mut wormholes),
            &mut shuffle_storage,
            shuffle_storage_source,
            &mut event_map,
        );

        if !event_map.is_empty() {
            persistent_event_system_nodes.insert(event_name, event_map);
        }
    }

    let mut system_names = system_names.into_iter().collect::<Vec<_>>();

    system_names.sort();

    let installed = "System Shuffler: Installed";
    let current_preset = "System Shuffler: Current Preset";
    let last_shuffle_day = "System Shuffler: Last Shuffle";

    let restore_prefix = "System Shuffler: Restore Preset";
    let activate_prefix = "System Shuffler: Activate Preset";

    archive.write_dir("data/")?;

    {
        let output_root_node_count = output_data.root_nodes().len();

        {
            let main_mission_source = output_data.insert_source(String::new());

            let main_mission = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "mission", "System Shuffler: Select Preset" ;
                {
                    : "invisible" ;
                    : "repeat" ;
                    : "non-blocking" ;
                    : "landing" ;
                    : "offer precedence", "1000000" ;
                }
            );

            output_data.push_root_node(main_mission_source, main_mission);

            let main_mission_to_offer = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "to", "offer" ;
            );

            output_data.push_child(main_mission, main_mission_to_offer);

            if settings.shuffle_chance == 0
                && settings.fixed_shuffle_days == 0
                && !settings.shuffle_once_on_install
            {
                let main_mission_never = tree_from_tokens!(
                    &mut output_data; main_mission_source =>
                    : "never" ;
                );

                output_data.push_child(main_mission_to_offer, main_mission_never);
            } else {
                let main_mission_to_offer_or = tree_from_tokens!(
                    &mut output_data; main_mission_source =>
                    : "or" ;
                );

                output_data.push_child(main_mission_to_offer, main_mission_to_offer_or);

                if settings.shuffle_chance > 0 {
                    let random_chance = tree_from_tokens!(
                        &mut output_data; main_mission_source =>
                        : "random", "<", settings.shuffle_chance ;
                    );

                    output_data.push_child(main_mission_to_offer_or, random_chance);
                }

                if settings.fixed_shuffle_days > 0 {
                    let guaranteed = tree_from_tokens!(
                        &mut output_data; main_mission_source =>
                        : "days since epoch", ">=", "(", last_shuffle_day, "+", settings.fixed_shuffle_days, ")" ;
                    );

                    output_data.push_child(main_mission_to_offer_or, guaranteed);
                }

                if settings.shuffle_once_on_install {
                    let first_time = tree_from_tokens!(
                        &mut output_data; main_mission_source =>
                        : "not", installed ;
                    );

                    output_data.push_child(main_mission_to_offer_or, first_time);
                }
            }

            let main_mission_on_offer = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "on", "offer" ;
            );

            output_data.push_child(main_mission, main_mission_on_offer);

            let main_mission_conversation = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "conversation" ;
                {
                    : "The universe has shuffled. Good luck." ;
                }
            );

            output_data.push_child(main_mission_on_offer, main_mission_conversation);

            conditional_events(
                &settings,
                &mut output_data,
                (main_mission_source, main_mission_conversation),
                current_preset,
                restore_prefix,
                "restore",
            );

            let preset_selection = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "action" ;
                {
                    : installed, "=", "1" ;
                    : current_preset, "=", "(", format!("roll: {}", settings.max_presets).as_str(), "+", "1", ")" ;
                    : last_shuffle_day, "=", "days since epoch" ;
                }
            );

            output_data.push_child(main_mission_conversation, preset_selection);

            conditional_events(
                &settings,
                &mut output_data,
                (main_mission_source, main_mission_conversation),
                current_preset,
                activate_prefix,
                "activate",
            );

            let main_failure = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "fail" ;
            );

            output_data.push_child(main_mission_on_offer, main_failure);
        }

        {
            let restore_job_source = output_data.insert_source(String::new());

            let restore_job = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "mission", "System Shuffler: Restore Universe" ;
                {
                    : "name", "Unshuffle the universe" ;
                    : "description", "Restore all systems in the universe to how they should be, free of charge." ;
                    : "repeat" ;
                    : "job" ;
                }
            );

            output_data.push_root_node(restore_job_source, restore_job);

            let restore_job_to_offer = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "to", "offer" ;
                {
                    : current_preset, "!=", "0" ;
                }
            );

            output_data.push_child(restore_job, restore_job_to_offer);

            let restore_job_on_accept = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "on", "accept" ;
            );

            output_data.push_child(restore_job, restore_job_on_accept);

            let restore_job_conversation = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "conversation" ;
                {
                    : "As per your request, the universe has been restored." ;
                }
            );

            output_data.push_child(restore_job_on_accept, restore_job_conversation);

            conditional_events(
                &settings,
                &mut output_data,
                (restore_job_source, restore_job_conversation),
                current_preset,
                restore_prefix,
                "restore",
            );

            let preset_selection = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "action" ;
                {
                    : current_preset, "=", "0" ;
                    : last_shuffle_day, "=", "days since epoch" ;
                    : "event", format!("wormhole: {activate_prefix} 0").as_str(), "0" ;
                    : "event", format!("system: {activate_prefix} 0").as_str(), "0" ;
                }
            );

            output_data.push_child(restore_job_conversation, preset_selection);

            let main_failure = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "fail" ;
            );

            output_data.push_child(restore_job_on_accept, main_failure);
        }

        {
            let manual_job_source = output_data.insert_source(String::new());

            let manual_job = tree_from_tokens!(
                &mut output_data; manual_job_source =>
                : "mission", "System Shuffler: Manual Shuffle" ;
                {
                    : "name", "Shuffle the universe" ;
                    : "description", format!("Shuffle all systems in the universe to one of {} presets.", settings.max_presets).as_str() ;
                    : "repeat" ;
                    : "job" ;
                }
            );

            output_data.push_root_node(manual_job_source, manual_job);

            let manual_job_on_accept = tree_from_tokens!(
                &mut output_data; manual_job_source =>
                : "on", "accept" ;
            );

            output_data.push_child(manual_job, manual_job_on_accept);

            let manual_job_conversation = tree_from_tokens!(
                &mut output_data; manual_job_source =>
                : "conversation" ;
                {
                    : "As per your request, the universe has shuffled. Good luck." ;
                }
            );

            output_data.push_child(manual_job_on_accept, manual_job_conversation);

            conditional_events(
                &settings,
                &mut output_data,
                (manual_job_source, manual_job_conversation),
                current_preset,
                restore_prefix,
                "restore",
            );

            let preset_selection = tree_from_tokens!(
                &mut output_data; manual_job_source =>
                : "action" ;
                {
                    : installed, "=", "1" ;
                    : current_preset, "=", "(", format!("roll: {}", settings.max_presets).as_str(), "+", "1", ")" ;
                    : last_shuffle_day, "=", "days since epoch" ;
                }
            );

            output_data.push_child(manual_job_conversation, preset_selection);

            conditional_events(
                &settings,
                &mut output_data,
                (manual_job_source, manual_job_conversation),
                current_preset,
                activate_prefix,
                "activate",
            );

            let main_failure = tree_from_tokens!(
                &mut output_data; manual_job_source =>
                : "fail" ;
            );

            output_data.push_child(manual_job_on_accept, main_failure);
        }

        zip_root_nodes(
            &mut archive,
            "data/main.txt",
            &output_data,
            &output_data.root_nodes()[output_root_node_count..],
        )?;
    }

    archive.write_dir("data/presets/")?;

    for preset_index in 0..=(settings.max_presets) {
        let preset_index = usize::from(preset_index);

        let shuffle_event_source = output_data.insert_source(String::new());

        let shuffled = if preset_index == 0 {
            (0..(system_names.len()))
                .map(|i| system_names[i])
                .collect::<Vec<_>>()
        } else {
            system_names
                .shuffled_indices(settings.seed.wrapping_add(preset_index))
                .into_iter()
                .map(|i| system_names[i])
                .collect::<Vec<_>>()
        };

        let system_swaps = system_names
            .iter()
            .copied()
            .zip(shuffled)
            .collect::<HashMap<_, _>>();

        let preset_path = format!("data/presets/universe_preset_{preset_index}");

        archive.write_dir(format!("{preset_path}/"))?;

        let restore_name = format!("{restore_prefix} {preset_index}");
        let activate_name = format!("{activate_prefix} {preset_index}");

        {
            let output_root_node_count = output_data.root_nodes().len();

            for original_kind in ["wormhole", "system"] {
                if !persistent_system_nodes
                    .keys()
                    .any(|&(node_kind, _)| node_kind == original_kind)
                {
                    continue;
                }

                let shuffle_event_restore = tree_from_tokens!(
                    &mut output_data; shuffle_event_source =>
                    : "event", format!("{original_kind}: {}", restore_name.as_str()) ;
                );

                output_data.push_root_node(shuffle_event_source, shuffle_event_restore);

                let shuffle_event_activate = tree_from_tokens!(
                    &mut output_data; shuffle_event_source =>
                    : "event", format!("{original_kind}: {}", activate_name.as_str()) ;
                );

                output_data.push_root_node(shuffle_event_source, shuffle_event_activate);

                for &(_, original) in persistent_system_nodes
                    .keys()
                    .filter(|&&(node_kind, _)| node_kind == original_kind)
                {
                    let replacement = if original_kind == "system" {
                        *system_swaps.get(original).unwrap()
                    } else {
                        original
                    };

                    let (removal, addition) = modify_node(
                        (original_kind, original, replacement),
                        &shuffle_storage,
                        shuffle_storage_source,
                        &mut output_data,
                        shuffle_event_source,
                        &system_swaps,
                        &persistent_system_nodes,
                    );

                    output_data.push_child(shuffle_event_restore, removal);
                    output_data.push_child(shuffle_event_activate, addition);
                }
            }

            zip_root_nodes(
                &mut archive,
                format!("{preset_path}/main.txt"),
                &output_data,
                &output_data.root_nodes()[output_root_node_count..],
            )?;
        }

        {
            let output_root_node_count = output_data.root_nodes().len();

            for (&event_name, event_map) in persistent_event_system_nodes.iter() {
                for original_kind in ["wormhole", "system"] {
                    if !event_map
                        .keys()
                        .any(|&(node_kind, _)| node_kind == original_kind)
                    {
                        continue;
                    }

                    let shuffle_event_restore = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "event", format!("{original_kind}: {}: {event_name}", restore_name.as_str()) ;
                    );

                    output_data.push_root_node(shuffle_event_source, shuffle_event_restore);

                    let shuffle_event_activate = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "event", format!("{original_kind}: {}: {event_name}", activate_name.as_str()) ;
                    );

                    output_data.push_root_node(shuffle_event_source, shuffle_event_activate);

                    for &(_, original) in event_map
                        .keys()
                        .filter(|&&(node_kind, _)| node_kind == original_kind)
                    {
                        let replacement = if original_kind == "system" {
                            *system_swaps.get(original).unwrap()
                        } else {
                            original
                        };

                        let (removal, addition) = modify_node(
                            (original_kind, original, replacement),
                            &shuffle_storage,
                            shuffle_storage_source,
                            &mut output_data,
                            shuffle_event_source,
                            &system_swaps,
                            &persistent_system_nodes,
                        );

                        output_data.push_child(shuffle_event_restore, removal);
                        output_data.push_child(shuffle_event_activate, addition);
                    }
                }
            }

            zip_root_nodes(
                &mut archive,
                format!("{preset_path}/events.txt"),
                &output_data,
                &output_data.root_nodes()[output_root_node_count..],
            )?;
        }

        {
            let output_root_node_count = output_data.root_nodes().len();

            for (&event_name, event_map) in persistent_event_system_nodes.iter() {
                for (should_activate, kind_name) in [
                    (false, restore_name.as_str()),
                    (true, activate_name.as_str()),
                ] {
                    let shuffle_mission = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "mission", format!("{kind_name}: {event_name}") ;
                        {
                            : "invisible" ;
                            : "repeat" ;
                            : "non-blocking" ;
                            : "landing" ;
                            : "offer precedence", "-1000000" ;
                        }
                    );

                    output_data.push_root_node(shuffle_event_source, shuffle_mission);

                    let mission_to_offer = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "to", "offer" ;
                        {
                            :
                                match should_activate {
                                    false => "not",
                                    true => "has",
                                },
                                format!("event: {event_name}")
                            ;
                        }
                    );

                    output_data.push_child(shuffle_mission, mission_to_offer);

                    let mission_on_offer = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "on", "offer" ;
                    );

                    output_data.push_child(shuffle_mission, mission_on_offer);

                    for original_kind in ["wormhole", "system"] {
                        if !event_map
                            .keys()
                            .any(|&(node_kind, _)| node_kind == original_kind)
                        {
                            continue;
                        }

                        match should_activate {
                            false => {
                                let condition = tree_from_tokens!(
                                    &mut output_data; shuffle_event_source =>
                                    : "has", format!("event: {original_kind}: {activate_name}: {event_name}") ;
                                );

                                output_data.push_child(mission_to_offer, condition);

                                let action = tree_from_tokens!(
                                    &mut output_data; shuffle_event_source =>
                                        : "event", format!("{original_kind}: {restore_name}: {event_name}"), "0" ;
                                );

                                output_data.push_child(mission_on_offer, action);

                                let action = tree_from_tokens!(
                                    &mut output_data; shuffle_event_source =>
                                        : format!("event {original_kind}: {activate_name}: {event_name}"), "=", "0" ;
                                );

                                output_data.push_child(mission_on_offer, action);
                            }
                            true => {
                                let condition = tree_from_tokens!(
                                    &mut output_data; shuffle_event_source =>
                                    : format!("event: {original_kind}: {activate_name}: {event_name}"), "!=", format!("event: {event_name}") ;
                                );

                                output_data.push_child(mission_to_offer, condition);

                                let action = tree_from_tokens!(
                                    &mut output_data; shuffle_event_source =>
                                    : "event", format!("{original_kind}: {activate_name}: {event_name}"), "0" ;
                                );

                                output_data.push_child(mission_on_offer, action);

                                let action = tree_from_tokens!(
                                    &mut output_data; shuffle_event_source =>
                                    : format!("event: {original_kind}: {activate_name}: {event_name}"), "=", format!("event: {event_name}") ;
                                );

                                output_data.push_child(mission_on_offer, action);
                            }
                        }
                    }

                    let mission_failure = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "fail" ;
                    );

                    output_data.push_child(mission_on_offer, mission_failure);
                }
            }

            zip_root_nodes(
                &mut archive,
                format!("{preset_path}/missions.txt"),
                &output_data,
                &output_data.root_nodes()[output_root_node_count..],
            )?;
        }
    }

    archive.finish()?;

    Ok(output)
}

fn find_wormholes_from_planets<'a>(data: &'a Data, wormholes: &mut HashSet<&'a str>) {
    for (source_index, node_index) in
        node_path_iter!(data; "planet").filter(|(source_index, node_index)| {
            data.get_tokens(*node_index).unwrap_or_default().len() >= 2
                && !data
                    .get_children(*node_index)
                    .unwrap_or_default()
                    .is_empty()
                && node_path_iter!(data => (*source_index, *node_index); "wormhole")
                    .filter(|&node_index| {
                        data.get_tokens(node_index).unwrap_or_default().len() >= 2
                    })
                    .next()
                    .is_some()
        })
    {
        wormholes.insert(
            data.get_tokens(node_index)
                .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
                .unwrap(),
        );
    }
}

fn find_wormholes_from_system<'a>(
    data: &'a Data,
    (system_name, source_index, node_index): (&'a str, SourceIndex, NodeIndex),
    (depth, planets, wormholes): (u64, &mut HashMap<&'a str, &'a str>, &mut HashSet<&'a str>),
    shuffle_storage: &mut Data,
    shuffle_storage_source: SourceIndex,
    persistent_system_nodes: &mut HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
) -> bool {
    let mut is_any_wormhole = false;

    for child in node_path_iter!(data => (source_index, node_index); "object").chain(
        node_path_iter!(data => (source_index, node_index); "add" | "remove").filter(
            |&node_index| {
                data.get_tokens(node_index)
                    .and_then(|tokens| tokens.get(1))
                    .and_then(|t| data.get_lexeme(source_index, *t))
                    .unwrap_or_default()
                    == "object"
            },
        ),
    ) {
        let mut is_wormhole = false;

        if let Some(object_name) = data
            .get_tokens(child)
            .and_then(|tokens| tokens.get(1))
            .and_then(|t| data.get_lexeme(source_index, *t))
        {
            if let Some(name) = planets.get(object_name)
                && *name != system_name
            {
                wormholes.insert(object_name);
            }

            planets.insert(object_name, system_name);

            if wormholes.contains(object_name) {
                is_any_wormhole |= true;
                is_wormhole = true;
            }
        }

        is_wormhole |= find_wormholes_from_system(
            data,
            (system_name, source_index, child),
            (depth + 1, planets, wormholes),
            shuffle_storage,
            shuffle_storage_source,
            persistent_system_nodes,
        );

        if depth == 0 && is_wormhole {
            let (removal, addition) = addition_and_removal(
                data,
                ("object", source_index, child),
                shuffle_storage,
                shuffle_storage_source,
            );

            let is_removing = data
                .get_lexeme(source_index, data.get_tokens(child).unwrap_or_default()[0])
                .unwrap_or_default()
                == "remove";

            persistent_system_nodes.persist(
                "system",
                system_name,
                "object",
                OriginalNode::new(is_removing, removal, addition),
            );
        }
    }

    is_any_wormhole
}

type OriginalNodes<'a> = HashMap<&'a str, Vec<(bool, NodeIndex, NodeIndex)>>;

struct OriginalNode {
    is_removing: bool,
    removal: NodeIndex,
    addition: NodeIndex,
}

impl OriginalNode {
    fn new(is_removing: bool, removal: NodeIndex, addition: NodeIndex) -> Self {
        Self {
            is_removing,
            removal,
            addition,
        }
    }
}

trait NodePersistence<'a> {
    fn persist(
        &mut self,
        original_node_kind: &'a str,
        original_node_name: &'a str,
        node_kind: &'a str,
        node: OriginalNode,
    );
}

impl<'a> NodePersistence<'a> for HashMap<(&'a str, &'a str), OriginalNodes<'a>> {
    fn persist(
        &mut self,
        original_node_kind: &'a str,
        original_node_name: &'a str,
        node_kind: &'a str,
        node: OriginalNode,
    ) {
        HashMap::entry(self, (original_node_kind, original_node_name))
            .and_modify(|e| {
                e.entry(node_kind)
                    .and_modify(|v| {
                        v.push((node.is_removing, node.removal, node.addition));
                    })
                    .or_insert(vec![(node.is_removing, node.removal, node.addition)]);
            })
            .or_insert({
                let mut nodes = HashMap::new();
                nodes.insert(
                    node_kind,
                    vec![(node.is_removing, node.removal, node.addition)],
                );
                nodes
            });
    }
}

fn data_from_system<'a>(
    data: &'a Data,
    nodes: impl Iterator<Item = (SourceIndex, NodeIndex)>,
    system_names: &mut HashSet<&'a str>,
    (planets, wormholes): (&mut HashMap<&'a str, &'a str>, &mut HashSet<&'a str>),
    shuffle_storage: &mut Data,
    shuffle_storage_source: SourceIndex,
    persistent_system_nodes: &mut HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
) {
    for (source_index, node_index) in nodes.filter(|(_, node_index)| {
        data.get_tokens(*node_index).unwrap_or_default().len() >= 2
            && !data
                .get_children(*node_index)
                .unwrap_or_default()
                .is_empty()
    }) {
        let original_node_kind = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[0]))
            .unwrap();

        let original_node_name = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
            .unwrap();

        if original_node_kind == "system" {
            system_names.insert(original_node_name);

            find_wormholes_from_system(
                data,
                (original_node_name, source_index, node_index),
                (0, planets, wormholes),
                shuffle_storage,
                shuffle_storage_source,
                persistent_system_nodes,
            );
        }

        for child in node_path_iter!(
            data => (source_index, node_index); "pos"
        )
        .filter(|&child| data.get_tokens(child).unwrap_or_default().len() >= 2)
        {
            let x = data
                .get_lexeme(source_index, data.get_tokens(child).unwrap()[1])
                .unwrap();

            let y = data
                .get_lexeme(source_index, data.get_tokens(child).unwrap()[2])
                .unwrap();

            let copied_node = tree_from_tokens!(
                shuffle_storage; shuffle_storage_source =>
                : "pos", x, y ;
            );

            persistent_system_nodes.persist(
                original_node_kind,
                original_node_name,
                "pos",
                OriginalNode::new(false, copied_node, copied_node),
            );
        }

        for node_kind in [
            ["link", "jump range", "inaccessible", "hidden", "shrouded"].as_slice(),
            // TODO: find a way for this to work without crashing the game (may be impossible for a plugin)
            // (wormhole logic can be removed if it is possible)
            // ["object", "hazard", "arrival", "departure"].as_slice(),
        ]
        .concat()
        {
            for child in data.filter_children(source_index, node_index, |source_index, tokens| {
                let key_index = if matches!(
                    tokens
                        .first()
                        .and_then(|t| data.get_lexeme(source_index, *t)),
                    Some("remove" | "add")
                ) {
                    1
                } else {
                    0
                };

                matches!(
                    tokens
                        .get(key_index)
                        .and_then(|t| data.get_lexeme(source_index, *t)),
                    Some(l) if l == node_kind
                )
            }) {
                let (removal, addition) = addition_and_removal(
                    data,
                    (node_kind, source_index, child),
                    shuffle_storage,
                    shuffle_storage_source,
                );

                let is_removing = data
                    .get_lexeme(source_index, data.get_tokens(child).unwrap_or_default()[0])
                    .unwrap_or_default()
                    == "remove";

                persistent_system_nodes.persist(
                    original_node_kind,
                    original_node_name,
                    node_kind,
                    OriginalNode::new(is_removing, removal, addition),
                );
            }
        }
    }
}

fn addition_and_removal(
    data: &Data,
    (node_kind, source_index, child): (&str, SourceIndex, NodeIndex),
    shuffle_storage: &mut Data,
    shuffle_storage_source: SourceIndex,
) -> (NodeIndex, NodeIndex) {
    let (mut removal, mut addition) = (None, None);

    for should_activate in [false, true] {
        let action = match node_kind {
            "jump range" | "inaccessible" | "hidden" | "shrouded" | "arrival" | "departure" => {
                match should_activate {
                    false => tree_from_tokens!(
                        shuffle_storage; shuffle_storage_source =>
                        : "remove", node_kind ;
                    ),
                    true => copy_node(
                        data,
                        (source_index, child),
                        shuffle_storage,
                        shuffle_storage_source,
                        should_activate,
                    )
                    .unwrap(),
                }
            }
            _ => {
                let action = copy_node(
                    data,
                    (source_index, child),
                    shuffle_storage,
                    shuffle_storage_source,
                    should_activate,
                )
                .unwrap();

                let (span_start, span_end) = shuffle_storage
                    .push_source(
                        shuffle_storage_source,
                        match should_activate {
                            false => "remove",
                            true => "add",
                        },
                    )
                    .unwrap();

                match shuffle_storage.get_mut_node(action).unwrap() {
                    Node::Error => unreachable!(),
                    Node::Some { tokens } | Node::Parent { tokens, .. } => {
                        if data
                            .get_tokens(child)
                            .and_then(|tokens| data.get_lexeme(source_index, tokens[0]))
                            .unwrap()
                            == node_kind
                        {
                            tokens.insert(
                                0,
                                Token::new(TokenKind::Symbol, Span::new(span_start, span_end)),
                            );
                        } else {
                            *tokens.first_mut().unwrap() =
                                Token::new(TokenKind::Symbol, Span::new(span_start, span_end));
                        }
                    }
                }

                action
            }
        };

        match should_activate {
            false => removal = Some(action),
            true => addition = Some(action),
        }
    }

    (removal.take().unwrap(), addition.take().unwrap())
}

fn modify_node(
    (original_kind, original, replacement): (&str, &str, &str),
    shuffle_storage: &Data,
    shuffle_storage_source: SourceIndex,
    output_data: &mut Data,
    shuffle_event_source: SourceIndex,
    system_swaps: &HashMap<&str, &str>,
    persistent_system_nodes: &HashMap<(&str, &str), OriginalNodes<'_>>,
) -> (NodeIndex, NodeIndex) {
    let persistent_nodes = persistent_system_nodes
        .get(&(original_kind, original))
        .unwrap();

    let (mut restoration, mut activation) = (None, None);

    for should_activate in [false, true] {
        let modified_node = tree_from_tokens!(
            output_data; shuffle_event_source =>
            : original_kind, replacement ;
        );

        for (node_kind, node_values) in persistent_nodes.iter() {
            for (i, node_value) in node_values.iter().enumerate() {
                match *node_kind {
                    "link" => {
                        let modified_link = if (should_activate && !node_value.0)
                            || (!should_activate && node_value.0)
                        {
                            let modified_link = tree_from_tokens!(
                                output_data; shuffle_event_source =>
                                : "link" ;
                            );

                            for lexeme in shuffle_storage
                                .get_tokens(node_value.1)
                                .unwrap_or_default()
                                .iter()
                                .skip(2)
                                .flat_map(|t| {
                                    shuffle_storage.get_lexeme(shuffle_storage_source, *t)
                                })
                            {
                                let (start, end) = output_data
                                    .push_source(
                                        shuffle_event_source,
                                        system_swaps.get(lexeme).unwrap(),
                                    )
                                    .unwrap();

                                output_data.push_token(
                                    modified_link,
                                    Token::new(TokenKind::Symbol, Span::new(start, end)),
                                );
                            }

                            modified_link
                        } else {
                            if i > 0 {
                                break;
                            }

                            tree_from_tokens!(
                                output_data; shuffle_event_source =>
                                : "remove", "link" ;
                            )
                        };

                        output_data.push_child(modified_node, modified_link);
                    }
                    _ => {
                        if i > 0 {
                            match *node_kind {
                                "link" | "jump range" | "inaccessible" | "hidden" | "shrouded"
                                | "arrival" | "departure" => break,
                                _ => {}
                            }
                        }

                        let modified_copy = copy_node(
                            shuffle_storage,
                            (
                                shuffle_storage_source,
                                if (should_activate && !node_value.0)
                                    || (!should_activate && node_value.0)
                                {
                                    node_value.2
                                } else {
                                    node_value.1
                                },
                            ),
                            output_data,
                            shuffle_event_source,
                            true,
                        )
                        .unwrap();

                        output_data.push_child(modified_node, modified_copy);
                    }
                }
            }
        }

        match should_activate {
            false => restoration = Some(modified_node),
            true => activation = Some(modified_node),
        }
    }

    (restoration.take().unwrap(), activation.take().unwrap())
}

fn conditional_events(
    settings: &SystemShufflerConfig,
    output_data: &mut Data,
    (source, parent): (SourceIndex, NodeIndex),
    condition: &str,
    event_name_prefix: &str,
    label_suffix: &str,
) {
    for preset_index in 0..=(settings.max_presets) {
        let skip_label = format!("not {preset_index} {label_suffix}");

        let event_branch = tree_from_tokens!(
            output_data; source =>
            : "branch", skip_label.as_str() ;
            {
                : condition, "!=", preset_index ;
            }
        );

        output_data.push_child(parent, event_branch);

        let event_action = tree_from_tokens!(
            output_data; source =>
            : "action" ;
            {
                : "event", format!("wormhole: {event_name_prefix} {preset_index}"), "0" ;
                : "event", format!("system: {event_name_prefix} {preset_index}"), "0" ;
            }
        );

        output_data.push_child(parent, event_action);

        let event_label = tree_from_tokens!(
            output_data; source =>
            : "label", skip_label.as_str() ;
        );

        output_data.push_child(parent, event_label);

        if preset_index == settings.max_presets {
            let blank_action = tree_from_tokens!(
                output_data; source =>
                : "action" ;
                {
                    : condition, "=", condition ;
                }
            );

            output_data.push_child(parent, blank_action);
        }
    }
}
