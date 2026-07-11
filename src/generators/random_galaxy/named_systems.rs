use crate::{
    generators::random_galaxy::{
        config,
        quad_tree::{BoundingBox, QuadTree},
        vec2f::{self, Vec2f},
    },
    markov::MarkovChain,
    wandom::{XoShiRo256SS, shuffle_index::ShuffleIndex},
};

use std::collections::HashSet;

pub fn plot(
    (system_name_rng, system_placement_rng): (&mut XoShiRo256SS, &mut XoShiRo256SS),
    settings: &config::RandomGalaxyConfig,
) -> Galaxy {
    let mut existing_names = HashSet::new();

    let mut quad_tree = QuadTree::<16, SystemId>::new(
        settings
            .clusters()
            .iter()
            .map(|cluster| {
                BoundingBox::new(
                    (*cluster.placement().origin() - (*cluster.capacity().size() / 2.0)).floor(),
                    cluster.capacity().size().floor(),
                )
            })
            .reduce(|a, b| {
                BoundingBox::new(
                    Vec2f::new(
                        a.top_left().x().min(*b.top_left().x()),
                        a.top_left().y().min(*b.top_left().y()),
                    ),
                    Vec2f::new(
                        a.size().x().max(*b.size().x()),
                        a.size().y().max(*b.size().y()),
                    ),
                )
            })
            .unwrap_or_else(|| BoundingBox::new(Vec2f::new(-1.0, -1.0), Vec2f::new(2.0, 2.0))),
    );

    let mut galaxy_map = Galaxy {
        clusters: vec![],
        systems: vec![],
    };

    for (cluster_index, cluster_config) in settings.clusters().iter().enumerate() {
        galaxy_map.make_cluster(
            (system_name_rng, system_placement_rng),
            settings,
            (cluster_index, cluster_config),
            &mut existing_names,
            &mut quad_tree,
        );
    }

    galaxy_map
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SystemId {
    cluster_index: usize,
    system_index: usize,
}

type SystemClusterTree = QuadTree<16, SystemId>;

pub struct Galaxy {
    clusters: Vec<SystemCluster>,
    systems: Vec<System>,
}

impl Galaxy {
    pub fn systems(&self) -> impl Iterator<Item = (usize, &System)> {
        self.clusters
            .iter()
            .enumerate()
            .flat_map(|(cluster_index, cluster)| {
                cluster
                    .0
                    .iter()
                    .filter_map(|system_index| self.systems.get(*system_index))
                    .map(move |system| (cluster_index, system))
            })
    }

    pub const fn systems_slice(&self) -> &[System] {
        self.systems.as_slice()
    }
}

struct SystemCluster(Vec<usize>);

#[derive(Debug)]
pub struct System {
    pos: Vec2f,
    name: String,
    links: Vec<usize>,
}

impl System {
    pub const fn pos(&self) -> Vec2f {
        self.pos
    }

    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    pub const fn links(&self) -> &[usize] {
        self.links.as_slice()
    }
}

impl Galaxy {
    const MAX_NAME_ATTEMPTS: usize = 64;

    fn make_cluster(
        &mut self,
        (system_name_rng, system_placement_rng): (&mut XoShiRo256SS, &mut XoShiRo256SS),
        settings: &config::RandomGalaxyConfig,
        (cluster_index, cluster_config): (usize, &config::Cluster),
        existing_names: &mut HashSet<String>,
        quad_tree: &mut SystemClusterTree,
    ) {
        let mut cluster = SystemCluster(vec![]);

        let names = settings
            .system_name_sources()
            .groups()
            .get(*cluster_config.names().source_index())
            .into_iter()
            .flat_map(crate::generators::random_galaxy::config::SystemNameSource::names)
            .map(String::as_str)
            .collect::<Vec<_>>();

        let names = MarkovChain::new(names.as_slice(), 2);

        let mut system_index = self.systems.len();

        let mut frontier = vec![];

        'attempt_placement: while cluster.0.len() < *cluster_config.capacity().system_count() {
            if let Some(name) = Self::try_new_name(
                system_name_rng,
                cluster_config.names(),
                &names,
                existing_names,
            ) {
                let system = if cluster.0.is_empty() {
                    System {
                        pos: cluster_config.placement().origin().floor(),
                        name,
                        links: vec![],
                    }
                } else if frontier.is_empty() {
                    break 'attempt_placement;
                } else if let Some((pos, linked_to)) = self.try_new_system(
                    system_placement_rng,
                    cluster_config.placement(),
                    quad_tree,
                    &mut frontier,
                ) {
                    System {
                        pos,
                        name,
                        links: if let Some(from_system) = self.systems.get_mut(linked_to) {
                            from_system.links.push(system_index);

                            vec![linked_to]
                        } else {
                            vec![]
                        },
                    }
                } else {
                    break 'attempt_placement;
                };

                quad_tree.insert(
                    SystemId {
                        cluster_index,
                        system_index,
                    },
                    system.pos(),
                );

                frontier.push((system_index, system.pos()));

                self.systems.push(system);

                cluster.0.push(system_index);

                system_index += 1;
            } else {
                break 'attempt_placement;
            }
        }

        let cluster = self.randomly_link(
            (cluster_index, cluster),
            quad_tree,
            system_placement_rng,
            cluster_config.capacity(),
            cluster_config.placement(),
        );

        self.clusters.push(cluster);

        self.check(
            quad_tree,
            cluster_config.capacity(),
            cluster_config.placement(),
        );
    }

    fn try_new_name(
        rng: &mut XoShiRo256SS,
        name_config: &config::SystemNames,
        names: &MarkovChain<'_>,
        existing_names: &mut HashSet<String>,
    ) -> Option<String> {
        for _ in 0..(Self::MAX_NAME_ATTEMPTS) {
            let maybe_name = names.one(rng, |name| {
                name.chars().count() >= usize::from(*name_config.max_length())
            });

            if !existing_names.contains(&maybe_name) {
                existing_names.insert(maybe_name.clone());

                return Some(maybe_name);
            }
        }

        None
    }

    fn try_new_system(
        &self,
        rng: &mut XoShiRo256SS,
        placement: &config::SystemPlacement,
        quad_tree: &SystemClusterTree,
        frontier: &mut Vec<(usize, Vec2f)>,
    ) -> Option<(Vec2f, usize)> {
        let shuffled_frontier_indices = frontier.shuffled_indices_with_rng(rng);

        let mut linked_to = None;
        let mut pos = None;

        let mut failed_frontiers = vec![];

        'select_from_frontier: for frontier_index in shuffled_frontier_indices {
            if let Some((from_system_index, from_pos)) = frontier.get(frontier_index) {
                #[allow(clippy::cast_precision_loss)]
                let random_angle = (rng.rand_range(0, 360_000) as f64) / 1_000.0;

                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                let random_distance = rng.rand_range(
                    *placement.step_size().min() as u64,
                    *placement.step_size().max() as u64,
                ) as f64;

                for random_angle in (0..360).step_by(36).map(f64::from).map(|angle_offset| {
                    (random_angle + angle_offset) * ((2.0 * std::f64::consts::PI) / 360.0)
                }) {
                    let next_pos: Vec2f = *from_pos
                        + (Vec2f::new(
                            random_angle.cos(),
                            // I think we're supposed to negate sine to make negative Y be up
                            -random_angle.sin(),
                        ) * random_distance);

                    let next_pos = next_pos.floor();

                    if quad_tree.bounding_box().contains(next_pos)
                        && self
                            .overlapping_systems(quad_tree, placement, next_pos)
                            .next()
                            .is_none()
                    {
                        linked_to = Some(*from_system_index);

                        pos = Some(next_pos);

                        break 'select_from_frontier;
                    }
                }
            }

            failed_frontiers.push(frontier_index);
        }

        failed_frontiers.sort_unstable();

        for frontier_index in failed_frontiers.into_iter().rev() {
            frontier.swap_remove(frontier_index);
        }

        pos.zip(linked_to)
    }

    const OVERLAP_DISTANCE: f64 = 16.0;
    const OVERLAP_MULTIPLIER: f64 = 2.0;

    fn overlapping_systems(
        &self,
        quad_tree: &SystemClusterTree,
        placement: &config::SystemPlacement,
        pos: Vec2f,
    ) -> impl Iterator<Item = (SystemId, &System)> {
        let query_diameter = (Vec2f::new(
            placement
                .step_size()
                .max()
                .abs()
                .max(*placement.minimum_distance())
                .max(Self::OVERLAP_DISTANCE),
            placement.minimum_distance().max(Self::OVERLAP_DISTANCE),
        ) * 2.0
            * Self::OVERLAP_MULTIPLIER)
            .floor();

        quad_tree
            .query(BoundingBox::new(
                (pos - query_diameter / 2.0).floor(),
                query_diameter,
            ))
            .into_iter()
            .filter_map(|(id, _)| {
                self.systems
                    .get(id.system_index)
                    .map(|system| (*id, system))
            })
            .filter(move |(_, other)| {
                other.pos().distance(pos) < *placement.minimum_distance()
                    || ((other.pos().x - pos.x).abs() < placement.step_size().max().abs()
                        && (other.pos().y - pos.y).abs() < Self::OVERLAP_DISTANCE)
            })
    }

    fn randomly_link(
        &mut self,
        (cluster_index, cluster): (usize, SystemCluster),
        quad_tree: &SystemClusterTree,
        rng: &mut XoShiRo256SS,
        _: &config::SystemCapacity,
        placement: &config::SystemPlacement,
    ) -> SystemCluster {
        let query_diameter = Vec2f::new(
            f64::from(*placement.max_link_length()),
            f64::from(*placement.max_link_length()),
        ) * 2.0;

        for f in &cluster.0 {
            if let Some(from_pos) = self.systems.get(*f).map(System::pos) {
                for t in quad_tree
                    .query(BoundingBox::new(
                        from_pos - query_diameter / 2.0,
                        query_diameter,
                    ))
                    .into_iter()
                    .filter(|(t, _)| t.system_index != *f && t.cluster_index == cluster_index)
                    .map(|(t, _)| t.system_index)
                {
                    #[allow(clippy::cast_precision_loss)]
                    let link_roll = (rng.rand_range(0, 100_000) as f64) / 1_000.0;

                    if link_roll < *placement.link_chance()
                        && let Some(to) = self.systems.get(t)
                        && from_pos.distance(to.pos) <= f64::from(*placement.max_link_length())
                        && let Some(from) = self.systems.get(*f)
                        && !from.links().contains(&t)
                        && self
                            .intersections_with(
                                quad_tree,
                                placement,
                                (*f, from),
                                std::iter::once((t, to)),
                            )
                            .is_empty()
                        && let Some(from) = self.systems.get_mut(*f)
                    {
                        from.links.push(t);

                        if let Some(to) = self.systems.get_mut(t)
                            && !to.links().contains(f)
                        {
                            to.links.push(*f);
                        }
                    }
                }
            }
        }

        cluster
    }

    const INTERSECTION_DISTANCE_MULTIPLIER: f64 = 3.0;

    fn intersections_with<'a>(
        &'a self,
        quad_tree: &SystemClusterTree,
        placement: &config::SystemPlacement,
        (f, from): (usize, &'a System),
        additional: impl Iterator<Item = (usize, &'a System)>,
    ) -> Vec<((usize, usize), (usize, usize))> {
        let query_diameter = Vec2f::new(
            f64::from(*placement.max_link_length()) * Self::INTERSECTION_DISTANCE_MULTIPLIER,
            f64::from(*placement.max_link_length()) * Self::INTERSECTION_DISTANCE_MULTIPLIER,
        ) * 2.0;

        from.links()
            .iter()
            .copied()
            .filter_map(move |t| {
                if t == f {
                    None
                } else {
                    Some((t, self.systems.get(t)?))
                }
            })
            .chain(additional)
            .map(move |(t, to)| ((f, from), (t, to)))
            .flat_map(|((f, from), (t, to))| {
                quad_tree
                    .query(BoundingBox::new(
                        from.pos - query_diameter / 2.0,
                        query_diameter,
                    ))
                    .into_iter()
                    .map(|(other_f, _)| other_f.system_index)
                    .filter_map(move |other_f| {
                        if other_f == f || other_f == t {
                            None
                        } else {
                            Some((((f, from), (t, to)), (other_f, self.systems.get(other_f)?)))
                        }
                    })
            })
            .flat_map(|(((f, from), (t, to)), (other_f, other_from))| {
                other_from
                    .links()
                    .iter()
                    .copied()
                    .filter_map(move |other_t| {
                        if other_t == f || other_t == t || other_t == other_f {
                            None
                        } else {
                            Some((
                                ((f, from), (t, to)),
                                ((other_f, other_from), (other_t, self.systems.get(other_t)?)),
                            ))
                        }
                    })
            })
            .filter(|(((_, from), (_, to)), ((_, other_from), (_, other_to)))| {
                vec2f::intersects((from.pos, to.pos), (other_from.pos, other_to.pos))
            })
            .map(|(((f, _), (t, _)), ((other_f, _), (other_t, _)))| ((f, t), (other_f, other_t)))
            .collect::<Vec<_>>()
    }

    fn invalid_system_indices(
        &self,
        quad_tree: &SystemClusterTree,
        placement: &config::SystemPlacement,
    ) -> Vec<usize> {
        (0..(self.systems.len()))
            .filter(|system_index| {
                self.systems
                    .get(*system_index)
                    .map(System::pos)
                    .is_some_and(|center| {
                        self.overlapping_systems(quad_tree, placement, center)
                            .any(|(id, _)| id.system_index != *system_index)
                    })
            })
            .collect::<Vec<_>>()
    }

    fn check(
        &self,
        quad_tree: &SystemClusterTree,
        _: &config::SystemCapacity,
        placement: &config::SystemPlacement,
    ) {
        println!("{} total systems", self.systems.len());

        let existing = quad_tree.query(quad_tree.bounding_box());

        assert_eq!(self.systems.len(), existing.len(), "System count mismatch");

        let invalid_system_indices = self.invalid_system_indices(quad_tree, placement);

        assert_eq!(
            invalid_system_indices,
            [].as_slice(),
            "{} invalid systems",
            invalid_system_indices.len()
        );

        println!("{} remaining systems", self.systems.len());

        for system in &self.systems {
            for linked in system.links() {
                if system.links().iter().fold(0, |accum, other_linked| {
                    if other_linked == linked {
                        accum + 1
                    } else {
                        accum
                    }
                }) > 1
                {
                    panic!("A system has a duplicate link");
                }
            }
        }
    }
}
