pub mod config;

use crate::{
    generators,
    wandom::{XoShiRo256SS, shuffle_index::ShuffleIndex},
    zippy::Zip,
};

use endless_sky_rw::{
    Data, DataFolder, Node, NodeIndex, SourceIndex, Span, Token, TokenKind, node_path_iter,
    tree_from_tokens,
};

use std::{collections::HashMap, error::Error, path::PathBuf};

const PLUGIN_NAME: &str = "Chaos";

const PLUGIN_VERSION: &str = "0.2.0";

#[allow(clippy::missing_errors_doc)]
pub fn process_data(
    data_folder: &DataFolder,
    settings: &config::ChaosConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = data_folder.data();

    let mut rng = XoShiRo256SS::new(*settings.seed());
    let mut output = vec![];

    let mut generator = Chaos {
        archive: Zip::new(&mut output),
        output_data: Data::default(),
    };

    generator.description(settings)?;

    generator.archive.write_dir("data/")?;

    if *settings.outfits() {
        let mut outfit_rng = XoShiRo256SS::new(rng.step());

        generator.outfits(data, &mut outfit_rng)?;
    }

    if *settings.ships() {
        let mut ship_rng = XoShiRo256SS::new(rng.step());

        generator.ships(data, &mut ship_rng)?;
    }

    if *settings.systems() {
        let mut system_name_rng = XoShiRo256SS::new(rng.step());

        generator.systems(data, &mut system_name_rng)?;
    }

    if *settings.planets() {
        let mut planet_name_rng = XoShiRo256SS::new(rng.step());

        generator.planets(data, &mut planet_name_rng)?;
    }

    generator.archive.finish()?;

    Ok(output)
}

struct Chaos<'a> {
    archive: Zip<'a>,
    output_data: Data,
}

struct OutfitData<'a> {
    name: &'a str,
    thumbnail: NodeIndex,
    series: Option<NodeIndex>,
    index: Option<NodeIndex>,
}

struct ShipData<'a> {
    name: &'a str,
    model: &'a str,
    noun: Option<NodeIndex>,
    plural: Option<NodeIndex>,
    sprite: Option<NodeIndex>,
    thumbnail: Option<NodeIndex>,
}

struct SystemData<'a> {
    name: &'a str,
}

struct PlanetData<'a> {
    name: &'a str,
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
    ) -> impl Iterator<Item = NodeIndex> {
        data.filter_children(source_index, node_index, move |source_index, tokens| {
            matches!(
                tokens
                    .first()
                    .and_then(|token| data.get_lexeme(source_index, *token)),
                Some(lexeme) if lexeme == kind
            )
        })
        .filter(move |node_index| {
            data.get_tokens(*node_index).map_or(0, <[Token]>::len) >= minimum_length
        })
        .filter_map(move |node_index| {
            generators::copy_node(
                data,
                (source_index, node_index),
                &mut self.output_data,
                output_source,
                [].as_slice(),
            )
        })
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
        .last()
    }

    fn description(&mut self, settings: &config::ChaosConfig) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();
        let plugin_txt_source = self.output_data.insert_source(String::new());

        let plugin_name = tree_from_tokens!(
            &mut self.output_data; plugin_txt_source =>
            : "name", PLUGIN_NAME ;
        );

        self.output_data
            .push_root_node(plugin_txt_source, plugin_name);

        if *settings.outfits() {
            let plugin_about = tree_from_tokens!(
                &mut self.output_data; plugin_txt_source =>
                : "about", "Shuffles every outfit name and image." ;
            );

            self.output_data
                .push_root_node(plugin_txt_source, plugin_about);
        }

        if *settings.ships() {
            let plugin_about = tree_from_tokens!(
                &mut self.output_data; plugin_txt_source =>
                : "about", "Shuffles every ship name and image." ;
            );

            self.output_data
                .push_root_node(plugin_txt_source, plugin_about);
        }

        if *settings.systems() {
            let plugin_about = tree_from_tokens!(
                &mut self.output_data; plugin_txt_source =>
                : "about", "Shuffles every system name." ;
            );

            self.output_data
                .push_root_node(plugin_txt_source, plugin_about);
        }

        if *settings.planets() {
            let plugin_about = tree_from_tokens!(
                &mut self.output_data; plugin_txt_source =>
                : "about", "Shuffles every planet name." ;
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

        let dependencies = tree_from_tokens!(
            &mut self.output_data; plugin_txt_source =>
            : "dependencies" ;
            {
                : "game version", crate::GAME_VERSION ;
            }
        );

        self.output_data
            .push_root_node(plugin_txt_source, dependencies);

        self.zip_root_nodes("plugin.txt", output_root_node_count)
    }

    fn outfits(&mut self, data: &Data, rng: &mut XoShiRo256SS) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let outfit_output_source = self.output_data.insert_source(String::new());

        let outfit_data = self.get_outfit_data(data, outfit_output_source);

        let mut outfit_keys = outfit_data.keys().collect::<Vec<_>>();

        outfit_keys.sort_unstable();

        let outfit_swaps = outfit_keys
            .iter()
            .zip(
                outfit_keys
                    .shuffled_indices_with_rng(rng)
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

    fn ships(&mut self, data: &Data, rng: &mut XoShiRo256SS) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let ship_output_source = self.output_data.insert_source(String::new());

        let mut ship_data = self.get_ship_data(data, ship_output_source);

        self.get_ship_variant_data(data, ship_output_source, &mut ship_data);

        let mut ship_keys = ship_data.keys().collect::<Vec<_>>();

        ship_keys.sort_unstable();

        let ship_swaps = ship_keys
            .iter()
            .zip(
                ship_keys
                    .shuffled_indices_with_rng(rng)
                    .into_iter()
                    .filter_map(|i| ship_keys.get(i)),
            )
            .collect::<HashMap<_, _>>();

        for original in &ship_keys {
            let original_data = ship_data.get(**original).expect("Ship data must exist");
            let swap = ship_swaps.get(original).expect("Ship data must exist");
            let swapped_data = ship_data.get(**swap).expect("Ship data must exist");

            let ship = if original_data.model == **original {
                tree_from_tokens!(
                    &mut self.output_data; ship_output_source =>
                    : "ship", original ;
                    {
                        : "display name", swapped_data.name ;
                    }
                )
            } else {
                tree_from_tokens!(
                    &mut self.output_data; ship_output_source =>
                    : "ship", original_data.model, original ;
                    {
                        : "display name", swapped_data.name ;
                    }
                )
            };

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
            } else if let Some(sprite) = swapped_data.sprite
                && let Some(tokens) = self.output_data.get_tokens(sprite)
                && let Some(token) = tokens.get(1)
                && let Some(sprite) = self.output_data.get_lexeme(ship_output_source, *token)
            {
                let sprite = sprite.to_string();

                let thumbnail = tree_from_tokens!(
                    &mut self.output_data; ship_output_source =>
                    : "thumbnail", sprite ;
                );

                self.output_data.push_child(ship, thumbnail);
            }

            self.output_data.push_root_node(ship_output_source, ship);
        }

        self.zip_root_nodes("data/ships.txt", output_root_node_count)
    }

    fn systems(&mut self, data: &Data, rng: &mut XoShiRo256SS) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let system_output_source = self.output_data.insert_source(String::new());

        let system_data = Self::get_system_data(data);

        let mut system_keys = system_data.keys().collect::<Vec<_>>();

        system_keys.sort_unstable();

        let system_swaps = system_keys
            .iter()
            .zip(
                system_keys
                    .shuffled_indices_with_rng(rng)
                    .into_iter()
                    .filter_map(|i| system_keys.get(i)),
            )
            .collect::<HashMap<_, _>>();

        for original in &system_keys {
            let swap = system_swaps.get(original).expect("System data must exist");
            let swapped_data = system_data.get(**swap).expect("System data must exist");

            let system = tree_from_tokens!(
                &mut self.output_data; system_output_source =>
                : "system", original ;
                {
                    : "display name", swapped_data.name ;
                }
            );

            self.output_data
                .push_root_node(system_output_source, system);
        }

        self.zip_root_nodes("data/systems.txt", output_root_node_count)
    }

    fn planets(&mut self, data: &Data, rng: &mut XoShiRo256SS) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let planet_output_source = self.output_data.insert_source(String::new());

        let planet_data = Self::get_planet_data(data);

        let mut planet_keys = planet_data.keys().collect::<Vec<_>>();

        planet_keys.sort_unstable();

        let planet_swaps = planet_keys
            .iter()
            .zip(
                planet_keys
                    .shuffled_indices_with_rng(rng)
                    .into_iter()
                    .filter_map(|i| planet_keys.get(i)),
            )
            .collect::<HashMap<_, _>>();

        for original in &planet_keys {
            let swap = planet_swaps.get(original).expect("Planet data must exist");
            let swapped_data = planet_data.get(**swap).expect("Planet data must exist");

            let planet = tree_from_tokens!(
                &mut self.output_data; planet_output_source =>
                : "planet", original ;
                {
                    : "display name", swapped_data.name ;
                }
            );

            self.output_data
                .push_root_node(planet_output_source, planet);
        }

        self.zip_root_nodes("data/planets.txt", output_root_node_count)
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
                                    .filter_map(|(_, node_index)| {
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
                            .filter_map(|(_, node_index)| {
                                data.get_tokens(node_index).and_then(|tokens| {
                                    tokens.get(1).and_then(|token| {
                                        data.get_lexeme(ship_source_index, *token)
                                    })
                                })
                            })
                            .last()
                            .map_or(ship_name, |ship_name| ship_name),
                        model: ship_name,
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
                        thumbnail: self.get_copy_of_child_node(
                            data,
                            (ship_source_index, ship),
                            "thumbnail",
                            2,
                            ship_output_source,
                        ),
                    },
                );

                accum
            })
    }

    fn get_ship_variant_data<'a>(
        &mut self,
        data: &'a Data,
        ship_output_source: SourceIndex,
        ship_data: &mut HashMap<&'a str, ShipData<'a>>,
    ) {
        node_path_iter!(data; "ship")
            .filter(|(source_index, node_index)| {
                data.get_tokens(*node_index).map_or(0, <[Token]>::len) == 3
                    && node_path_iter!(
                        data => (*source_index, *node_index);
                        "display name" | "plural" | "noun" | "sprite" | "thumbnail"
                    )
                    .next()
                    .is_some()
            })
            .for_each(|(ship_source_index, ship)| {
                let ship_variant = data
                    .get_tokens(ship)
                    .and_then(|tokens| tokens.get(2))
                    .and_then(|token| data.get_lexeme(ship_source_index, *token))
                    .expect("The iterator should use a filter to ensure all ships have a name");

                let ship_model = data
                    .get_tokens(ship)
                    .and_then(|tokens| tokens.get(1))
                    .and_then(|token| data.get_lexeme(ship_source_index, *token))
                    .expect("The iterator should use a filter to ensure all ships have a name");

                let ship_sprite = self
                    .get_copy_of_child_node(
                        data,
                        (ship_source_index, ship),
                        "sprite",
                        2,
                        ship_output_source,
                    )
                    .or_else(|| ship_data.get(&ship_model).and_then(|data| data.sprite));

                ship_data.insert(
                    ship_variant,
                    ShipData {
                        name: node_path_iter!(data => (ship_source_index, ship); "display name")
                            .filter_map(|(_, node_index)| {
                                data.get_tokens(node_index).and_then(|tokens| {
                                    tokens.get(1).and_then(|token| {
                                        data.get_lexeme(ship_source_index, *token)
                                    })
                                })
                            })
                            .last()
                            .map_or(ship_model, |ship_name| ship_name),
                        model: ship_model,
                        plural: self
                            .get_copy_of_child_node(
                                data,
                                (ship_source_index, ship),
                                "plural",
                                2,
                                ship_output_source,
                            )
                            .or_else(|| ship_data.get(&ship_model).and_then(|data| data.plural)),
                        noun: self
                            .get_copy_of_child_node(
                                data,
                                (ship_source_index, ship),
                                "noun",
                                2,
                                ship_output_source,
                            )
                            .or_else(|| ship_data.get(&ship_model).and_then(|data| data.noun)),
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
            });
    }

    fn get_system_data(data: &Data) -> HashMap<&str, SystemData<'_>> {
        node_path_iter!(data; "system")
            .filter(|(_, node_index)| {
                data.get_tokens(*node_index)
                    .map_or(0, <[Token]>::len)
                    == 2
            })
            .fold(
                HashMap::new(),
                |mut accum, (system_source_index, system)| {
                    let system_name = data
                        .get_tokens(system)
                        .and_then(|tokens| tokens.get(1))
                        .and_then(|token| data.get_lexeme(system_source_index, *token))
                        .expect(
                            "The iterator should use a filter to ensure all systems have a name",
                        );

                    accum.insert(
                        system_name,
                        SystemData {
                            name:
                                node_path_iter!(data => (system_source_index, system); "display name")
                                    .filter_map(|(_, node_index)| {
                                        data.get_tokens(node_index).and_then(|tokens| {
                                            tokens.get(1).and_then(|token| {
                                                data.get_lexeme(system_source_index, *token)
                                            })
                                        })
                                    })
                                    .last()
                                    .map_or(system_name, |system_name| system_name),
                        },
                    );

                    accum
                },
            )
    }

    fn get_planet_data(data: &Data) -> HashMap<&str, PlanetData<'_>> {
        node_path_iter!(data; "planet")
            .filter(|(_, node_index)| {
                data.get_tokens(*node_index)
                    .map_or(0, <[Token]>::len)
                    == 2
            })
            .fold(
                HashMap::new(),
                |mut accum, (planet_source_index, planet)| {
                    let planet_name = data
                        .get_tokens(planet)
                        .and_then(|tokens| tokens.get(1))
                        .and_then(|token| data.get_lexeme(planet_source_index, *token))
                        .expect(
                            "The iterator should use a filter to ensure all planets have a name",
                        );

                    accum.insert(
                        planet_name,
                        PlanetData {
                            name:
                                node_path_iter!(data => (planet_source_index, planet); "display name")
                                    .filter_map(|(_, node_index)| {
                                        data.get_tokens(node_index).and_then(|tokens| {
                                            tokens.get(1).and_then(|token| {
                                                data.get_lexeme(planet_source_index, *token)
                                            })
                                        })
                                    })
                                    .last()
                                    .map_or(planet_name, |planet_name| planet_name),
                        },
                    );

                    accum
                },
            )
    }
}
