pub mod config;
mod named_systems;
pub mod quad_tree;
pub mod vec2f;

use self::{
    named_systems::{Galaxy, System},
    vec2f::Vec2f,
};

use crate::{
    generators,
    wandom::{XoShiRo256SS, shuffle_index::ShuffleIndex, weighted_choice::WeightedChoice},
    zippy::Zip,
};

use endless_sky_rw::{
    Data, DataFolder, Node, NodeIndex, SourceIndex, Span, Token, tree_from_tokens,
};

use std::{error::Error, path::PathBuf};

const PLUGIN_NAME: &str = "Random Galaxy";

const PLUGIN_DESCRIPTION: &str = "\
    A randomly generated galaxy\n\
";

const PLUGIN_VERSION: &str = "0.1.0";

#[allow(clippy::missing_errors_doc)]
pub fn process_data(
    data_folder: &DataFolder,
    settings: config::RandomGalaxyConfig,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let _ = data_folder.data();

    let mut output = vec![];

    let mut rng = XoShiRo256SS::new(*settings.seed());

    let mut generator = RandomGalaxy {
        archive: Zip::new(&mut output),
        output_data: Data::default(),
        settings,
    };

    generator.description()?;

    generator.archive.write_dir("data/")?;

    generator.galaxy(&mut rng)?;

    generator.archive.write_dir("images/")?;

    generator.archive.write_dir("images/ui/")?;

    generator.archive.write_file(
        format!(
            "images/ui/{}",
            generator.settings.sprites().galaxy().sprite_name()
        ),
        generator.settings.sprites().galaxy().blob(),
    )?;

    generator.archive.finish()?;

    Ok(output)
}

struct GeneratedStar {
    habitable: u32,
    max_planets: u8,
}

struct GeneratedPlanet {
    sprite: String,
    distance: f64,
    period: f64,
    offset: f64,
    moons: Vec<Self>,
}

struct RandomGalaxy<'a> {
    archive: Zip<'a>,
    output_data: Data,
    settings: config::RandomGalaxyConfig,
}

impl RandomGalaxy<'_> {
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

    fn galaxy(&mut self, rng: &mut XoShiRo256SS) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();
        let galaxy_source = self.output_data.insert_source(String::new());

        let galaxy_center = Vec2f::average(
            self.settings
                .clusters()
                .iter()
                .map(|cluster| *cluster.placement().origin())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let galaxy_node = tree_from_tokens!(
            &mut self.output_data; galaxy_source =>
            : "galaxy", self.settings.name() ;
            {
                : "pos", galaxy_center.x, galaxy_center.y ;
                :
                    "sprite",
                    format!(
                        "ui/{}",
                        self.settings
                            .sprites()
                            .galaxy()
                            .sprite_name()
                            .rsplit_once('.')
                            .map_or_else(
                                || {
                                    self.settings
                                        .sprites()
                                        .galaxy()
                                        .sprite_name()
                                        .as_str()
                                },
                                |(sprite, _)| sprite
                            )
                    ) ;
            }
        );

        self.output_data.push_root_node(galaxy_source, galaxy_node);

        self.systems(rng, output_root_node_count, galaxy_source)
    }

    fn systems(
        &mut self,
        rng: &mut XoShiRo256SS,
        output_root_node_count: usize,
        galaxy_source: SourceIndex,
    ) -> Result<(), Box<dyn Error>> {
        let mut system_name_rng = XoShiRo256SS::new(rng.step());
        let mut system_placement_rng = XoShiRo256SS::new(rng.step());

        let mut star_rng = XoShiRo256SS::new(rng.step());
        let mut planet_rng = XoShiRo256SS::new(rng.step());

        let galaxy = named_systems::plot(
            (&mut system_name_rng, &mut system_placement_rng),
            &self.settings,
        );

        let origins = self.settings.clusters().iter().enumerate().fold(
            vec![],
            |mut accum, (wormhole_index, cluster)| {
                accum.push((
                    *cluster.placement().origin(),
                    format!("{} {}", self.wormhole_name(wormhole_index), accum.len()),
                    cluster.placement().wormhole().clone(),
                ));

                accum
            },
        );

        let mut wormholes = vec![];

        for (cluster_index, system) in galaxy.systems() {
            self.system(
                if let Some((_, wormhole_name, from)) =
                    origins.iter().find(|(pos, _, _)| *pos == system.pos())
                {
                    wormholes.push((wormhole_name.as_str(), from.as_str(), system.name()));

                    Some(wormhole_name)
                } else {
                    None
                },
                galaxy_source,
                &galaxy,
                (cluster_index, system),
                (&mut star_rng, &mut planet_rng),
            );
        }

        for (wormhole_name, from, _) in &wormholes {
            let system_node = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "system", from ;
                {
                    : "add", "object", wormhole_name ;
                    {
                        : "sprite", "planet/wormhole" ;
                        : "distance", 7168.0 ;
                        : "period", 8192.0 ;
                    }
                }
            );

            self.output_data.push_root_node(galaxy_source, system_node);
        }

        self.zip_root_nodes("data/map_systems.txt", output_root_node_count)?;

        if *self.settings.reveal_all() {
            self.reveal(galaxy_source, &galaxy, wormholes.as_slice())?;
        }

        self.wormholes(rng, galaxy_source, wormholes.as_slice())
    }

    fn reveal(
        &mut self,
        galaxy_source: SourceIndex,
        galaxy: &Galaxy,
        wormholes: &[(&str, &str, &str)],
    ) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        let name = format!("Random Galaxy: Reveal {}", self.settings.name());

        let visit_event = tree_from_tokens!(
            &mut self.output_data; galaxy_source =>
            : "event", name.as_str() ;
        );

        for (wormhole_name, _, _) in wormholes {
            let visit = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "visit planet", wormhole_name ;
            );

            self.output_data.push_child(visit_event, visit);
        }

        for (_, system) in galaxy.systems() {
            let visit = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "visit", system.name() ;
            );

            self.output_data.push_child(visit_event, visit);
        }

        self.output_data.push_root_node(galaxy_source, visit_event);

        let visit_mission = tree_from_tokens!(
            &mut self.output_data; galaxy_source =>
            : "mission", name.as_str() ;
            {
                : "invisible" ;
                : "non-blocking" ;
                : "landing" ;
                : "on", "offer" ;
                {
                    : "event", name.as_str(), "0" ;
                }
            }
        );

        self.output_data
            .push_root_node(galaxy_source, visit_mission);

        self.zip_root_nodes("data/reveal.txt", output_root_node_count)
    }

    fn system(
        &mut self,
        wormhole: Option<&str>,
        galaxy_source: SourceIndex,
        galaxy: &Galaxy,
        (cluster_index, system): (usize, &System),
        (star_rng, planet_rng): (&mut XoShiRo256SS, &mut XoShiRo256SS),
    ) {
        let system_node = tree_from_tokens!(
            &mut self.output_data; galaxy_source =>
            : "system", system.name() ;
            {
                : "display name", system.name() ;
                : "pos", system.pos().x, system.pos().y ;
            }
        );

        if let Some(wormhole) = wormhole {
            let wormhole = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "object", wormhole ;
                {
                    : "sprite", "planet/wormhole" ;
                    : "distance", 7168.0 ;
                    : "period", 8192.0 ;
                }
            );

            self.output_data.push_child(system_node, wormhole);
        }

        let mut links = system
            .links()
            .iter()
            .filter_map(|linked| galaxy.systems_slice().get(*linked))
            .map(named_systems::System::name)
            .collect::<Vec<_>>();

        links.sort_unstable();

        for link in links {
            let link = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "link", link ;
            );

            self.output_data.push_child(system_node, link);
        }

        if let Some(star) = self.star(galaxy_source, cluster_index, system_node, star_rng) {
            self.planets(galaxy_source, cluster_index, system_node, &star, planet_rng);
        }

        self.output_data.push_root_node(galaxy_source, system_node);
    }

    // TODO: split into more functions
    #[allow(clippy::too_many_lines)]
    fn star(
        &mut self,
        galaxy_source: SourceIndex,
        cluster_index: usize,
        system_node: NodeIndex,
        star_rng: &mut XoShiRo256SS,
    ) -> Option<GeneratedStar> {
        let star_groups = self
            .settings
            .clusters()
            .get(cluster_index)
            .into_iter()
            .flat_map(|cluster| cluster.contents().stars().iter())
            .map(|star_group| (star_group, *star_group.weight()))
            .collect::<Vec<_>>();

        let binary_star_groups = star_groups
            .clone()
            .into_iter()
            .filter(|(star_group, _)| *star_group.can_be_binary())
            .collect::<Vec<_>>();

        let star_group = star_groups.choose_with_rng(star_rng)?;

        let star = self
            .settings
            .sprites()
            .stars()
            .groups()
            .get(*star_group.group_index())?;

        let star = star
            .stars()
            .shuffled_indices_with_rng(star_rng)
            .first()
            .and_then(|star_index| star.stars().get(*star_index))?;

        let maybe_second_star = if *star_group.can_be_binary() && star_rng.rand_range(0, 8) < 3 {
            binary_star_groups
                .choose_with_rng(star_rng)
                .and_then(|second_star_group| {
                    Some((
                        second_star_group,
                        self.settings
                            .sprites()
                            .stars()
                            .groups()
                            .get(*second_star_group.group_index())?,
                    ))
                })
                .and_then(|(second_star_group, second_star)| {
                    Some((
                        second_star_group,
                        second_star
                            .stars()
                            .shuffled_indices_with_rng(star_rng)
                            .first()
                            .and_then(|second_star_index| {
                                second_star.stars().get(*second_star_index)
                            })?,
                    ))
                })
        } else {
            None
        };

        #[allow(clippy::cast_precision_loss)]
        let star_period = (star_rng.rand_range(0, 10_000) as f64) / 1_000.0;

        if let Some((second_star_group, second_star)) = maybe_second_star {
            #[allow(clippy::cast_precision_loss)]
            let second_star_period = (star_rng.rand_range(0, 10_000) as f64) / 1_000.0;

            let star_node = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "object" ;
                {
                    : "sprite", star.sprite_name() ;
                    : "distance", star.binary_distance().max(*second_star.binary_distance()) ;
                    : "period", star_period ;
                    : "offset", 0.0 ;
                }
            );

            self.output_data.push_child(system_node, star_node);

            let second_star_node = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "object" ;
                {
                    : "sprite", second_star.sprite_name() ;
                    : "distance", second_star.binary_distance().max(*star.binary_distance()) ;
                    : "period", second_star_period ;
                    : "offset", 180.0 ;
                }
            );

            self.output_data.push_child(system_node, second_star_node);

            Some(GeneratedStar {
                habitable: *star.habitable().max(second_star.habitable()),
                max_planets: *second_star_group
                    .max_planets()
                    .min(star_group.max_planets()),
            })
        } else {
            let star_node = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "object" ;
                {
                    : "sprite", star.sprite_name() ;
                    : "distance", 0.0 ;
                    : "period", star_period ;
                    : "offset", 0.0 ;
                }
            );

            self.output_data.push_child(system_node, star_node);

            Some(GeneratedStar {
                habitable: *star.habitable(),
                max_planets: *star_group.max_planets(),
            })
        }
    }

    fn planets(
        &mut self,
        galaxy_source: SourceIndex,
        cluster_index: usize,
        system_node: NodeIndex,
        star: &GeneratedStar,
        planet_rng: &mut XoShiRo256SS,
    ) {
        let planet_groups = self
            .settings
            .clusters()
            .get(cluster_index)
            .into_iter()
            .flat_map(|cluster| cluster.contents().planets().iter())
            .map(|planet_group| (planet_group, *planet_group.weight()))
            .collect::<Vec<_>>();

        let mut planets: Vec<GeneratedPlanet> = vec![];

        for _ in 0..(planet_rng.rand_range(0, star.max_planets.into())) {
            if let Some(planet) = self.planet(
                star,
                planet_rng,
                planets.as_slice(),
                planet_groups.as_slice(),
            ) {
                planets.push(planet);
            }
        }

        for planet in planets {
            let planet_node = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "object" ;
                {
                    : "sprite", planet.sprite ;
                    : "distance", planet.distance ;
                    : "period", planet.period ;
                    : "offset", planet.offset ;
                }
            );

            self.output_data.push_child(system_node, planet_node);
        }
    }

    fn planet(
        &self,
        star: &GeneratedStar,
        planet_rng: &mut XoShiRo256SS,
        planets: &[GeneratedPlanet],
        planet_groups: &[(&config::ClusterPlanetGroup, u32)],
    ) -> Option<GeneratedPlanet> {
        let planet_group = planet_groups.choose_with_rng(planet_rng)?;

        let planet = self
            .settings
            .sprites()
            .planets()
            .groups()
            .get(*planet_group.group_index())?;

        let planet = planet
            .sprite_names()
            .shuffled_indices_with_rng(planet_rng)
            .first()
            .and_then(|planet_index| planet.sprite_names().get(*planet_index))?;

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let distance = planet_rng.rand_range(
            (f64::from(star.habitable)
                * 2.0
                * (planet_group.distance_range_percentage().min() / 100.0)) as u64,
            (f64::from(star.habitable)
                * 2.0
                * (planet_group.distance_range_percentage().max() / 100.0)) as u64,
        ) as f64;

        #[allow(clippy::cast_precision_loss)]
        let period = ((2.0 * std::f64::consts::PI * distance) / 965.0)
            .mul_add(365.0, (planet_rng.rand_range(0, 128) as f64) - 64.0);

        let mut planet = GeneratedPlanet {
            sprite: planet.clone(),
            distance,
            period,
            offset: 0.0,
            moons: vec![],
        };

        while planets.iter().any(|other_planet| {
            (other_planet.distance - planet.distance).abs() < 240.0
                && (((other_planet.offset - planet.offset) % 360.0 + 360.0) % 360.0).abs() < 12.0
        }) {
            planet.offset = ((planet.offset + 12.0) % 360.0 + 360.0) % 360.0;

            if planet.offset < 6.0 {
                return None;
            }
        }

        self.moons(planet_rng, planet_group, &mut planet.moons);

        Some(planet)
    }

    fn moons(
        &self,
        planet_rng: &mut XoShiRo256SS,
        planet_group: &config::ClusterPlanetGroup,
        planet_moons: &mut Vec<GeneratedPlanet>,
    ) {
        #[allow(clippy::cast_precision_loss)]
        let moon_roll = planet_rng.rand_range(0, 100) as f64;

        if !planet_group.moons().from_planet_groups().is_empty()
            && moon_roll < *planet_group.moons().chance()
            && let Some(moon_planet_group) = planet_group
                .moons()
                .from_planet_groups()
                .iter()
                .map(|moon_planet_group| {
                    (
                        *moon_planet_group.planet_group_index(),
                        *moon_planet_group.weight(),
                    )
                })
                .collect::<Vec<_>>()
                .shuffled_indices_with_rng(planet_rng)
                .first()
                .and_then(|moon_planet_group_index| {
                    self.settings
                        .sprites()
                        .planets()
                        .groups()
                        .get(*moon_planet_group_index)
                })
            && let Some(moon_planet) = moon_planet_group
                .sprite_names()
                .shuffled_indices_with_rng(planet_rng)
                .first()
                .and_then(|planet_index| moon_planet_group.sprite_names().get(*planet_index))
        {
            #[allow(clippy::cast_precision_loss)]
            let moon_period = planet_rng.rand_range(10, 50) as f64;

            planet_moons.push(GeneratedPlanet {
                sprite: moon_planet.clone(),
                distance: 160.0,
                period: moon_period,
                offset: 0.0,
                moons: vec![],
            });
        }
    }

    fn wormholes(
        &mut self,
        _rng: &mut XoShiRo256SS,
        galaxy_source: SourceIndex,
        wormholes: &[(&str, &str, &str)],
    ) -> Result<(), Box<dyn Error>> {
        let output_root_node_count = self.output_data.root_nodes().len();

        for (wormhole_name, from, to) in wormholes {
            let wormhole_planet = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "planet", wormhole_name ;
                {
                    : "wormhole", wormhole_name ;
                }
            );

            self.output_data
                .push_root_node(galaxy_source, wormhole_planet);

            let wormhole = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "wormhole", wormhole_name ;
            );

            let link_from = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "link", from, to ;
            );

            self.output_data.push_child(wormhole, link_from);

            let link_to = tree_from_tokens!(
                &mut self.output_data; galaxy_source =>
                : "link", to, from ;
            );

            self.output_data.push_child(wormhole, link_to);

            self.output_data.push_root_node(galaxy_source, wormhole);
        }

        self.zip_root_nodes("data/map_wormholes.txt", output_root_node_count)
    }

    fn wormhole_name(&self, wormhole_index: usize) -> String {
        format!("{} Pathway {wormhole_index}", self.settings.name())
    }
}
