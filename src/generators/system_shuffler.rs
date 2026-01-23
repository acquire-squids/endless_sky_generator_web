use crate::generators;
use crate::wandom::ShuffleIndex;
use crate::zippy::Zip;

use endless_sky_rw::{
    Data, Node, NodeIndex, SourceIndex, Span, Token, TokenKind, node_path_iter, tree_from_tokens,
};

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
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
    #[allow(clippy::missing_const_for_fn)]
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

struct SystemShuffler<'a> {
    archive: Zip<'a>,
    output_data: Data,
    settings: SystemShufflerConfig,
}

pub fn process(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: SystemShufflerConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = generators::read_upload(paths, sources)?;

    let data = data_folder.data();

    let mut output = vec![];

    let mut generator = SystemShuffler {
        archive: Zip::new(&mut output),
        output_data: Data::default(),
        settings,
    };

    generator.description()?;

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

    let persistent_event_nodes =
        find_persistent_event_nodes(data, &mut system_names, &mut planets, &mut wormholes);

    let mut system_names = system_names.into_iter().collect::<Vec<_>>();

    system_names.sort_unstable();

    let mut persistent_event_node_keys = persistent_event_nodes.keys().copied().collect::<Vec<_>>();

    // TODO: sort events by when they happen chronologically?
    persistent_event_node_keys.sort_unstable();

    generator.archive.write_dir("data/")?;

    generator.main_data(persistent_event_node_keys.as_slice())?;

    generator.archive.write_dir("data/presets/")?;

    for preset_index in 0..=(settings.max_presets) {
        generator.preset(
            data,
            preset_index,
            system_names.as_slice(),
            &persistent_nodes,
            (&persistent_event_node_keys, &persistent_event_nodes),
        )?;
    }

    generator.archive.finish()?;

    Ok(output)
}

impl SystemShuffler<'_> {
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
            if self.settings.shuffle_once_on_install {
                "In addition to shuffling once immediately upon installation, this plugin was generated with the following settings:\n"
            } else {
                "This plugin was generated with the following settings:\n"
            },
            self.settings.seed,
            self.settings.max_presets,
            if self.settings.shuffle_chance > 0 {
                format!(
                    "- A {}% chance to shuffle to a different preset every time you land\n",
                    self.settings.shuffle_chance
                )
            } else {
                String::new()
            },
            if self.settings.fixed_shuffle_days > 0 {
                format!(
                    "- A guaranteed shuffle roughly once every {} days\n",
                    self.settings.fixed_shuffle_days
                )
            } else {
                String::new()
            },
        );

        let output_root_node_count = self.output_data.root_nodes().len();
        let plugin_txt_source = self.output_data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut self.output_data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        self.output_data
            .push_root_node(plugin_txt_source, plugin_name);

        for about in plugin_description_txt.lines().map(str::trim) {
            if !about.is_empty() {
                let plugin_about = tree_from_tokens!(
                    &mut self.output_data; plugin_txt_source =>
                    : "about", about ;
                );

                self.output_data
                    .push_root_node(plugin_txt_source, plugin_about);
            }
        }

        let plugin_version = tree_from_tokens!(
            &mut self.output_data; plugin_txt_source =>
            : "version", PLUGIN_VERSION ;
        );

        self.output_data
            .push_root_node(plugin_txt_source, plugin_version);

        self.zip_root_nodes("plugin.txt", output_root_node_count)
    }

    fn main_data(&mut self, persistent_event_node_keys: &[&str]) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        self.main_mission(persistent_event_node_keys);

        self.restore_job(persistent_event_node_keys);

        self.manual_trigger(persistent_event_node_keys);

        self.zip_root_nodes("data/main.txt", output_root_node_count)
    }

    fn main_mission(&mut self, persistent_event_node_keys: &[&str]) {
        let main_mission_source = self.output_data.insert_source(String::new());

        let main_mission = tree_from_tokens!(
            &mut self.output_data; main_mission_source =>
            : "mission", "zzzzz System Shuffler: Select Preset" ;
            {
                : "invisible" ;
                : "repeat" ;
                : "non-blocking" ;
                : "landing" ;
                : "offer precedence", "-1000000" ;
            }
        );

        self.output_data
            .push_root_node(main_mission_source, main_mission);

        let main_mission_to_offer = tree_from_tokens!(
            &mut self.output_data; main_mission_source =>
            : "to", "offer" ;
        );

        self.output_data
            .push_child(main_mission, main_mission_to_offer);

        if self.settings.shuffle_chance == 0
            && self.settings.fixed_shuffle_days == 0
            && !self.settings.shuffle_once_on_install
        {
            let main_mission_never = tree_from_tokens!(
                &mut self.output_data; main_mission_source =>
                : "never" ;
            );

            self.output_data
                .push_child(main_mission_to_offer, main_mission_never);
        } else {
            let main_mission_to_offer_or = tree_from_tokens!(
                &mut self.output_data; main_mission_source =>
                : "or" ;
            );

            self.output_data
                .push_child(main_mission_to_offer, main_mission_to_offer_or);

            if self.settings.shuffle_chance > 0 {
                let random_chance = tree_from_tokens!(
                    &mut self.output_data; main_mission_source =>
                    : "random", "<", self.settings.shuffle_chance ;
                );

                self.output_data
                    .push_child(main_mission_to_offer_or, random_chance);
            }

            if self.settings.fixed_shuffle_days > 0 {
                let guaranteed = tree_from_tokens!(
                    &mut self.output_data; main_mission_source =>
                    : "days since epoch", ">=", "(", LAST_SHUFFLE_DAY, "+", self.settings.fixed_shuffle_days, ")" ;
                );

                self.output_data
                    .push_child(main_mission_to_offer_or, guaranteed);
            }

            if self.settings.shuffle_once_on_install {
                let first_time = tree_from_tokens!(
                    &mut self.output_data; main_mission_source =>
                    : "not", INSTALLED ;
                );

                self.output_data
                    .push_child(main_mission_to_offer_or, first_time);
            }
        }

        let main_mission_on_offer = tree_from_tokens!(
            &mut self.output_data; main_mission_source =>
            : "on", "offer" ;
        );

        self.output_data
            .push_child(main_mission, main_mission_on_offer);

        let main_mission_conversation = tree_from_tokens!(
            &mut self.output_data; main_mission_source =>
            : "conversation" ;
            {
                : "The universe has shuffled. Good luck." ;
            }
        );

        self.output_data
            .push_child(main_mission_on_offer, main_mission_conversation);

        self.conditional_events(
            (main_mission_source, main_mission_conversation),
            (false, RESTORE_PREFIX, "restore"),
            persistent_event_node_keys,
        );

        let preset_selection = self.select_preset(main_mission_source, false);

        self.output_data
            .push_child(main_mission_conversation, preset_selection);

        self.conditional_events(
            (main_mission_source, main_mission_conversation),
            (true, ACTIVATE_PREFIX, "activate"),
            persistent_event_node_keys,
        );

        let main_failure = tree_from_tokens!(
            &mut self.output_data; main_mission_source =>
            : "fail" ;
        );

        self.output_data
            .push_child(main_mission_on_offer, main_failure);
    }

    fn select_preset(&mut self, source: SourceIndex, reset: bool) -> NodeIndex {
        if reset {
            tree_from_tokens!(
                &mut self.output_data; source =>
                : "action" ;
                {
                    : INSTALLED, "=", "1" ;
                    : CURRENT_PRESET, "=", "0" ;
                    : LAST_SHUFFLE_DAY, "=", "days since epoch" ;
                }
            )
        } else {
            tree_from_tokens!(
                &mut self.output_data; source =>
                : "action" ;
                {
                    : INSTALLED, "=", "1" ;
                    : CURRENT_PRESET, "=", "(", format!("roll: {}", self.settings.max_presets).as_str(), "+", "1", ")" ;
                    : LAST_SHUFFLE_DAY, "=", "days since epoch" ;
                }
            )
        }
    }

    fn restore_job(&mut self, persistent_event_node_keys: &[&str]) {
        let restore_job_source = self.output_data.insert_source(String::new());

        let restore_job = tree_from_tokens!(
            &mut self.output_data; restore_job_source =>
            : "mission", "zzzzz System Shuffler: Restore Universe" ;
            {
                : "name", "Unshuffle the universe" ;
                : "description", "Restore all systems in the universe to how they should be, free of charge." ;
                : "repeat" ;
                : "job" ;
            }
        );

        self.output_data
            .push_root_node(restore_job_source, restore_job);

        let restore_job_to_offer = tree_from_tokens!(
            &mut self.output_data; restore_job_source =>
            : "to", "offer" ;
            {
                : CURRENT_PRESET, "!=", "0" ;
            }
        );

        self.output_data
            .push_child(restore_job, restore_job_to_offer);

        let restore_job_on_accept = tree_from_tokens!(
            &mut self.output_data; restore_job_source =>
            : "on", "accept" ;
        );

        self.output_data
            .push_child(restore_job, restore_job_on_accept);

        let restore_job_conversation = tree_from_tokens!(
            &mut self.output_data; restore_job_source =>
            : "conversation" ;
            {
                : "As per your request, the universe has been restored." ;
            }
        );

        self.output_data
            .push_child(restore_job_on_accept, restore_job_conversation);

        self.conditional_events(
            (restore_job_source, restore_job_conversation),
            (false, RESTORE_PREFIX, "restore"),
            persistent_event_node_keys,
        );

        let preset_selection = self.select_preset(restore_job_source, true);

        self.output_data
            .push_child(restore_job_conversation, preset_selection);

        self.conditional_events(
            (restore_job_source, restore_job_conversation),
            (true, ACTIVATE_PREFIX, "activate"),
            persistent_event_node_keys,
        );

        let main_failure = tree_from_tokens!(
            &mut self.output_data; restore_job_source =>
            : "fail" ;
        );

        self.output_data
            .push_child(restore_job_on_accept, main_failure);
    }

    fn manual_trigger(&mut self, persistent_event_node_keys: &[&str]) {
        let manual_job_source = self.output_data.insert_source(String::new());

        let manual_job = tree_from_tokens!(
            &mut self.output_data; manual_job_source =>
            : "mission", "zzzzz System Shuffler: Manual Shuffle" ;
            {
                : "name", "Shuffle the universe" ;
                : "description", format!("Shuffle all systems in the universe to one of {} presets.", self.settings.max_presets).as_str() ;
                : "repeat" ;
                : "job" ;
            }
        );

        self.output_data
            .push_root_node(manual_job_source, manual_job);

        let manual_job_on_accept = tree_from_tokens!(
            &mut self.output_data; manual_job_source =>
            : "on", "accept" ;
        );

        self.output_data
            .push_child(manual_job, manual_job_on_accept);

        let manual_job_conversation = tree_from_tokens!(
            &mut self.output_data; manual_job_source =>
            : "conversation" ;
            {
                : "As per your request, the universe has shuffled. Good luck." ;
            }
        );

        self.output_data
            .push_child(manual_job_on_accept, manual_job_conversation);

        self.conditional_events(
            (manual_job_source, manual_job_conversation),
            (false, RESTORE_PREFIX, "restore"),
            persistent_event_node_keys,
        );

        let preset_selection = self.select_preset(manual_job_source, false);

        self.output_data
            .push_child(manual_job_conversation, preset_selection);

        self.conditional_events(
            (manual_job_source, manual_job_conversation),
            (true, ACTIVATE_PREFIX, "activate"),
            persistent_event_node_keys,
        );

        let main_failure = tree_from_tokens!(
            &mut self.output_data; manual_job_source =>
            : "fail" ;
        );

        self.output_data
            .push_child(manual_job_on_accept, main_failure);
    }

    fn preset(
        &mut self,
        data: &Data,
        preset_index: u8,
        system_names: &[&str],
        persistent_nodes: &PersistentOriginalNodes<'_>,
        (persistent_event_node_keys, persistent_event_nodes): (
            &[&str],
            &HashMap<&str, PersistentOriginalNodes<'_>>,
        ),
    ) -> Result<(), Box<dyn Error>> {
        let preset_index = usize::from(preset_index);

        let shuffle_event_source = self.output_data.insert_source(String::new());

        let system_swaps = self.get_system_swaps(system_names, preset_index);

        let preset_path = format!("data/presets/universe_preset_{preset_index}");

        self.archive.write_dir(format!("{preset_path}/"))?;

        let restore_name = format!("{RESTORE_PREFIX} {preset_index}");
        let activate_name = format!("{ACTIVATE_PREFIX} {preset_index}");

        {
            let output_root_node_count = self.output_data.root_nodes().len();

            self.preset_event(
                data,
                shuffle_event_source,
                &system_swaps,
                persistent_nodes,
                (restore_name.as_str(), activate_name.as_str()),
            );

            self.zip_root_nodes(format!("{preset_path}/main.txt"), output_root_node_count)?;
        }

        {
            let output_root_node_count = self.output_data.root_nodes().len();

            for (event_name, event_map) in persistent_event_node_keys.iter().map(|&e| {
                (
                    e,
                    persistent_event_nodes
                        .get(e)
                        .expect("An event node should never be in the keys list if it is not real"),
                )
            }) {
                self.preset_event(
                    data,
                    shuffle_event_source,
                    &system_swaps,
                    event_map,
                    (
                        format!("{restore_name}: {event_name}").as_str(),
                        format!("{activate_name}: {event_name}").as_str(),
                    ),
                );
            }

            self.zip_root_nodes(format!("{preset_path}/events.txt"), output_root_node_count)?;
        }

        {
            let output_root_node_count = self.output_data.root_nodes().len();

            for &event_name in persistent_event_nodes.keys() {
                self.backpatch_mission(
                    shuffle_event_source,
                    preset_index,
                    event_name,
                    (restore_name.as_str(), activate_name.as_str()),
                );
            }

            self.zip_root_nodes(
                format!("{preset_path}/missions.txt"),
                output_root_node_count,
            )
        }
    }

    fn get_system_swaps<'a>(
        &self,
        system_names: &[&'a str],
        preset_index: usize,
    ) -> HashMap<&'a str, &'a str> {
        let shuffled = if preset_index == 0 {
            (0..(system_names.len()))
                .map(|i| system_names[i])
                .collect::<Vec<_>>()
        } else {
            system_names
                .shuffled_indices(self.settings.seed.wrapping_add(preset_index))
                .into_iter()
                .map(|i| system_names[i])
                .collect::<Vec<_>>()
        };

        system_names
            .iter()
            .copied()
            .zip(shuffled)
            .collect::<HashMap<_, _>>()
    }

    fn preset_event(
        &mut self,
        data: &Data,
        shuffle_event_source: SourceIndex,
        system_swaps: &HashMap<&str, &str>,
        persistent_nodes: &HashMap<(&str, &str), OriginalNodes<'_>>,
        (restore_name, activate_name): (&str, &str),
    ) {
        let shuffle_event_restore = tree_from_tokens!(
            &mut self.output_data; shuffle_event_source =>
            : "event", restore_name ;
        );

        self.output_data
            .push_root_node(shuffle_event_source, shuffle_event_restore);

        let shuffle_event_activate = tree_from_tokens!(
            &mut self.output_data; shuffle_event_source =>
            : "event", activate_name ;
        );

        self.output_data
            .push_root_node(shuffle_event_source, shuffle_event_activate);

        self.event(
            data,
            shuffle_event_source,
            (shuffle_event_restore, shuffle_event_activate),
            system_swaps,
            persistent_nodes,
        );
    }

    fn backpatch_mission(
        &mut self,
        shuffle_event_source: SourceIndex,
        preset_index: usize,
        event_name: &str,
        (restore_name, activate_name): (&str, &str),
    ) {
        for (should_activate, kind_name) in [(false, restore_name), (true, activate_name)] {
            let shuffle_mission = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "mission", format!("zzzzz {kind_name}: {event_name}") ;
                {
                    : "invisible" ;
                    : "repeat" ;
                    : "non-blocking" ;
                    : "landing" ;
                    : "offer precedence", "-1000000" ;
                }
            );

            self.output_data
                .push_root_node(shuffle_event_source, shuffle_mission);

            let mission_to_offer = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "to", "offer" ;
                {
                    : "has", INSTALLED ;
                    :
                        CURRENT_PRESET,
                        if should_activate {
                            "=="
                        } else {
                            "!="
                        },
                        preset_index
                    ;
                }
            );

            self.output_data
                .push_child(shuffle_mission, mission_to_offer);

            let mission_on_offer = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "on", "offer" ;
            );

            self.output_data
                .push_child(shuffle_mission, mission_on_offer);

            let ((condition1, condition2), (action1, action2)) = self.side_event_conditions(
                event_name,
                (restore_name, activate_name),
                (should_activate, false),
                shuffle_event_source,
            );

            self.output_data.push_child(mission_to_offer, condition1);
            self.output_data.push_child(mission_to_offer, condition2);

            self.output_data.push_child(mission_on_offer, action1);
            self.output_data.push_child(mission_on_offer, action2);

            let mission_failure = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "fail" ;
            );

            self.output_data
                .push_child(mission_on_offer, mission_failure);
        }
    }

    fn conditional_events(
        &mut self,
        (source, parent): (SourceIndex, NodeIndex),
        (should_activate, event_name_prefix, label_suffix): (bool, &str, &str),
        persistent_event_node_keys: &[&str],
    ) {
        for preset_index in 0..=(self.settings.max_presets) {
            let skip_label = format!("not {preset_index} {label_suffix}");

            let event_branch = tree_from_tokens!(
                &mut self.output_data; source =>
                : "branch", skip_label.as_str() ;
                {
                    : CURRENT_PRESET, "!=", preset_index ;
                }
            );

            self.output_data.push_child(parent, event_branch);

            let event_action = tree_from_tokens!(
                &mut self.output_data; source =>
                : "action" ;
                {
                    : "event", format!("{event_name_prefix} {preset_index}"), "0" ;
                }
            );

            if should_activate {
                self.output_data.push_child(parent, event_action);
            }

            for &event_name in persistent_event_node_keys {
                let restore_name = format!("{RESTORE_PREFIX} {preset_index}");
                let activate_name = format!("{ACTIVATE_PREFIX} {preset_index}");

                let skip_label = format!("not {preset_index} {label_suffix} {event_name}");

                let ((condition1, condition2), (action1, action2)) = self.side_event_conditions(
                    event_name,
                    (restore_name.as_str(), activate_name.as_str()),
                    (should_activate, true),
                    source,
                );

                let event_branch = tree_from_tokens!(
                    &mut self.output_data; source =>
                    : "branch", skip_label.as_str() ;
                );

                self.output_data.push_child(parent, event_branch);

                self.output_data.push_child(event_branch, condition1);
                self.output_data.push_child(event_branch, condition2);

                let event_action = tree_from_tokens!(
                    &mut self.output_data; source =>
                    : "action" ;
                );

                self.output_data.push_child(parent, event_action);
                self.output_data.push_child(event_action, action1);
                self.output_data.push_child(event_action, action2);

                let event_label = tree_from_tokens!(
                    &mut self.output_data; source =>
                    : "label", skip_label.as_str() ;
                );

                self.output_data.push_child(parent, event_label);
            }

            if !should_activate {
                self.output_data.push_child(parent, event_action);
            }

            let event_label = tree_from_tokens!(
                &mut self.output_data; source =>
                : "label", skip_label.as_str() ;
            );

            self.output_data.push_child(parent, event_label);

            if preset_index == self.settings.max_presets {
                let blank_action = tree_from_tokens!(
                    &mut self.output_data; source =>
                    : "action" ;
                    {
                        : CURRENT_PRESET, "=", CURRENT_PRESET ;
                    }
                );

                self.output_data.push_child(parent, blank_action);
            }
        }
    }

    fn event<'a>(
        &mut self,
        data: &'a Data,
        shuffle_event_source: SourceIndex,
        (shuffle_event_restore, shuffle_event_activate): (NodeIndex, NodeIndex),
        system_swaps: &HashMap<&'a str, &'a str>,
        persistent_nodes: &HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
    ) {
        let mut persistent_node_keys = persistent_nodes.keys().copied().collect::<Vec<_>>();

        persistent_node_keys.sort_unstable();

        for (original_kind, original) in persistent_node_keys {
            let replacement = system_swaps.get(original).map_or(original, |swap| swap);

            let (removals, additions) = self.modify_node(
                (original_kind, original),
                data,
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
                        self.output_data
                            .get_tokens(node_index)
                            .and_then(|tokens| self
                                .output_data
                                .get_lexeme(shuffle_event_source, tokens[0])),
                        Some("link" | "unlink")
                    )
                })
                .map(|&node_index| (false, node_index))
                .chain(
                    additions
                        .iter()
                        .filter(|&&node_index| {
                            !matches!(
                                self.output_data
                                    .get_tokens(node_index)
                                    .and_then(|tokens| self
                                        .output_data
                                        .get_lexeme(shuffle_event_source, tokens[0])),
                                Some("link" | "unlink")
                            )
                        })
                        .map(|&node_index| (true, node_index)),
                )
                .collect::<Vec<_>>();

            if !nodes_with_parent.is_empty() {
                let parent_restore = tree_from_tokens!(
                    &mut self.output_data; shuffle_event_source =>
                    : original_kind, replacement ;
                );

                let parent_activate = tree_from_tokens!(
                    &mut self.output_data; shuffle_event_source =>
                    : original_kind, replacement ;
                );

                for (activate, modification) in nodes_with_parent {
                    self.output_data.push_child(
                        if activate {
                            parent_activate
                        } else {
                            parent_restore
                        },
                        modification,
                    );
                }

                self.output_data
                    .push_child(shuffle_event_restore, parent_restore);
                self.output_data
                    .push_child(shuffle_event_activate, parent_activate);
            }

            // copy and paste for now, I can always make it better later
            // TODO: don't copy and paste
            //
            // TODO: don't collect?
            #[allow(clippy::needless_collect)]
            for (modification_parent, modification) in removals
                .iter()
                .filter(|&&node_index| {
                    matches!(
                        self.output_data
                            .get_tokens(node_index)
                            .and_then(|tokens| self
                                .output_data
                                .get_lexeme(shuffle_event_source, tokens[0])),
                        Some("link" | "unlink")
                    )
                })
                .map(|&node_index| (shuffle_event_restore, node_index))
                .chain(
                    additions
                        .iter()
                        .filter(|&&node_index| {
                            matches!(
                                self.output_data
                                    .get_tokens(node_index)
                                    .and_then(|tokens| self
                                        .output_data
                                        .get_lexeme(shuffle_event_source, tokens[0])),
                                Some("link" | "unlink")
                            )
                        })
                        .map(|&node_index| (shuffle_event_activate, node_index)),
                )
                .collect::<Vec<_>>()
            {
                self.output_data
                    .push_child(modification_parent, modification);
            }
        }
    }

    fn side_event_conditions(
        &mut self,
        event_name: &str,
        (restore_name, activate_name): (&str, &str),
        (should_activate, invert): (bool, bool),
        shuffle_event_source: SourceIndex,
    ) -> ((NodeIndex, NodeIndex), (NodeIndex, NodeIndex)) {
        let condition1 = tree_from_tokens!(
            &mut self.output_data; shuffle_event_source =>
            :
                if invert {
                    "not"
                } else {
                    "has"
                },
                format!("event: {event_name}")
            ;
        );

        if should_activate {
            let condition2 = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                :
                    format!("event: {activate_name}: {event_name}"),
                    if invert {
                        "=="
                    } else {
                        "!="
                    },
                    format!("event: {event_name}")
                ;
            );

            let action1 = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "event", format!("{activate_name}: {event_name}"), "0" ;
            );

            let action2 = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : format!("event: {activate_name}: {event_name}"), "=", format!("event: {event_name}") ;
            );

            ((condition1, condition2), (action1, action2))
        } else {
            let condition2 = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                :
                    if invert {
                        "not"
                    } else {
                        "has"
                    },
                    format!("event: {activate_name}: {event_name}")
                ;
            );

            let action1 = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "event", format!("{restore_name}: {event_name}"), "0" ;
            );

            let action2 = tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : format!("event: {activate_name}: {event_name}"), "=", "0" ;
            );

            ((condition1, condition2), (action1, action2))
        }
    }

    fn modify_node(
        &mut self,
        (original_kind, original): (&str, &str),
        data: &Data,
        shuffle_event_source: SourceIndex,
        system_swaps: &HashMap<&str, &str>,
        persistent_nodes: &HashMap<(&str, &str), OriginalNodes<'_>>,
    ) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
        let persistent_nodes = persistent_nodes
            .get(&(original_kind, original))
            .expect("Data must be verified in previous steps");

        let (mut restoration, mut activation) = (None, None);

        for should_activate in [false, true] {
            let mut modified_nodes = vec![];

            for (node_kind, node_values) in persistent_nodes {
                let mut removed_all = false;

                for node_value in node_values {
                    let is_adding = (should_activate
                        && matches!(node_value.0, NodeAction::Add | NodeAction::ClearAdd))
                        || (!should_activate
                            && matches!(
                                node_value.0,
                                NodeAction::Remove | NodeAction::ClearRemove
                            ));

                    match *node_kind {
                        "pos" => {
                            modified_nodes.push(self.modify_pos(
                                data,
                                node_value,
                                shuffle_event_source,
                            ));
                        }
                        "jump range" => {
                            modified_nodes.push(self.modify_jump_range(
                                data,
                                node_value,
                                shuffle_event_source,
                                is_adding,
                            ));
                        }
                        "link" | "unlink" => {
                            if original_kind == "wormhole" && !is_adding {
                                if removed_all {
                                    break;
                                }

                                removed_all = true;
                            }

                            modified_nodes.push(self.modify_link(
                                (node_kind, original_kind),
                                data,
                                node_value,
                                shuffle_event_source,
                                is_adding,
                                system_swaps,
                            ));
                        }
                        "object" => {
                            modified_nodes.push(self.modify_object(
                                data,
                                node_value,
                                shuffle_event_source,
                                is_adding,
                            ));
                        }
                        _ => {
                            modified_nodes.push(self.modify_other(
                                node_kind,
                                data,
                                node_value,
                                shuffle_event_source,
                                is_adding,
                            ));
                        }
                    }
                }
            }

            modified_nodes.sort_by(|&a, &b| {
                match (
                    self.output_data
                        .get_tokens(a)
                        .and_then(|tokens| tokens.first())
                        .and_then(|token| self.output_data.get_lexeme(shuffle_event_source, *token))
                        .expect("Only nodes with at least one token should be modified"),
                    self.output_data
                        .get_tokens(b)
                        .and_then(|tokens| tokens.first())
                        .and_then(|token| self.output_data.get_lexeme(shuffle_event_source, *token))
                        .expect("Only nodes with at least one token should be modified"),
                ) {
                    ("pos" | "remove", _) => Ordering::Less,
                    (_, "pos" | "remove") => Ordering::Greater,
                    (_, _) => Ordering::Equal,
                }
            });

            if should_activate {
                activation = Some(modified_nodes);
            } else {
                restoration = Some(modified_nodes);
            }
        }

        (
            restoration
                .take()
                .expect("Data must be verified in previous steps"),
            activation
                .take()
                .expect("Data must be verified in previous steps"),
        )
    }

    fn modify_pos(
        &mut self,
        data: &Data,
        node_value: &(NodeAction, SourceIndex, NodeIndex),
        shuffle_event_source: SourceIndex,
    ) -> NodeIndex {
        copy_node(
            data,
            (node_value.1, node_value.2),
            &mut self.output_data,
            shuffle_event_source,
            true,
        )
        .expect("Pos data must be verified in previous steps")
    }

    fn modify_jump_range(
        &mut self,
        data: &Data,
        node_value: &(NodeAction, SourceIndex, NodeIndex),
        shuffle_event_source: SourceIndex,
        is_adding: bool,
    ) -> NodeIndex {
        if is_adding {
            copy_node(
                data,
                (node_value.1, node_value.2),
                &mut self.output_data,
                shuffle_event_source,
                true,
            )
            .expect("Jump range data must be verified in previous steps")
        } else {
            tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "jump range", "0" ;
            )
        }
    }

    fn modify_link(
        &mut self,
        (node_kind, original_kind): (&str, &str),
        data: &Data,
        node_value: &(NodeAction, SourceIndex, NodeIndex),
        shuffle_event_source: SourceIndex,
        is_adding: bool,
        system_swaps: &HashMap<&str, &str>,
    ) -> NodeIndex {
        let modified_link = if is_adding {
            match original_kind {
                "link" | "unlink" => tree_from_tokens!(
                    &mut self.output_data; shuffle_event_source =>
                    : "link" ;
                ),
                _ => tree_from_tokens!(
                    &mut self.output_data; shuffle_event_source =>
                    : "add", "link" ;
                ),
            }
        } else {
            match original_kind {
                "link" | "unlink" => tree_from_tokens!(
                    &mut self.output_data; shuffle_event_source =>
                    : "unlink" ;
                ),
                _ => tree_from_tokens!(
                    &mut self.output_data; shuffle_event_source =>
                    : "remove", "link" ;
                ),
            }
        };

        if original_kind != "wormhole" || is_adding {
            for lexeme in data
                .get_tokens(node_value.2)
                .unwrap_or_default()
                .iter()
                .filter_map(|t| data.get_lexeme(node_value.1, *t))
                .skip_while(|l| *l != node_kind)
                .skip(1)
            {
                let (start, end) = self
                    .output_data
                    .push_source(
                        shuffle_event_source,
                        system_swaps
                            .get(lexeme)
                            .expect("Link data must be verified in previous steps"),
                    )
                    .expect("Link data must be verified in previous steps");

                self.output_data.push_token(
                    modified_link,
                    Token::new(TokenKind::Symbol, Span::new(start, end)),
                );
            }
        }

        modified_link
    }

    fn modify_object(
        &mut self,
        data: &Data,
        node_value: &(NodeAction, SourceIndex, NodeIndex),
        shuffle_event_source: SourceIndex,
        is_adding: bool,
    ) -> NodeIndex {
        let modified_object = copy_node(
            data,
            (node_value.1, node_value.2),
            &mut self.output_data,
            shuffle_event_source,
            is_adding,
        )
        .expect("Object data must be verified in previous steps");

        let (start, end) = self
            .output_data
            .push_source(
                shuffle_event_source,
                if is_adding { "add" } else { "remove" },
            )
            .expect("Object data must be verified in previous steps");

        if let Some(Node::Some { tokens } | Node::Parent { tokens, .. }) =
            self.output_data.get_mut_node(modified_object)
        {
            let modified_token = Token::new(TokenKind::Symbol, Span::new(start, end));

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

        modified_object
    }

    fn modify_other(
        &mut self,
        node_kind: &str,
        data: &Data,
        node_value: &(NodeAction, SourceIndex, NodeIndex),
        shuffle_event_source: SourceIndex,
        is_adding: bool,
    ) -> NodeIndex {
        if is_adding {
            let modified_copy = copy_node(
                data,
                (node_value.1, node_value.2),
                &mut self.output_data,
                shuffle_event_source,
                true,
            )
            .expect("Data must be verified in previous steps");

            if matches!(
                self.output_data
                    .get_tokens(modified_copy)
                    .and_then(|tokens| tokens.first().and_then(|token| self
                        .output_data
                        .get_lexeme(shuffle_event_source, *token))),
                Some("add" | "remove")
            ) && let Some(Node::Some { tokens } | Node::Parent { tokens, .. }) =
                self.output_data.get_mut_node(modified_copy)
            {
                tokens.remove(0);
            }

            modified_copy
        } else {
            tree_from_tokens!(
                &mut self.output_data; shuffle_event_source =>
                : "remove", node_kind ;
            )
        }
    }
}

fn find_wormholes_from_planets<'a>(data: &'a Data, wormholes: &mut HashSet<&'a str>) {
    node_path_iter!(data; "planet")
        .filter(|(source_index, node_index)| {
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
        .for_each(|(source_index, node_index)| {
            wormholes.insert(
                data.get_tokens(node_index)
                    .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
                    .expect("The iterator should have a filter applied to ensure only nodes with at least two tokens make it here"),
            );
        });
}

fn find_wormholes_from_system<'a>(
    data: &'a Data,
    (system_name, source_index, node_index): (&'a str, SourceIndex, NodeIndex),
    (depth, planets, wormholes): (u64, &mut HashMap<&'a str, &'a str>, &mut HashSet<&'a str>),
    persistent_nodes: &mut HashMap<(&'a str, &'a str), OriginalNodes<'a>>,
) -> bool {
    data.filter_children(source_index, node_index, |source_index, tokens| {
        let key_index = usize::from(matches!(
            tokens
                .first()
                .and_then(|t| data.get_lexeme(source_index, *t)),
            Some("remove" | "add")
        ));

        matches!(
            tokens
                .get(key_index)
                .and_then(|t| data.get_lexeme(source_index, *t)),
            Some(l) if l == "object"
        )
    })
    .fold(false, |any_is_wormhole, child| {
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
                .expect("The iterator should use a filter to ensure only nodes with at least one token (or two, if starting with add/remove) are allowed")
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

        any_is_wormhole || is_wormhole
    })
}

fn find_persistent_event_nodes<'a>(
    data: &'a Data,
    system_names: &mut HashSet<&'a str>,
    planets: &mut HashMap<&'a str, &'a str>,
    wormholes: &mut HashSet<&'a str>,
) -> HashMap<&'a str, HashMap<(&'a str, &'a str), OriginalNodes<'a>>> {
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
            .expect("The iterator should have a filter to ensure only nodes with at least two tokens make it here");

        let mut event_map = HashMap::new();

        data_from_node(
            data,
            node_path_iter!(&data => (source_index, node_index); "system" | "wormhole" | "link" | "unlink")
                .map(|n| (source_index, n)),
            system_names,
            (planets, wormholes),
            &mut event_map,
        );

        if !event_map.is_empty() {
            persistent_event_nodes.insert(event_name, event_map);
        }
    }

    persistent_event_nodes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeAction {
    Remove,
    ClearRemove,
    Add,
    ClearAdd,
}

type PersistentOriginalNodes<'a> = HashMap<(&'a str, &'a str), OriginalNodes<'a>>;

type OriginalNodes<'a> = HashMap<&'a str, Vec<(NodeAction, SourceIndex, NodeIndex)>>;

struct OriginalNode {
    action: NodeAction,
    source: SourceIndex,
    node: NodeIndex,
}

impl OriginalNode {
    const fn new(action: NodeAction, source: SourceIndex, node: NodeIndex) -> Self {
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

impl<'a> NodePersistence<'a> for PersistentOriginalNodes<'a> {
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
                    .or_insert_with(|| {
                        vec![(
                            original_node.action,
                            original_node.source,
                            original_node.node,
                        )]
                    });
            })
            .or_insert_with(|| {
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
            .expect("The iterator should use a filter to ensure only nodes with at least two tokens make it this far");

        let original_node_name = data
            .get_tokens(node_index)
            .and_then(|tokens| data.get_lexeme(source_index, tokens[1]))
            .expect("The iterator should use a filter to ensure only nodes with at least two tokens make it this far");

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
                let key_index = usize::from(matches!(
                    tokens
                        .first()
                        .and_then(|t| data.get_lexeme(source_index, *t)),
                    Some("remove" | "add")
                ));

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
                .expect("The iterator should use a filter to ensure only nodes with at least one tokens (or two, if starting with add/remove) are allowed")
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
