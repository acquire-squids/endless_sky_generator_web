crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in main =>
    #[derive(Debug)]
    pub RandomGalaxyConfig ;
    name: String => |name: String| {
        name
            .chars()
            .filter(|ch| {
                *ch == ' ' || *ch == '\'' || *ch == '_' || ch.is_ascii_alphanumeric()
            })
            .collect::<String>()
    },
    seed: u64,
    reveal_all: bool,
    clusters: Vec<random_galaxy::config::Cluster>,
    system_name_sources: random_galaxy::config::SystemNameSources,
    sprites: random_galaxy::config::Sprites,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in sprites =>
    #[derive(Debug)]
    pub Sprites ;
    galaxy: random_galaxy::config::GalaxySprite,
    stars: random_galaxy::config::Stars,
    planets: random_galaxy::config::Planets,
}

crate::macros::wasm_newtype! {
    in galaxy_sprite =>
    #[derive(Debug)]
    pub GalaxySprite ;
    sprite_name: String,
    blob: Vec<u8>,
}

crate::macros::wasm_newtype! {
    in min_max =>
    #[derive(Debug)]
    pub MinMax ;
    min: f64,
    max: f64,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in cluster =>
    #[derive(Debug)]
    pub Cluster ;
    capacity: random_galaxy::config::SystemCapacity,
    placement: random_galaxy::config::SystemPlacement,
    names: random_galaxy::config::SystemNames,
    contents: random_galaxy::config::SystemContents,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in system_capacity =>
    #[derive(Debug)]
    pub SystemCapacity ;
    size: random_galaxy::vec2f::Vec2f,
    system_count: usize,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in system_placement =>
    #[derive(Debug)]
    pub SystemPlacement ;
    origin: random_galaxy::vec2f::Vec2f,
    wormhole: String,
    max_link_length: u16,
    link_chance: f64,
    minimum_distance: f64,
    step_size: random_galaxy::config::MinMax,
}

crate::macros::wasm_newtype! {
    in system_names =>
    #[derive(Debug)]
    pub SystemNames ;
    source_index: usize,
    max_length: u8,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in system_name_sources =>
    #[derive(Debug)]
    pub SystemNameSources ;
    groups: Vec<random_galaxy::config::SystemNameSource>,
}

crate::macros::wasm_newtype! {
    in system_name_source =>
    #[derive(Debug)]
    pub SystemNameSource ;
    group_name: String,
    names: Vec<String>,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in system_contents =>
    #[derive(Debug)]
    pub SystemContents ;
    stars: Vec<random_galaxy::config::ClusterStarGroup>,
    planets: Vec<random_galaxy::config::ClusterPlanetGroup>,
}

crate::macros::wasm_newtype! {
    in cluster_star_group =>
    #[derive(Debug)]
    pub ClusterStarGroup ;
    group_index: usize,
    can_be_binary: bool,
    weight: u32,
    max_planets: u8,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in stars =>
    #[derive(Debug)]
    pub Stars ;
    groups: Vec<random_galaxy::config::StarGroup>,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in star_group =>
    #[derive(Debug)]
    pub StarGroup ;
    group_name: String,
    stars: Vec<random_galaxy::config::Star>,
}

crate::macros::wasm_newtype! {
    in star =>
    #[derive(Debug)]
    pub Star ;
    sprite_name: String,
    habitable: u32,
    binary_distance: f64,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in cluster_planet_group =>
    #[derive(Debug)]
    pub ClusterPlanetGroup ;
    group_index: usize,
    weight: u32,
    distance_range_percentage: random_galaxy::config::MinMax,
    moons: random_galaxy::config::PlanetMoons,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in planet_moons =>
    #[derive(Debug)]
    pub PlanetMoons ;
    chance: f64,
    from_planet_groups: Vec<random_galaxy::config::PlanetMoon>,
}

crate::macros::wasm_newtype! {
    in planet_moon =>
    #[derive(Debug)]
    pub PlanetMoon ;
    planet_group_index: usize,
    weight: u32,
}

crate::macros::wasm_newtype! {
    using crate::generators::random_galaxy ;
    in planets =>
    #[derive(Debug)]
    pub Planets ;
    groups: Vec<random_galaxy::config::PlanetGroup>,
}

crate::macros::wasm_newtype! {
    in planet_group =>
    #[derive(Debug)]
    pub PlanetGroup ;
    group_name: String,
    sprite_names: Vec<String>,
}

#[allow(unreachable_patterns, unused_variables)]
pub mod from_file {
    use crate::{
        config::{self, Value},
        generators::random_galaxy::{
            config::{
                Cluster, ClusterPlanetGroup, ClusterStarGroup, GalaxySprite, MinMax, PlanetGroup,
                PlanetMoon, PlanetMoons, Planets, RandomGalaxyConfig, Sprites, Star, StarGroup,
                Stars, SystemCapacity, SystemContents, SystemNameSource, SystemNameSources,
                SystemNames, SystemPlacement,
            },
            vec2f::Vec2f,
        },
    };

    use std::{fs, path::PathBuf};

    #[must_use]
    pub fn parse(source: &str) -> Option<RandomGalaxyConfig> {
        let system_names = self::system_name_sources(source)?;
        let stars = self::stars(source)?;
        let planets = self::planets(source)?;

        config::parse_config!(
            source => RandomGalaxyConfig;
            name => { string => name.to_string() }
            seed => { int of u64 => seed }
            reveal_all => { bool => *reveal_all }
            clusters => { list where !clusters.is_empty() => {
                clusters.iter().filter_map(|cluster| {
                    self::cluster(cluster, &system_names, &stars, &planets)
                }).collect::<Vec<_>>()
            }}
            system_name_sources => { list where !system_name_sources.is_empty() => {
                system_names
            }}
            sprite_name => { string => Sprites::new(self::galaxy_sprite(source)?, stars, planets) }
        )
    }

    fn system_name_sources(source: &str) -> Option<SystemNameSources> {
        config::parse_config!(
            source => SystemNameSources;
            system_name_sources => {list where !system_name_sources.is_empty() => {
                system_name_sources.iter().filter_map(|system_name_source| {
                    self::system_name_source(system_name_source)
                }).collect::<Vec<_>>()
            }}
        )
    }

    fn system_name_source(system_name_source: &Value) -> Option<SystemNameSource> {
        config::config_option!(
            system_name_source as list where system_name_source.len() == 2 => {
                let name_source_group_name = system_name_source.first()?;

                let name_source_group_name = config::config_option!(
                    name_source_group_name as string => name_source_group_name
                )?.to_string();

                let names = system_name_source.get(1)?;

                SystemNameSource::new(
                    name_source_group_name,
                    config::config_option!(
                        names as list where !names.is_empty() => {
                            names.iter().filter_map(|name_example| config::config_option!(
                                name_example as string => name_example
                            ).map(std::string::ToString::to_string)).collect::<Vec<_>>()
                        }
                    )?,
                )
            }
        )
    }

    fn stars(source: &str) -> Option<Stars> {
        config::parse_config!(
            source => Stars ;
            star_groups => { list => {
                star_groups.iter().filter_map(self::star_group).collect::<Vec<_>>()
            }}
        )
    }

    fn star_group(star_group: &Value) -> Option<StarGroup> {
        config::config_option!(
            star_group as list where star_group.len() == 2 => {
                let star_group_name = star_group.first()?;

                let star_group_name = config::config_option!(
                    star_group_name as string => star_group_name
                )?.to_string();

                let stars = star_group.get(1)?;

                StarGroup::new(
                    star_group_name,
                    config::config_option!(
                        stars as list where !stars.is_empty() => {
                            stars.iter().filter_map(self::star).collect::<Vec<_>>()
                        }
                    )?,
                )
            }
        )
    }

    fn star(star: &Value) -> Option<Star> {
        config::config_option!(
            star as list where star.len() == 2 => {
                let star_name = star.first()?;

                let star_name = config::config_option!(
                    star_name as string => star_name
                )?.to_string();

                let properties = config::key_value_list(star.get(1)?)?;

                let (habitable, binary_distance) = (
                    properties.get("habitable").map_or_else(
                        || {
                            eprintln!("All stars must have (\"habitable\" u32)");

                            None
                        },
                        Some,
                    ),
                    properties.get("binary_distance").map_or_else(
                        || {
                            eprintln!("All stars must have (\"binary_distance\" u16)");

                            None
                        },
                        Some
                    )
                );

                let (habitable, binary_distance) = (habitable?, binary_distance?);

                Star::new(
                    star_name,
                    config::config_option!(
                        habitable as int of u32 => habitable
                    )?,
                    config::config_option!(
                        binary_distance as int of f64 => binary_distance
                    )?
                )
            }
        )
    }

    fn planets(source: &str) -> Option<Planets> {
        config::parse_config!(
            source => Planets ;
            planet_groups => { list => {
                planet_groups.iter().filter_map(self::planet_group).collect::<Vec<_>>()
            }}
        )
    }

    fn planet_group(planet_group: &Value) -> Option<PlanetGroup> {
        config::config_option!(
            planet_group as list where planet_group.len() == 2 => {
                let planet_group_name = planet_group.first()?;

                let planet_group_name = config::config_option!(
                    planet_group_name as string => planet_group_name
                )?.to_string();

                let properties = config::key_value_list(planet_group.get(1)?)?;

                let planet_sprites = properties.get("sprites").map_or_else(
                    || {
                        eprintln!("All planet sprite groups must have (\"sprite\" (string))");

                        None
                    },
                    Some,
                );

                let planet_sprites = planet_sprites?;

                PlanetGroup::new(
                    planet_group_name,
                    config::config_option!(
                        planet_sprites as list where !planet_sprites.is_empty() => {
                            planet_sprites.iter().filter_map(|planet_sprite| {
                                config::config_option!(
                                    planet_sprite as string => planet_sprite
                                )
                            }).map(std::string::ToString::to_string).collect::<Vec<_>>()
                        }
                    )?
                )
            }
        )
    }

    fn galaxy_sprite(source: &str) -> Option<GalaxySprite> {
        config::parse_config!(
            source => GalaxySprite;
            sprite_name => { string => sprite_name.to_string() }
            blob => { string => {
                let path = PathBuf::from(blob);

                if !path.exists() {
                    eprintln!("Galaxy sprite \"{blob}\" does not exist!");
                    return None;
                } else if !path.is_file() {
                    eprintln!("Galaxy sprite \"{blob}\" is not a file!");
                    return None;
                }

                match fs::read(path) {
                    Ok(blob) => blob,
                    Err(error) => {
                        eprintln!("{error}");
                        eprintln!("Failed to read galaxy sprite \"{blob}\"!");
                        return None;
                    }
                }
            }}
        )
    }

    fn cluster(
        cluster: &Value,
        system_name_sources: &SystemNameSources,
        stars: &Stars,
        planets: &Planets,
    ) -> Option<Cluster> {
        let cluster = config::key_value_list(cluster)?;

        let (capacity, placement, names, system_contents) = (
            cluster.get("capacity").map_or_else(|| {
                eprintln!("All system clusters must have (\"capacity\" ((\"size\" (f64 f64)) (\"system_count\" usize)))");

                None
            }, Some),
            cluster.get("system_placement").map_or_else(|| {
                eprintln!(
                    "All system clusters must have (\"system_placement\" ((\"origin\" (f64 f64)) (\"wormhole\" string) (\"max_link_length\" u16) (\"minimum_distance\" f64) (\"min_step_size\" f64) (\"max_step_size\" f64)))"
                );

                None
            }, Some),
            cluster.get("system_names").map_or_else(|| {
                eprintln!(
                    "All system clusters must have (\"system_names\" ((\"source\" string) (\"max_length\" u8)))"
                );

                None
            }, Some),
            cluster.get("system_contents").map_or_else(|| {
                eprintln!(
                    "All system clusters must have (\"system_contents\" (\"star_groups\" ((\"weight\" u32))) (\"planet_groups\" ((\"weight\" u32) (\"distance_range_percentange\" (f64 f64)) (\"moons\" ((\"chance\" f64) (\"from_planet_groups\" ((string ((weight u32))))))))))"
                );

                None
            }, Some),
        );

        let (capacity, placement, names, system_contents) =
            (capacity?, placement?, names?, system_contents?);

        Some(Cluster::new(
            self::system_capacity(capacity)?,
            self::system_placement(placement)?,
            self::system_names(names, system_name_sources)?,
            self::system_contents(system_contents, stars, planets)?,
        ))
    }

    fn system_capacity(capacity: &Value) -> Option<SystemCapacity> {
        let capacity = config::key_value_list(capacity)?;

        let (size, system_count) = (
            capacity.get("size").map_or_else(
                || {
                    eprintln!("All system cluster capacities must have (\"size\" (f64 f64))");

                    None
                },
                Some,
            ),
            capacity.get("system_count").map_or_else(
                || {
                    eprintln!("All system cluster capacities must have (\"system_count\" usize)");

                    None
                },
                Some,
            ),
        );

        let (size, system_count) = (size?, system_count?);

        Some(SystemCapacity::new(
            config::config_option!(
                size as list where size.len() == 2 => {
                    let width = size.first()?;
                    let height = size.get(1)?;

                    Vec2f::new(
                        config::config_option!(
                            width as float of f64 => width
                        )?,
                        config::config_option!(
                            height as float of f64 => height
                        )?
                    )
                }
            )?,
            config::config_option!(
                system_count as int of usize => system_count
            )?,
        ))
    }

    fn system_placement(placement: &Value) -> Option<SystemPlacement> {
        let placement = config::key_value_list(placement)?;

        let (origin, wormhole, max_link_length, link_chance, minimum_distance, step_size) = (
            placement.get("origin").map_or_else(
                || {
                    eprintln!("All system cluster placement specifications must have (\"origin\" (f64 f64))");

                    None
                },
                Some,
            ),
            placement.get("wormhole").map_or_else(|| {
                eprintln!("All system cluster placement specifications must have (\"wormhole\" string)");

                None
            }, Some),
            placement.get("max_link_length").map_or_else(
                || {
                    eprintln!("All system cluster placement specifications must have (\"max_link_length\" u16)");

                    None
                },
                Some,
            ),
            placement.get("link_chance").map_or_else(
                || {
                    eprintln!("All system cluster placement specifications must have (\"link_chance\" f64)");

                    None
                },
                Some,
            ),
            placement.get("minimum_distance").map_or_else(
                || {
                    eprintln!("All system cluster placement specifications must have (\"minimum_distance\" f64)");

                    None
                },
                Some,
            ),
            placement.get("step_size").map_or_else(
                || {
                    eprintln!("All system cluster placement specifications must have (\"step_size\" (f64 f64))");

                    None
                },
                Some,
            ),
        );

        let (origin, wormhole, max_link_length, link_chance, minimum_distance, step_size) = (
            origin?,
            wormhole?,
            max_link_length?,
            link_chance?,
            minimum_distance?,
            step_size?,
        );

        Some(SystemPlacement::new(
            config::config_option!(
                origin as list where origin.len() == 2 => {
                    let origin_x = origin.first()?;
                    let origin_y = origin.get(1)?;

                    Vec2f::new(
                        config::config_option!(
                            origin_x as float of f64 => origin_x
                        )?,
                        config::config_option!(
                            origin_y as float of f64 => origin_y
                        )?
                    )
                }
            )?,
            config::config_option!(
                wormhole as string => wormhole.to_string()
            )?,
            config::config_option!(
                max_link_length as int of u16 => max_link_length
            )?,
            config::config_option!(
                link_chance as float of f64 => link_chance
            )?,
            config::config_option!(
                minimum_distance as float of f64 => minimum_distance
            )?,
            config::config_option!(
                step_size as list where step_size.len() == 2 => {
                    let minimum_step_size = step_size.first()?;
                    let maximum_step_size = step_size.get(1)?;

                    MinMax::new(
                        config::config_option!(
                            minimum_step_size as float of f64 => minimum_step_size
                        )?,
                        config::config_option!(
                            maximum_step_size as float of f64 => maximum_step_size
                        )?
                    )
                }
            )?,
        ))
    }

    fn system_names(names: &Value, system_name_sources: &SystemNameSources) -> Option<SystemNames> {
        let names = config::key_value_list(names)?;

        let (source, max_name_length) = (
            names.get("source").map_or_else(
                || {
                    eprintln!(
                        "All system cluster names specifications must have (\"source\" string)"
                    );

                    None
                },
                Some,
            ),
            names.get("max_length").map_or_else(
                || {
                    eprintln!(
                        "All system cluster names specifications must have (\"max_length\" u8)"
                    );

                    None
                },
                Some,
            ),
        );

        let (source, max_name_length) = (source?, max_name_length?);

        Some(SystemNames::new(
            config::config_option!(
                source as string => {
                    system_name_sources.groups().iter().position(|name_group| name_group.group_name() == source)?
                }
            )?,
            config::config_option!(
                max_name_length as int of u8 => max_name_length
            )?,
        ))
    }

    fn system_contents(
        contents: &Value,
        stars: &Stars,
        planets: &Planets,
    ) -> Option<SystemContents> {
        let contents = config::key_value_list(contents)?;

        let (cluster_star_groups, cluster_planet_groups) = (
                    contents.get("star_groups").map_or_else(
                        || {
                            eprintln!("All system cluster contents specifications must have (\"star_groups\" ((\"weight\" u32)))");

                            None
                        },
                        Some,
                    ),
                    contents.get("planet_groups").map_or_else(|| {
                        eprintln!("All system cluster contents specifications must have (\"planet_groups\" ((\"weight\" u32) (\"distance_range_percentage\" (f64 f64)) (\"moons\" ((\"chance\" f64) (\"from_planet_groups\" ((string ((weight u32)))))))))");

                        None
                    }, Some),
                );

        let (cluster_star_groups, cluster_planet_groups) =
            (cluster_star_groups?, cluster_planet_groups?);

        Some(SystemContents::new(
            config::config_option!(
                cluster_star_groups as list => {
                    cluster_star_groups.iter().filter_map(|cluster_star_group| {
                        self::cluster_star_group(cluster_star_group, stars)
                    }).collect::<Vec<_>>()
                }
            )?,
            config::config_option!(
                cluster_planet_groups as list => {
                    cluster_planet_groups.iter().filter_map(|cluster_planet_group| {
                        self::cluster_planet_group(cluster_planet_group, planets)
                    }).collect::<Vec<_>>()
                }
            )?,
        ))
    }

    fn cluster_star_group(group: &Value, stars: &Stars) -> Option<ClusterStarGroup> {
        let group = config::key_value_list(group)?;

        let (source, can_be_binary, weight, max_planets) = (
            group.get("source").map_or_else(
                || {
                    eprintln!("All system cluster star groups must have (\"source\" string)");

                    None
                },
                Some,
            ),
            group.get("can_be_binary").map_or_else(
                || {
                    eprintln!("All system cluster star groups must have (\"can_be_binary\" bool)");

                    None
                },
                Some,
            ),
            group
                .get("weight")
                .and_then(|weight| {
                    config::config_option!(
                        weight as int of u32 => weight
                    )
                })
                .unwrap_or(1),
            group.get("max_planets").map_or_else(
                || {
                    eprintln!("All system cluster star groups must have (\"max_planets\" u32)");

                    None
                },
                Some,
            ),
        );

        let (source, can_be_binary, max_planets) = (source?, can_be_binary?, max_planets?);

        Some(ClusterStarGroup::new(
            config::config_option!(
                source as string => {
                    stars.groups().iter().position(|star_group| star_group.group_name() == source)?
                }
            )?,
            config::config_option!(
                can_be_binary as bool => *can_be_binary
            )?,
            weight,
            config::config_option!(
                max_planets as int of u8 => max_planets
            )?,
        ))
    }

    fn cluster_planet_group(group: &Value, planets: &Planets) -> Option<ClusterPlanetGroup> {
        let group = config::key_value_list(group)?;

        let (source, weight, distance_range_percentage, moons) = (
                    group.get("source").map_or_else(
                        || {
                            eprintln!("All system cluster planet groups must have (\"source\" string)");

                            None
                        },
                        Some
                    ),
                    group.get("weight").and_then(|weight| {
                        config::config_option!(
                            weight as int of u32 => weight
                        )
                    }).unwrap_or(1),
                    group.get("distance_range_percentage").map_or_else(
                        || {
                            eprintln!("All system cluster planet groups must have (\"distance_range_percentage\" (f64 f64))");

                            None
                        },
                        Some
                    ),
                    group.get("moons").map_or_else(
                        || {
                            eprintln!("All system cluster planet groups must have (\"moons\" (\"chance\" f64) (\"from_planet_groups\" ((string ((weight u32))))))");

                            None
                        },
                        Some
                    ),
                );

        let (source, distance_range_percentage, moons) =
            (source?, distance_range_percentage?, moons?);

        Some(ClusterPlanetGroup::new(
            config::config_option!(
                source as string => planets.groups().iter().position(|planet_group| planet_group.group_name() == source)?
            )?,
            weight,
            config::config_option!(
                distance_range_percentage as list where distance_range_percentage.len() == 2 => {
                    let distance_range_percentage_minimum = distance_range_percentage.first()?;
                    let distance_range_percentage_maximum = distance_range_percentage.get(1)?;

                    MinMax::new(
                        config::config_option!(
                            distance_range_percentage_minimum as float of f64 => distance_range_percentage_minimum
                        )?,
                        config::config_option!(
                            distance_range_percentage_maximum as float of f64 => distance_range_percentage_maximum
                        )?,
                    )
                }
            )?,
            self::planet_moons(planets, moons)?,
        ))
    }

    fn planet_moons(planets: &Planets, moons: &Value) -> Option<PlanetMoons> {
        let moons = config::key_value_list(moons)?;

        let (moon_chance, moon_from_planet_groups) = (
                    moons.get("chance").map_or_else(
                        || {
                            eprintln!("All system cluster planet group moons must have (\"chance\" f64)");

                            None
                        },
                        Some,
                    ),
                    moons.get("from_planet_groups").map_or_else(
                        || {
                            eprintln!("All system cluster planet group moons must have (\"from_planet_groups\" ((string ((weight u32)))))");

                            None
                        },
                        Some,
                    )
                );

        let (moon_chance, moon_from_planet_groups) = (moon_chance?, moon_from_planet_groups?);

        Some(PlanetMoons::new(
            config::config_option!(
                moon_chance as float of f64 => moon_chance
            )?,
            config::config_option!(
                moon_from_planet_groups as list => {
                    moon_from_planet_groups.iter()
                        .filter_map(|moon_planet_group| self::planet_moon(planets, moon_planet_group))
                        .collect::<Vec<_>>()
                }
            )?,
        ))
    }

    fn planet_moon(planets: &Planets, moon_planet_group: &Value) -> Option<PlanetMoon> {
        config::config_option!(
            moon_planet_group as list where moon_planet_group.len() == 2 => {
                let moon_planet_group_name = moon_planet_group.first()?;

                let moon_planet_group_name = config::config_option!(
                    moon_planet_group_name as string => moon_planet_group_name
                )?;

                let properties = config::key_value_list(moon_planet_group.get(1)?)?;

                let weight = properties.get("weight").and_then(|weight| {
                    config::config_option!(
                        weight as int of u32 => weight
                    )
                    }).unwrap_or(1);

                PlanetMoon::new(
                    planets.groups().iter().position(|planet_group| planet_group.group_name() == moon_planet_group_name)?,
                    weight,
                )
            }
        )
    }
}

pub mod page {
    use crate::html::{self, HtmlElement};

    #[must_use]
    pub fn random_galaxy() -> HtmlElement {
        HtmlElement::new("form")
            .with_name("random-galaxy-form")
            .with_id("random-galaxy-form")
            .novalidate()
            .with_element(
                HtmlElement::new("h2")
                    .with_element(html::page::anchor("Random_Galaxy", "Random Galaxy")),
            )
            .with_element(
                HtmlElement::new("p")
                    .with_text("Generates a random galaxy with the given parameters.<br/>")
                    .with_text(
                        "Nothing too interesting when it comes to content, and no story.<br/>",
                    )
                    .with_text("But it can be fun?<br/>"),
            )
            .with_element(random_galaxy_fieldset())
            .with_element(
                HtmlElement::new("button")
                    .with_id("random-galaxy-output")
                    .with_attribute("type", "submit")
                    .with_text("Generate and download"),
            )
    }

    fn random_galaxy_fieldset() -> HtmlElement {
        HtmlElement::new("fieldset")
            .with_element(HtmlElement::new("legend").with_text("Random Galaxy Settings:"))
            .with_element(html::page::labeled(
                "random-galaxy-name",
                "galaxy name:",
                HtmlElement::new("input")
                    .with_attribute("type", "text")
                    .required(),
            ))
            .with_element(html::page::labeled(
                "random-galaxy-sprite",
                "galaxy sprite:",
                HtmlElement::new("input")
                    .with_attribute("type", "file")
                    .with_attribute("accept", "image/*")
                    .required(),
            ))
            .with_element(html::page::labeled(
                "random-galaxy-seed",
                "seed:",
                HtmlElement::new("input")
                    .with_attribute("type", "number")
                    .required()
                    .with_attribute("value", 0u32),
            ))
            .with_element(html::page::labeled(
                "random-galaxy-reveal-all",
                "reveal all systems:",
                HtmlElement::new("input")
                    .with_attribute("type", "checkbox")
                    .checked(),
            ))
            .with_element(random_galaxy_clusters_fieldset())
            .with_element(random_galaxy_system_name_sources_fieldset())
            .with_element(random_galaxy_star_groups_fieldset())
            .with_element(random_galaxy_planet_groups_fieldset())
    }

    fn random_galaxy_clusters_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "System Clusters:",
            "random-galaxy-cluster",
            "System Cluster:",
            ("Remove system cluster", "New system cluster"),
            vec![
                random_galaxy_cluster_capacity_fieldset(),
                random_galaxy_cluster_placement_fieldset(),
                random_galaxy_cluster_system_names_fieldset(),
                random_galaxy_cluster_star_groups_fieldset(),
                random_galaxy_cluster_planet_groups_fieldset(),
            ],
        )
    }

    fn random_galaxy_cluster_capacity_fieldset() -> HtmlElement {
        HtmlElement::new("fieldset")
            .with_element(HtmlElement::new("legend").with_text("System Cluster Capacity:"))
            .with_div(html::page::labeled_min_max(
                (
                    "random-galaxy-cluster-width",
                    "random-galaxy-cluster-height",
                ),
                "cluster size:",
                (1024u32, 1024u32),
                (100u32, 16384u32),
                false,
            ))
            .with_element(html::page::labeled(
                "random-galaxy-cluster-system-count",
                "maximum systems (may generate less):",
                HtmlElement::new("input")
                    .with_attribute("type", "number")
                    .required()
                    .with_attribute("min", 1u32),
            ))
    }

    fn random_galaxy_cluster_placement_fieldset() -> HtmlElement {
        HtmlElement::new("fieldset")
            .with_element(HtmlElement::new("legend").with_text("System Cluster Placement:"))
            .with_div(
                HtmlElement::new("label")
                    .with_text("origin point: ")
                    .with_element(
                        HtmlElement::new("input")
                            .with_class("random-galaxy-cluster-origin-x")
                            .with_name("random-galaxy-cluster-origin-x")
                            .with_id("random-galaxy-cluster-origin-x")
                            .with_attribute("type", "number")
                            .required(),
                    )
                    .with_element(
                        HtmlElement::new("input")
                            .with_class("random-galaxy-cluster-origin-y")
                            .with_name("random-galaxy-cluster-origin-y")
                            .with_id("random-galaxy-cluster-origin-y")
                            .with_attribute("type", "number")
                            .required(),
                    ),
            )
            .with_element(html::page::labeled(
                "random-galaxy-cluster-wormhole",
                "place the wormhole to the cluster in this system:",
                HtmlElement::new("input")
                    .with_attribute("type", "text")
                    .required(),
            ))
            .with_element(html::page::labeled_range(
                "random-galaxy-cluster-max-link-length",
                "maximum hyperspace link length:",
                100u32,
                (40u32, 255u32),
                false,
            ))
            .with_element(html::page::labeled_range(
                "random-galaxy-cluster-link-chance",
                "chance for other systems to link:",
                40u32,
                (0u32, 100u32),
                false,
            ))
            .with_element(html::page::labeled_range(
                "random-galaxy-cluster-minimum-distance",
                "prevent systems within this distance of each other:",
                33.3,
                (16.0, 50.0),
                true,
            ))
            .with_div(html::page::labeled_min_max(
                (
                    "random-galaxy-cluster-step-size-min",
                    "random-galaxy-cluster-step-size-max",
                ),
                "systems are placed within this range of each other:",
                (50.0, 87.5),
                (20.0, 100.0),
                true,
            ))
    }

    fn random_galaxy_cluster_system_names_fieldset() -> HtmlElement {
        HtmlElement::new("fieldset")
            .with_element(HtmlElement::new("legend").with_text("System Cluster System Names:"))
            .with_element(html::page::labeled_range(
                "random-galaxy-cluster-max-system-name-length",
                "maximum system name length:",
                64u32,
                (16u32, 255u32),
                false,
            ))
            .with_element(html::page::labeled(
                "random-galaxy-cluster-system-names-examples-group",
                "example name group to use as a Markov chain:",
                HtmlElement::new("input")
                    .with_attribute("type", "text")
                    .required(),
            ))
    }

    fn random_galaxy_cluster_star_groups_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "System Cluster Star Groups:",
            "random-galaxy-cluster-star-group",
            "System Cluster Star Group:",
            (
                "Remove system cluster star group",
                "New system cluster star group",
            ),
            vec![
                html::page::labeled(
                    "random-galaxy-cluster-star-group-name",
                    "star group name:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                html::page::labeled(
                    "random-galaxy-cluster-star-group-can-be-binary",
                    "can be part of a dual-star system:",
                    HtmlElement::new("input")
                        .with_attribute("type", "checkbox")
                        .checked(),
                ),
                html::page::weight("random-galaxy-cluster-star-group-weight", None),
                html::page::labeled(
                    "random-galaxy-cluster-star-group-max-planets",
                    "maximum planets in its system:",
                    HtmlElement::new("input")
                        .with_attribute("type", "number")
                        .required()
                        .with_attributes(vec![("value", 5u32), ("min", 0u32), ("max", 255u32)]),
                ),
            ],
        )
    }

    fn random_galaxy_cluster_planet_groups_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "System Cluster Planet Groups:",
            "random-galaxy-cluster-planet-group",
            "System Cluster Planet Group:",
            (
                "Remove system cluster planet group",
                "New system cluster planet group",
            ),
            vec![
                html::page::labeled(
                    "random-galaxy-cluster-planet-group-name",
                    "planet group name:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                html::page::weight("random-galaxy-cluster-planet-group-weight", None),
                html::page::labeled_min_max(
                    (
                        "random-galaxy-cluster-planet-group-distance-range-percentage-min",
                        "random-galaxy-cluster-planet-group-distance-range-percentage-max",
                    ),
                    "spawns within this percentage range of 2x the habitable zone:",
                    (50.0, 65.0),
                    (0.0, 100.0),
                    true,
                ),
                html::page::labeled_range(
                    "random-galaxy-cluster-planet-moon-chance",
                    "Chance to have a moon:",
                    12.5,
                    (0.0, 100.0),
                    false,
                ),
                random_galaxy_cluster_planet_moons_fieldset(),
            ],
        )
    }

    fn random_galaxy_cluster_planet_moons_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Planet groups as moons:",
            "random-galaxy-cluster-planet-moon-group",
            "Planet groups as moon:",
            ("Remove moon", "New moon"),
            vec![
                html::page::labeled(
                    "random-galaxy-cluster-planet-moon-group-name",
                    "planet group name:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                html::page::weight("random-galaxy-cluster-planet-moon-weight", None),
            ],
        )
    }

    fn random_galaxy_system_name_sources_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Example name groups:",
            "random-galaxy-system-name-examples-group",
            "Example name group:",
            ("Remove example name group", "New example name group"),
            vec![
                html::page::labeled(
                    "random-galaxy-system-name-examples-group-name",
                    "example name group name:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                random_galaxy_system_name_source_fieldset(),
            ],
        )
    }

    fn random_galaxy_system_name_source_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Example names for systems to use via Markov chain:",
            "random-galaxy-system-name-examples",
            "Example name:",
            ("Remove example name", "New example name"),
            vec![html::page::labeled(
                "random-galaxy-system-name-example",
                "name:",
                HtmlElement::new("input")
                    .with_attribute("type", "text")
                    .required(),
            )],
        )
    }

    fn random_galaxy_star_groups_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Star groups:",
            "random-galaxy-star-group",
            "Star group:",
            ("Remove star group", "New star group"),
            vec![
                html::page::labeled(
                    "random-galaxy-star-group-name",
                    "star group name:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                random_galaxy_star_group_fieldset(),
            ],
        )
    }

    fn random_galaxy_star_group_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Stars:",
            "random-galaxy-star",
            "Star:",
            ("Remove star", "New star"),
            vec![
                html::page::labeled(
                    "random-galaxy-star-sprite",
                    "sprite:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                html::page::labeled(
                    "random-galaxy-star-habitable",
                    "habitable zone:",
                    HtmlElement::new("input")
                        .with_attribute("type", "number")
                        .required()
                        .with_attributes(vec![("value", 1000u32), ("min", 0)]),
                ),
                html::page::labeled(
                    "random-galaxy-star-binary-distance",
                    "distance from other star if in a dual-star system:",
                    HtmlElement::new("input")
                        .with_attribute("type", "number")
                        .required()
                        .with_attributes(vec![("value", 160), ("min", 0)]),
                ),
            ],
        )
    }

    fn random_galaxy_planet_groups_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Planet groups:",
            "random-galaxy-planet-group",
            "Planet group:",
            ("Remove planet group", "New planet group"),
            vec![
                html::page::labeled(
                    "random-galaxy-planet-group-name",
                    "planet group name:",
                    HtmlElement::new("input")
                        .with_attribute("type", "text")
                        .required(),
                ),
                random_galaxy_planet_group_fieldset(),
            ],
        )
    }

    fn random_galaxy_planet_group_fieldset() -> HtmlElement {
        html::page::fieldset_group(
            "Planets:",
            "random-galaxy-planet",
            "Planet:",
            ("Remove planet", "New planet"),
            vec![html::page::labeled(
                "random-galaxy-planet-sprite",
                "sprite:",
                HtmlElement::new("input")
                    .with_attribute("type", "text")
                    .required(),
            )],
        )
    }
}
