use crate::generators;
use crate::wandom::ShuffleIndex;
use crate::zippy::Zip;

use endless_sky_rw::{
    Data, Node, NodeIndex, SourceIndex, Span, Token, TokenKind, node_path_iter, tree_from_tokens,
};

use std::{collections::HashMap, error::Error, path::PathBuf};

use wasm_bindgen::prelude::*;

const PLUGIN_NAME: &str = "Chaos";

const PLUGIN_DESCRIPTION: &str = "\
    Shuffles every outfit name and image.\n\
    Shuffles every ship name and image.\
";

const PLUGIN_VERSION: &str = "0.1.0";

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct ChaosConfig {
    seed: usize,
}

#[wasm_bindgen]
impl ChaosConfig {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(seed: usize) -> Self {
        Self { seed }
    }
}

pub fn process(
    paths: Vec<String>,
    sources: Vec<String>,
    settings: ChaosConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_folder = generators::read_upload(paths, sources)?;

    let data = data_folder.data();

    let mut output = vec![];

    let mut generator = Chaos {
        archive: Zip::new(&mut output),
        output_data: Data::default(),
        settings,
    };

    generator.description()?;

    generator.archive.write_dir("data/")?;

    generator.outfits(data)?;

    generator.ships(data)?;

    generator.archive.finish()?;

    Ok(output)
}

struct Chaos<'a> {
    archive: Zip<'a>,
    output_data: Data,
    settings: ChaosConfig,
}

struct OutfitData<'a> {
    name: &'a str,
    thumbnail: NodeIndex,
    series: Option<NodeIndex>,
    index: Option<NodeIndex>,
}

struct ShipData<'a> {
    name: &'a str,
    noun: Option<NodeIndex>,
    plural: Option<NodeIndex>,
    sprite: Option<NodeIndex>,
    thumbnail: Option<NodeIndex>,
}

fn copy_node(
    data: &Data,
    (source_index, node_index): (SourceIndex, NodeIndex),
    output_data: &mut Data,
    output_source: SourceIndex,
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
            if let Some(output_child) =
                copy_node(data, (source_index, *child), output_data, output_source)
            {
                output_data.push_child(output_node, output_child);
            }
        }
    }

    Some(output_node)
}

impl Chaos<'_> {
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

    fn get_copies_of_child_node(
        &mut self,
        data: &Data,
        (source_index, node_index): (SourceIndex, NodeIndex),
        kind: &str,
        minimum_length: usize,
        output_source: SourceIndex,
    ) -> Vec<NodeIndex> {
        data.filter_children(source_index, node_index, |source_index, tokens| {
            matches!(
                tokens
                    .first()
                    .and_then(|token| data.get_lexeme(source_index, *token)),
                Some(lexeme) if lexeme == kind
            )
        })
        .filter(|node_index| {
            data.get_tokens(*node_index).map_or(0, <[Token]>::len) >= minimum_length
        })
        .filter_map(|node_index| {
            copy_node(
                data,
                (source_index, node_index),
                &mut self.output_data,
                output_source,
            )
        })
        // TODO: don't collect?
        .collect::<Vec<_>>()
    }

    fn get_copy_of_child_node(
        &mut self,
        data: &Data,
        (source_index, node_index): (SourceIndex, NodeIndex),
        kind: &str,
        minimum_length: usize,
        output_source: SourceIndex,
    ) -> Option<NodeIndex> {
        self.get_copies_of_child_node(
            data,
            (source_index, node_index),
            kind,
            minimum_length,
            output_source,
        )
        .into_iter()
        .last()
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

    fn outfits(&mut self, data: &Data) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let outfit_output_source = self.output_data.insert_source(String::new());

        let outfit_data = self.get_outfit_data(data, outfit_output_source);

        let mut outfit_keys = outfit_data.keys().collect::<Vec<_>>();

        outfit_keys.sort_unstable();

        let outfit_swaps = outfit_keys
            .iter()
            .zip(
                outfit_keys
                    .as_slice()
                    .shuffled_indices(self.settings.seed)
                    .into_iter()
                    .filter_map(|i| outfit_keys.get(i)),
            )
            .collect::<HashMap<_, _>>();

        for original in &outfit_keys {
            let swap = outfit_swaps.get(original).expect("Outfit data must exist");
            let swapped_data = outfit_data.get(**swap).expect("Outfit data must exist");

            let outfit = tree_from_tokens!(
                &mut self.output_data; outfit_output_source =>
                : "outfit", original ;
                {
                    : "display name", swapped_data.name ;
                }
            );

            self.output_data.push_child(outfit, swapped_data.thumbnail);

            if let Some(series) = swapped_data.series {
                self.output_data.push_child(outfit, series);
            }

            if let Some(index) = swapped_data.index {
                self.output_data.push_child(outfit, index);
            }

            self.output_data
                .push_root_node(outfit_output_source, outfit);
        }

        self.zip_root_nodes("data/outfits.txt", output_root_node_count)
    }

    fn ships(&mut self, data: &Data) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let ship_output_source = self.output_data.insert_source(String::new());

        let ship_data = self.get_ship_data(data, ship_output_source);

        let mut ship_keys = ship_data.keys().collect::<Vec<_>>();

        ship_keys.sort_unstable();

        let ship_swaps = ship_keys
            .iter()
            .zip(
                ship_keys
                    .as_slice()
                    .shuffled_indices(self.settings.seed)
                    .into_iter()
                    .filter_map(|i| ship_keys.get(i)),
            )
            .collect::<HashMap<_, _>>();

        // TODO: ship variants don't get swapped assets, fix that
        for original in &ship_keys {
            let swap = ship_swaps.get(original).expect("Ship data must exist");
            let swapped_data = ship_data.get(**swap).expect("Ship data must exist");

            let ship = tree_from_tokens!(
                &mut self.output_data; ship_output_source =>
                : "ship", original ;
                {
                    : "display name", swapped_data.name ;
                }
            );

            if let Some(noun) = swapped_data.noun {
                self.output_data.push_child(ship, noun);
            }

            if let Some(plural) = swapped_data.plural {
                self.output_data.push_child(ship, plural);
            }

            if let Some(sprite) = swapped_data.sprite {
                self.output_data.push_child(ship, sprite);
            }

            if let Some(thumbnail) = swapped_data.thumbnail {
                self.output_data.push_child(ship, thumbnail);
            }

            self.output_data.push_root_node(ship_output_source, ship);
        }

        self.zip_root_nodes("data/ships.txt", output_root_node_count)
    }

    fn get_outfit_data<'a>(
        &mut self,
        data: &'a Data,
        outfit_output_source: SourceIndex,
    ) -> HashMap<&'a str, OutfitData<'a>> {
        node_path_iter!(data; "outfit")
            .filter(|(source_index, node_index)| {
                data.get_tokens(*node_index)
                    .map_or(0, <[Token]>::len)
                    == 2
                && data.get_children(*node_index)
                    .map_or(
                        [].as_slice(),
                        |children| children
                    )
                    .iter()
                    .any(|child| {
                        !matches!(
                            data.get_tokens(*child)
                                .and_then(|tokens| tokens.first())
                                .and_then(|token| data.get_lexeme(*source_index, *token)),
                            Some("weapon")
                        )
                    })
            })
            .fold(
                HashMap::new(),
                |mut accum, (outfit_source_index, outfit)| {
                    let outfit_name = data
                        .get_tokens(outfit)
                        .and_then(|tokens| tokens.get(1))
                        .and_then(|token| data.get_lexeme(outfit_source_index, *token))
                        .expect(
                            "The iterator should use a filter to ensure all outfits have a name",
                        );

                    accum.insert(
                    outfit_name,
                    OutfitData {
                        name:
                            node_path_iter!(data => (outfit_source_index, outfit); "display name")
                                .filter_map(|node_index| {
                                    data.get_tokens(node_index).and_then(|tokens| {
                                        tokens.get(1).and_then(|token| {
                                            data.get_lexeme(outfit_source_index, *token)
                                        })
                                    })
                                })
                                .last()
                                .map_or(outfit_name, |outfit_name| outfit_name),
                        thumbnail:
                            self.get_copy_of_child_node(data, (outfit_source_index, outfit), "thumbnail", 2, outfit_output_source).unwrap_or_else(||
                                    tree_from_tokens!(
                                        &mut self.output_data; outfit_output_source =>
                                        : "thumbnail", "outfit/unknown" ;
                                ),
                            ),
                        series: self.get_copy_of_child_node(data, (outfit_source_index, outfit), "series", 2, outfit_output_source),
                        index: self.get_copy_of_child_node(data, (outfit_source_index, outfit), "index", 2, outfit_output_source),
                    },
                );

                    accum
                },
            )
    }

    fn get_ship_data<'a>(
        &mut self,
        data: &'a Data,
        ship_output_source: SourceIndex,
    ) -> HashMap<&'a str, ShipData<'a>> {
        node_path_iter!(data; "ship")
            .filter(|(_, node_index)| data.get_tokens(*node_index).map_or(0, <[Token]>::len) == 2)
            .fold(HashMap::new(), |mut accum, (ship_source_index, ship)| {
                let ship_name = data
                    .get_tokens(ship)
                    .and_then(|tokens| tokens.get(1))
                    .and_then(|token| data.get_lexeme(ship_source_index, *token))
                    .expect("The iterator should use a filter to ensure all ships have a name");

                let ship_sprite = self.get_copy_of_child_node(
                    data,
                    (ship_source_index, ship),
                    "sprite",
                    2,
                    ship_output_source,
                );

                accum.insert(
                    ship_name,
                    ShipData {
                        name: node_path_iter!(data => (ship_source_index, ship); "display name")
                            .filter_map(|node_index| {
                                data.get_tokens(node_index).and_then(|tokens| {
                                    tokens.get(1).and_then(|token| {
                                        data.get_lexeme(ship_source_index, *token)
                                    })
                                })
                            })
                            .last()
                            .map_or(ship_name, |ship_name| ship_name),
                        plural: self.get_copy_of_child_node(
                            data,
                            (ship_source_index, ship),
                            "plural",
                            2,
                            ship_output_source,
                        ),
                        noun: self.get_copy_of_child_node(
                            data,
                            (ship_source_index, ship),
                            "noun",
                            2,
                            ship_output_source,
                        ),
                        sprite: ship_sprite,
                        thumbnail: self
                            .get_copy_of_child_node(
                                data,
                                (ship_source_index, ship),
                                "thumbnail",
                                2,
                                ship_output_source,
                            )
                            .or(ship_sprite),
                    },
                );

                accum
            })
    }
}
