use crate::import_from_javascript;
use crate::wandom::ShuffleIndex;
use crate::zippy::Zip;
use endless_sky_rw::*;

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    io,
    path::PathBuf,
};

use wasm_bindgen::prelude::*;

const PLUGIN_NAME: &str = "System Shuffler";

const PLUGIN_VERSION: &str = "0.4.0";

const INSTALLED: &str = "System Shuffler: Installed";
const CURRENT_PRESET: &str = "System Shuffler: Current Preset";
const LAST_SHUFFLE_DAY: &str = "System Shuffler: Last Shuffle Day";

const RESTORE_PREFIX: &str = "System Shuffler: Restore Preset";
const ACTIVATE_PREFIX: &str = "System Shuffler: Activate Preset";

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

    let mut system_names = HashSet::new();

    let mut persistent_nodes = HashMap::new();

    let mut planets = HashMap::new();
    let mut wormholes = HashSet::new();

    find_wormholes_from_planets(data, &mut wormholes);

    data_from_node(
        data,
        node_path_iter!(&data; "system" | "wormhole"),
        &mut system_names,
        (&mut planets, &mut wormholes),
        &mut persistent_nodes,
    );

    let mut persistent_event_nodes = HashMap::new();

    for (source_index, node_index) in
        node_path_iter!(&data; "event").filter(|&(source_index, node_index)| {
            data.get_tokens(node_index).unwrap_or_default().len() >= 2
                && !data
                    .get_children(node_index)
                    .unwrap_or_default()
                    .is_empty()
                && node_path_iter!(&data => (source_index, node_index); "system" | "wormhole" | "link" | "unlink")
                    .next()
                    .is_some()
        })
    {
        let event_name = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
            .unwrap();

        let mut event_map = HashMap::new();

        data_from_node(
            data,
            node_path_iter!(&data => (source_index, node_index); "system" | "wormhole" | "link" | "unlink")
                .map(|n| (source_index, n)),
            &mut system_names,
            (&mut planets, &mut wormholes),
            &mut event_map,
        );

        if !event_map.is_empty() {
            persistent_event_nodes.insert(event_name, event_map);
        }
    }

    let mut system_names = system_names.into_iter().collect::<Vec<_>>();

    system_names.sort();

    // TODO: sort events by when they happen chronologically?
    let persistent_event_node_keys = persistent_event_nodes.keys().copied().collect::<Vec<_>>();

    archive.write_dir("data/")?;

    {
        let output_root_node_count = output_data.root_nodes().len();

        {
            let main_mission_source = output_data.insert_source(String::new());

            let main_mission = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "mission", "zzzzz System Shuffler: Select Preset" ;
                {
                    : "invisible" ;
                    : "repeat" ;
                    : "non-blocking" ;
                    : "landing" ;
                    : "offer precedence", "-1000000" ;
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
                        : "days since epoch", ">=", "(", LAST_SHUFFLE_DAY, "+", settings.fixed_shuffle_days, ")" ;
                    );

                    output_data.push_child(main_mission_to_offer_or, guaranteed);
                }

                if settings.shuffle_once_on_install {
                    let first_time = tree_from_tokens!(
                        &mut output_data; main_mission_source =>
                        : "not", INSTALLED ;
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
                (false, RESTORE_PREFIX, "restore"),
                persistent_event_node_keys.as_slice(),
            );

            let preset_selection = tree_from_tokens!(
                &mut output_data; main_mission_source =>
                : "action" ;
                {
                    : INSTALLED, "=", "1" ;
                    : CURRENT_PRESET, "=", "(", format!("roll: {}", settings.max_presets).as_str(), "+", "1", ")" ;
                    : LAST_SHUFFLE_DAY, "=", "days since epoch" ;
                }
            );

            output_data.push_child(main_mission_conversation, preset_selection);

            conditional_events(
                &settings,
                &mut output_data,
                (main_mission_source, main_mission_conversation),
                (true, ACTIVATE_PREFIX, "activate"),
                persistent_event_node_keys.as_slice(),
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
                : "mission", "zzzzz System Shuffler: Restore Universe" ;
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
                    : CURRENT_PRESET, "!=", "0" ;
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
                (false, RESTORE_PREFIX, "restore"),
                persistent_event_node_keys.as_slice(),
            );

            let preset_selection = tree_from_tokens!(
                &mut output_data; restore_job_source =>
                : "action" ;
                {
                    : INSTALLED, "=", "1" ;
                    : CURRENT_PRESET, "=", "0" ;
                    : LAST_SHUFFLE_DAY, "=", "days since epoch" ;
                }
            );

            output_data.push_child(restore_job_conversation, preset_selection);

            conditional_events(
                &settings,
                &mut output_data,
                (restore_job_source, restore_job_conversation),
                (true, ACTIVATE_PREFIX, "activate"),
                persistent_event_node_keys.as_slice(),
            );

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
                : "mission", "zzzzz System Shuffler: Manual Shuffle" ;
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
                (false, RESTORE_PREFIX, "restore"),
                persistent_event_node_keys.as_slice(),
            );

            let preset_selection = tree_from_tokens!(
                &mut output_data; manual_job_source =>
                : "action" ;
                {
                    : INSTALLED, "=", "1" ;
                    : CURRENT_PRESET, "=", "(", format!("roll: {}", settings.max_presets).as_str(), "+", "1", ")" ;
                    : LAST_SHUFFLE_DAY, "=", "days since epoch" ;
                }
            );

            output_data.push_child(manual_job_conversation, preset_selection);

            conditional_events(
                &settings,
                &mut output_data,
                (manual_job_source, manual_job_conversation),
                (true, ACTIVATE_PREFIX, "activate"),
                persistent_event_node_keys.as_slice(),
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

        let restore_name = format!("{RESTORE_PREFIX} {preset_index}");
        let activate_name = format!("{ACTIVATE_PREFIX} {preset_index}");

        {
            let output_root_node_count = output_data.root_nodes().len();

            let shuffle_event_restore = tree_from_tokens!(
                &mut output_data; shuffle_event_source =>
                : "event", restore_name.as_str() ;
            );

            output_data.push_root_node(shuffle_event_source, shuffle_event_restore);

            let shuffle_event_activate = tree_from_tokens!(
                &mut output_data; shuffle_event_source =>
                : "event", activate_name.as_str() ;
            );

            output_data.push_root_node(shuffle_event_source, shuffle_event_activate);

            generate_event(
                data,
                &mut output_data,
                shuffle_event_source,
                (shuffle_event_restore, shuffle_event_activate),
                &system_swaps,
                &persistent_nodes,
            );

            zip_root_nodes(
                &mut archive,
                format!("{preset_path}/main.txt"),
                &output_data,
                &output_data.root_nodes()[output_root_node_count..],
            )?;
        }

        {
            let output_root_node_count = output_data.root_nodes().len();

            for (event_name, event_map) in persistent_event_node_keys
                .as_slice()
                .iter()
                .map(|&e| (e, persistent_event_nodes.get(e).unwrap()))
            {
                let shuffle_event_restore = tree_from_tokens!(
                    &mut output_data; shuffle_event_source =>
                    : "event", format!("{restore_name}: {event_name}") ;
                );

                output_data.push_root_node(shuffle_event_source, shuffle_event_restore);

                let shuffle_event_activate = tree_from_tokens!(
                    &mut output_data; shuffle_event_source =>
                    : "event", format!("{activate_name}: {event_name}") ;
                );

                output_data.push_root_node(shuffle_event_source, shuffle_event_activate);

                generate_event(
                    data,
                    &mut output_data,
                    shuffle_event_source,
                    (shuffle_event_restore, shuffle_event_activate),
                    &system_swaps,
                    event_map,
                );
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

            for &event_name in persistent_event_nodes.keys() {
                for (should_activate, kind_name) in [
                    (false, restore_name.as_str()),
                    (true, activate_name.as_str()),
                ] {
                    let shuffle_mission = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "mission", format!("zzzzz {kind_name}: {event_name}") ;
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
                            : "has", INSTALLED ;
                            :
                                CURRENT_PRESET,
                                match should_activate {
                                    false => "!=",
                                    true => "==",
                                },
                                preset_index
                            ;
                        }
                    );

                    output_data.push_child(shuffle_mission, mission_to_offer);

                    let mission_on_offer = tree_from_tokens!(
                        &mut output_data; shuffle_event_source =>
                        : "on", "offer" ;
                    );

                    output_data.push_child(shuffle_mission, mission_on_offer);

                    let ((condition1, condition2), (action1, action2)) =
                        generate_side_event_conditions(
                            event_name,
                            (restore_name.as_str(), activate_name.as_str()),
                            (should_activate, false),
                            &mut output_data,
                            shuffle_event_source,
                        );

                    output_data.push_child(mission_to_offer, condition1);
                    output_data.push_child(mission_to_offer, condition2);

                    output_data.push_child(mission_on_offer, action1);
                    output_data.push_child(mission_on_offer, action2);

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
    persistent_nodes: &mut HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
) -> bool {
    let mut is_any_wormhole = false;

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
            Some(l) if l == "object"
        )
    }) {
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
            persistent_nodes,
        );

        if depth == 0 && is_wormhole {
            let action = match data
                .get_tokens(child)
                .and_then(|tokens| data.get_lexeme(source_index, tokens[0]))
                .unwrap()
            {
                "remove" => {
                    if data.get_tokens(child).unwrap_or_default().len() >= 2
                        || !data.get_children(child).unwrap_or_default().is_empty()
                    {
                        NodeAction::Remove
                    } else {
                        NodeAction::ClearRemove
                    }
                }
                "add" => NodeAction::Add,
                _ => NodeAction::ClearAdd,
            };

            persistent_nodes.persist(
                "system",
                system_name,
                "object",
                OriginalNode::new(action, source_index, child),
            );
        }
    }

    is_any_wormhole
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeAction {
    Remove,
    ClearRemove,
    Add,
    ClearAdd,
}

type OriginalNodes<'a> = HashMap<&'a str, Vec<(NodeAction, SourceIndex, NodeIndex)>>;

struct OriginalNode {
    action: NodeAction,
    source: SourceIndex,
    node: NodeIndex,
}

impl OriginalNode {
    fn new(action: NodeAction, source: SourceIndex, node: NodeIndex) -> Self {
        Self {
            action,
            source,
            node,
        }
    }
}

trait NodePersistence<'a> {
    fn persist(
        &mut self,
        original_node_kind: &'a str,
        original_node_name: &'a str,
        node_kind: &'a str,
        original_node: OriginalNode,
    );
}

impl<'a> NodePersistence<'a> for HashMap<(&'a str, &'a str), OriginalNodes<'a>> {
    fn persist(
        &mut self,
        original_node_kind: &'a str,
        original_node_name: &'a str,
        node_kind: &'a str,
        original_node: OriginalNode,
    ) {
        HashMap::entry(self, (original_node_kind, original_node_name))
            .and_modify(|e| {
                e.entry(node_kind)
                    .and_modify(|v| {
                        if !v.iter().any(|&(action, source, node)| {
                            action == original_node.action
                                && source == original_node.source
                                && node == original_node.node
                        }) {
                            v.push((
                                original_node.action,
                                original_node.source,
                                original_node.node,
                            ));
                        }
                    })
                    .or_insert(vec![(
                        original_node.action,
                        original_node.source,
                        original_node.node,
                    )]);
            })
            .or_insert({
                let mut nodes = HashMap::new();
                nodes.insert(
                    node_kind,
                    vec![(
                        original_node.action,
                        original_node.source,
                        original_node.node,
                    )],
                );
                nodes
            });
    }
}

fn data_from_node<'a>(
    data: &'a Data,
    nodes: impl Iterator<Item = (SourceIndex, NodeIndex)>,
    system_names: &mut HashSet<&'a str>,
    (planets, wormholes): (&mut HashMap<&'a str, &'a str>, &mut HashSet<&'a str>),
    persistent_nodes: &mut HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
) {
    for (source_index, node_index) in
        nodes.filter(|(_, node_index)| data.get_tokens(*node_index).unwrap_or_default().len() >= 2)
    {
        let original_node_kind = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[0]))
            .unwrap();

        let original_node_name = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
            .unwrap();

        match original_node_kind {
            "system" => {
                system_names.insert(original_node_name);

                find_wormholes_from_system(
                    data,
                    (original_node_name, source_index, node_index),
                    (0, planets, wormholes),
                    persistent_nodes,
                );
            }
            "link" => {
                persistent_nodes.persist(
                    original_node_kind,
                    "",
                    original_node_kind,
                    OriginalNode::new(NodeAction::Add, source_index, node_index),
                );
            }
            "unlink" => {
                persistent_nodes.persist(
                    original_node_kind,
                    "",
                    original_node_kind,
                    OriginalNode::new(NodeAction::Remove, source_index, node_index),
                );
            }
            _ => {}
        }

        for node_kind in match original_node_kind {
            // TODO: find a way for this to work without crashing the game (may be impossible for a plugin)
            // (wormhole logic can be removed if it is possible)
            // ["object", "hazard", "arrival", "departure"].as_slice(),
            "system" => [
                "pos",
                "link",
                "jump range",
                "inaccessible",
                "hidden",
                "shrouded",
            ]
            .as_slice(),
            "wormhole" => ["link"].as_slice(),
            _ => [].as_slice(),
        } {
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
                    Some(l) if l == *node_kind
                )
            }) {
                let action = match data
                    .get_tokens(child)
                    .and_then(|tokens| data.get_lexeme(source_index, tokens[0]))
                    .unwrap()
                {
                    "remove" => {
                        if data.get_tokens(child).unwrap_or_default().len() >= 2
                            || !data.get_children(child).unwrap_or_default().is_empty()
                        {
                            NodeAction::Remove
                        } else {
                            NodeAction::ClearRemove
                        }
                    }
                    "add" => NodeAction::Add,
                    _ => NodeAction::ClearAdd,
                };

                persistent_nodes.persist(
                    original_node_kind,
                    original_node_name,
                    node_kind,
                    OriginalNode::new(action, source_index, child),
                );
            }
        }
    }
}

fn modify_node(
    (original_kind, original): (&str, &str),
    data: &Data,
    output_data: &mut Data,
    shuffle_event_source: SourceIndex,
    system_swaps: &HashMap<&str, &str>,
    persistent_nodes: &HashMap<(&str, &str), OriginalNodes<'_>>,
) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
    let persistent_nodes = persistent_nodes.get(&(original_kind, original)).unwrap();

    let (mut restoration, mut activation) = (None, None);

    for should_activate in [false, true] {
        let mut modified_nodes = vec![];

        for (node_kind, node_values) in persistent_nodes.iter() {
            let mut removed_all = false;

            for node_value in node_values.iter() {
                let is_adding = (should_activate
                    && matches!(node_value.0, NodeAction::Add | NodeAction::ClearAdd))
                    || (!should_activate
                        && matches!(node_value.0, NodeAction::Remove | NodeAction::ClearRemove));

                match *node_kind {
                    "pos" => {
                        let modified_copy = copy_node(
                            data,
                            (node_value.1, node_value.2),
                            output_data,
                            shuffle_event_source,
                            true,
                        )
                        .unwrap();

                        modified_nodes.push(modified_copy);
                    }
                    "jump range" => {
                        let modified_copy = if !is_adding {
                            tree_from_tokens!(
                                output_data; shuffle_event_source =>
                                : "jump range", "0" ;
                            )
                        } else {
                            copy_node(
                                data,
                                (node_value.1, node_value.2),
                                output_data,
                                shuffle_event_source,
                                true,
                            )
                            .unwrap()
                        };

                        modified_nodes.push(modified_copy);
                    }
                    "link" | "unlink" => {
                        if original_kind == "wormhole" && !is_adding {
                            if removed_all {
                                break;
                            }

                            removed_all = true;
                        }

                        let modified_link = if is_adding {
                            match original_kind {
                                "link" | "unlink" => tree_from_tokens!(
                                    output_data; shuffle_event_source =>
                                    : "link" ;
                                ),
                                _ => tree_from_tokens!(
                                    output_data; shuffle_event_source =>
                                    : "add", "link" ;
                                ),
                            }
                        } else {
                            match original_kind {
                                "link" | "unlink" => tree_from_tokens!(
                                    output_data; shuffle_event_source =>
                                    : "unlink" ;
                                ),
                                _ => tree_from_tokens!(
                                    output_data; shuffle_event_source =>
                                    : "remove", "link" ;
                                ),
                            }
                        };

                        if original_kind != "wormhole" || is_adding {
                            for lexeme in data
                                .get_tokens(node_value.2)
                                .unwrap_or_default()
                                .iter()
                                .flat_map(|t| data.get_lexeme(node_value.1, *t))
                                .skip_while(|l| l != node_kind)
                                .skip(1)
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
                        }

                        modified_nodes.push(modified_link);
                    }
                    "object" => {
                        let modified_copy = copy_node(
                            data,
                            (node_value.1, node_value.2),
                            output_data,
                            shuffle_event_source,
                            is_adding,
                        )
                        .unwrap();

                        let (start, end) = output_data
                            .push_source(
                                shuffle_event_source,
                                if is_adding { "add" } else { "remove" },
                            )
                            .unwrap();

                        if let Some(Node::Some { tokens } | Node::Parent { tokens, .. }) =
                            output_data.get_mut_node(modified_copy)
                        {
                            let modified_token =
                                Token::new(TokenKind::Symbol, Span::new(start, end));

                            if matches!(
                                data.get_tokens(node_value.2)
                                    .and_then(|tokens| data.get_lexeme(node_value.1, tokens[0])),
                                Some("add" | "remove")
                            ) && let Some(modifier) = tokens.first_mut()
                            {
                                *modifier = modified_token;
                            } else {
                                tokens.insert(0, modified_token);
                            }
                        }

                        modified_nodes.push(modified_copy);
                    }
                    _ => {
                        let modified_copy = if matches!(
                            (should_activate, node_value.0),
                            (true, NodeAction::Add | NodeAction::ClearAdd)
                                | (false, NodeAction::Remove | NodeAction::ClearRemove)
                        ) {
                            let modified_copy = copy_node(
                                data,
                                (node_value.1, node_value.2),
                                output_data,
                                shuffle_event_source,
                                true,
                            )
                            .unwrap();

                            if let Some("add" | "remove") = output_data.get_lexeme(
                                shuffle_event_source,
                                output_data.get_tokens(modified_copy).unwrap()[0],
                            ) && let Some(Node::Some { tokens } | Node::Parent { tokens, .. }) =
                                output_data.get_mut_node(modified_copy)
                            {
                                tokens.remove(0);
                            }

                            modified_copy
                        } else {
                            tree_from_tokens!(
                                output_data; shuffle_event_source =>
                                : "remove", node_kind ;
                            )
                        };

                        modified_nodes.push(modified_copy);
                    }
                }
            }
        }

        modified_nodes.sort_by(|&a, &b| {
            match (
                output_data
                    .get_tokens(a)
                    .and_then(|tokens| output_data.get_lexeme(shuffle_event_source, tokens[0]))
                    .unwrap(),
                output_data
                    .get_tokens(b)
                    .and_then(|tokens| output_data.get_lexeme(shuffle_event_source, tokens[0]))
                    .unwrap(),
            ) {
                ("pos" | "remove", _) => Ordering::Less,
                (_, "pos" | "remove") => Ordering::Greater,
                (_, _) => Ordering::Equal,
            }
        });

        match should_activate {
            false => restoration = Some(modified_nodes),
            true => activation = Some(modified_nodes),
        }
    }

    (restoration.take().unwrap(), activation.take().unwrap())
}

fn conditional_events(
    settings: &SystemShufflerConfig,
    output_data: &mut Data,
    (source, parent): (SourceIndex, NodeIndex),
    (should_activate, event_name_prefix, label_suffix): (bool, &str, &str),
    persistent_event_node_keys: &[&str],
) {
    for preset_index in 0..=(settings.max_presets) {
        let skip_label = format!("not {preset_index} {label_suffix}");

        let event_branch = tree_from_tokens!(
            output_data; source =>
            : "branch", skip_label.as_str() ;
            {
                : CURRENT_PRESET, "!=", preset_index ;
            }
        );

        output_data.push_child(parent, event_branch);

        let event_action = tree_from_tokens!(
            output_data; source =>
            : "action" ;
            {
                : "event", format!("{event_name_prefix} {preset_index}"), "0" ;
            }
        );

        if should_activate {
            output_data.push_child(parent, event_action);
        }

        for &event_name in persistent_event_node_keys {
            let restore_name = format!("{RESTORE_PREFIX} {preset_index}");
            let activate_name = format!("{ACTIVATE_PREFIX} {preset_index}");

            let skip_label = format!("not {preset_index} {label_suffix} {event_name}");

            let ((condition1, condition2), (action1, action2)) = generate_side_event_conditions(
                event_name,
                (restore_name.as_str(), activate_name.as_str()),
                (should_activate, true),
                output_data,
                source,
            );

            let event_branch = tree_from_tokens!(
                output_data; source =>
                : "branch", skip_label.as_str() ;
            );

            output_data.push_child(parent, event_branch);

            output_data.push_child(event_branch, condition1);
            output_data.push_child(event_branch, condition2);

            let event_action = tree_from_tokens!(
                output_data; source =>
                : "action" ;
            );

            output_data.push_child(parent, event_action);
            output_data.push_child(event_action, action1);
            output_data.push_child(event_action, action2);

            let event_label = tree_from_tokens!(
                output_data; source =>
                : "label", skip_label.as_str() ;
            );

            output_data.push_child(parent, event_label);
        }

        if !should_activate {
            output_data.push_child(parent, event_action);
        }

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
                    : CURRENT_PRESET, "=", CURRENT_PRESET ;
                }
            );

            output_data.push_child(parent, blank_action);
        }
    }
}

fn generate_event<'a>(
    data: &'a Data,
    output_data: &mut Data,
    shuffle_event_source: SourceIndex,
    (shuffle_event_restore, shuffle_event_activate): (NodeIndex, NodeIndex),
    system_swaps: &HashMap<&'a str, &'a str>,
    persistent_nodes: &HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
) {
    let mut persistent_node_keys = persistent_nodes.keys().copied().collect::<Vec<_>>();

    persistent_node_keys.sort();

    for (original_kind, original) in persistent_node_keys {
        let replacement = if original_kind == "system" {
            *system_swaps.get(original).unwrap()
        } else {
            original
        };

        let (removals, additions) = modify_node(
            (original_kind, original),
            data,
            output_data,
            shuffle_event_source,
            system_swaps,
            persistent_nodes,
        );

        // do everything but links first in case `remove link` is one of the removals or additions
        //
        // this block is kind of ugly, in a beautiful way
        let nodes_with_parent = removals
            .iter()
            .filter(|&&node_index| {
                !matches!(
                    output_data
                        .get_tokens(node_index)
                        .and_then(|tokens| output_data.get_lexeme(shuffle_event_source, tokens[0])),
                    Some("link" | "unlink")
                )
            })
            .map(|&node_index| (false, node_index))
            .chain(
                additions
                    .iter()
                    .filter(|&&node_index| {
                        !matches!(
                            output_data
                                .get_tokens(node_index)
                                .and_then(|tokens| output_data
                                    .get_lexeme(shuffle_event_source, tokens[0])),
                            Some("link" | "unlink")
                        )
                    })
                    .map(|&node_index| (true, node_index)),
            )
            .collect::<Vec<_>>();

        if !nodes_with_parent.is_empty() {
            let parent_restore = tree_from_tokens!(
                output_data; shuffle_event_source =>
                : original_kind, replacement ;
            );

            let parent_activate = tree_from_tokens!(
                output_data; shuffle_event_source =>
                : original_kind, replacement ;
            );

            for (activate, modification) in nodes_with_parent {
                output_data.push_child(
                    if activate {
                        parent_activate
                    } else {
                        parent_restore
                    },
                    modification,
                );
            }

            output_data.push_child(shuffle_event_restore, parent_restore);
            output_data.push_child(shuffle_event_activate, parent_activate);
        }

        // copy and paste for now, I can always make it better later
        // TODO: don't copy and paste
        for (modification_parent, modification) in removals
            .iter()
            .filter(|&&node_index| {
                matches!(
                    output_data
                        .get_tokens(node_index)
                        .and_then(|tokens| output_data.get_lexeme(shuffle_event_source, tokens[0])),
                    Some("link" | "unlink")
                )
            })
            .map(|&node_index| (shuffle_event_restore, node_index))
            .chain(
                additions
                    .iter()
                    .filter(|&&node_index| {
                        matches!(
                            output_data
                                .get_tokens(node_index)
                                .and_then(|tokens| output_data
                                    .get_lexeme(shuffle_event_source, tokens[0])),
                            Some("link" | "unlink")
                        )
                    })
                    .map(|&node_index| (shuffle_event_activate, node_index)),
            )
            .collect::<Vec<_>>()
        {
            output_data.push_child(modification_parent, modification);
        }
    }
}

fn generate_side_event_conditions(
    event_name: &str,
    (restore_name, activate_name): (&str, &str),
    (should_activate, invert): (bool, bool),
    output_data: &mut Data,
    shuffle_event_source: SourceIndex,
) -> ((NodeIndex, NodeIndex), (NodeIndex, NodeIndex)) {
    let condition1 = tree_from_tokens!(
        output_data; shuffle_event_source =>
        :
            match invert {
                false => "has",
                true => "not",
            },
            format!("event: {event_name}")
        ;
    );

    match should_activate {
        false => {
            let condition2 = tree_from_tokens!(
                output_data; shuffle_event_source =>
                :
                    match invert {
                        false => "has",
                        true => "not",
                    },
                    format!("event: {activate_name}: {event_name}")
                ;
            );

            let action1 = tree_from_tokens!(
                output_data; shuffle_event_source =>
                : "event", format!("{restore_name}: {event_name}"), "0" ;
            );

            let action2 = tree_from_tokens!(
                output_data; shuffle_event_source =>
                : format!("event: {activate_name}: {event_name}"), "=", "0" ;
            );

            ((condition1, condition2), (action1, action2))
        }
        true => {
            let condition2 = tree_from_tokens!(
                output_data; shuffle_event_source =>
                :
                    format!("event: {activate_name}: {event_name}"),
                    match invert {
                        false => "!=",
                        true => "==",
                    },
                    format!("event: {event_name}")
                ;
            );

            let action1 = tree_from_tokens!(
                output_data; shuffle_event_source =>
                : "event", format!("{activate_name}: {event_name}"), "0" ;
            );

            let action2 = tree_from_tokens!(
                output_data; shuffle_event_source =>
                : format!("event: {activate_name}: {event_name}"), "=", format!("event: {event_name}") ;
            );

            ((condition1, condition2), (action1, action2))
        }
    }
}
