import {
  getPathsAndSources,
  downloadZip,
  generateAndDownload,
  iterateElements,
  defaultEventListeners
} from "../export_to_rust.js";

import {
  generate_random_galaxy,
  Vec2f,
  RandomGalaxyConfig,
  Cluster,
  SystemCapacity,
  SystemPlacement,
  SystemNames,
  SystemContents,
  ClusterStarGroup,
  ClusterPlanetGroup,
  PlanetMoons,
  PlanetMoon,
  SystemNameSources,
  SystemNameSource,
  Sprites,
  GalaxySprite,
  Stars,
  StarGroup,
  Star,
  Planets,
  PlanetGroup,
  MinMax
} from "../endless_sky_generator_web.js";

export const preparation = () => {
  const random_galaxy_form = document.getElementById("random-galaxy-form");

  iterateElements(random_galaxy_form, (node) => {
    defaultEventListeners(node);
  });

  const collect_example_name_groups = () => {
    return Array.from(random_galaxy_form
      .getElementsByClassName("random-galaxy-system-name-examples-group"))
      .map((example_name_group) => {
        const collected_example_name_group = {
          "group_name": Array.from(example_name_group.getElementsByClassName("random-galaxy-system-name-examples-group-name")),
          "names": Array.from(example_name_group.getElementsByClassName("random-galaxy-system-name-example"))
        };

        return collected_example_name_group;
      });
  };

  const collect_star_groups = () => {
    return Array.from(random_galaxy_form
      .getElementsByClassName("random-galaxy-star-group"))
      .map((star_group) => {
        const collected_star_group = {
          "group_name": Array.from(star_group.getElementsByClassName("random-galaxy-star-group-name")),
          "stars": Array.from(star_group.getElementsByClassName("random-galaxy-star")).map(
            (star) => {
              const collected_star = {
                "sprite": Array.from(star.getElementsByClassName("random-galaxy-star-sprite")),
                "habitable": Array.from(star.getElementsByClassName("random-galaxy-star-habitable")),
                "binary_distance": Array.from(star.getElementsByClassName("random-galaxy-star-binary-distance"))
              };

              return collected_star;
            }
          )
        };

        return collected_star_group;
      });
  };

  const collect_planet_groups = () => {
    return Array.from(random_galaxy_form
      .getElementsByClassName("random-galaxy-planet-group"))
      .map((planet_group) => {
        const collected_planet_group = {
          "group_name": Array.from(planet_group.getElementsByClassName("random-galaxy-planet-group-name")),
          "planets": Array.from(planet_group.getElementsByClassName("random-galaxy-planet")).map(
            (planet) => {
              const collected_planet = {
                "sprite": Array.from(planet.getElementsByClassName("random-galaxy-planet-sprite"))
              };

              return collected_planet;
            }
          )
        };

        return collected_planet_group;
      });
  };

  const collect_clusters = () => {
    return Array.from(random_galaxy_form
      .getElementsByClassName("random-galaxy-cluster"))
      .map((cluster) => {
        const collected_cluster = {
          "capacity": {
            "width": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-width")),
            "height": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-height")),
            "system_count": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-system-count"))
          },
          "placement": {
            "origin_x": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-origin-x")),
            "origin_y": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-origin-y")),
            "wormhole": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-wormhole")),
            "max_link_length": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-max-link-length")),
            "link_chance": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-link-chance")),
            "minimum_distance": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-minimum-distance")),
            "step_size_min": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-step-size-min")),
            "step_size_max": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-step-size-max"))
          },
          "system_names": {
            "max_length": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-max-system-name-length")),
            "group_name": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-system-names-examples-group"))
          },
          "star_groups": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-star-group")).map(
            (star_group) => {
              const collected_star_group = {
                "group_name": Array.from(star_group.getElementsByClassName("random-galaxy-cluster-star-group-name")),
                "can_be_binary": Array.from(star_group.getElementsByClassName("random-galaxy-cluster-star-group-can-be-binary")),
                "weight": Array.from(star_group.getElementsByClassName("random-galaxy-cluster-star-group-weight")),
                "max_planets": Array.from(star_group.getElementsByClassName("random-galaxy-cluster-star-group-max-planets"))
              };

              return collected_star_group;
            }
          ),
          "planet_groups": Array.from(cluster.getElementsByClassName("random-galaxy-cluster-planet-group")).map(
            (planet_group) => {
              const distance_range_percentage = {
                "min": Array.from(planet_group.getElementsByClassName("random-galaxy-cluster-planet-group-distance-range-percentage-min")),
                "max": Array.from(planet_group.getElementsByClassName("random-galaxy-cluster-planet-group-distance-range-percentage-max")),
              };

              const collected_planet_group = {
                "group_name": Array.from(planet_group.getElementsByClassName("random-galaxy-cluster-planet-group-name")),
                "weight": Array.from(planet_group.getElementsByClassName("random-galaxy-cluster-planet-group-weight")),
                "distance_range_percentage": distance_range_percentage,
                "moon_chance": Array.from(planet_group.getElementsByClassName("random-galaxy-cluster-planet-moon-chance")),
                "moons": Array.from(planet_group.getElementsByClassName("random-galaxy-cluster-planet-moon-group")).map(
                  (moon_group) => {
                    const collected_moon_group = {
                      "group_name": Array.from(moon_group.getElementsByClassName("random-galaxy-cluster-planet-moon-group-name")),
                      "weight": Array.from(moon_group.getElementsByClassName("random-galaxy-cluster-planet-moon-weight"))
                    };

                    return collected_moon_group;
                  }
                )
              };

              return collected_planet_group;
            }
          )
        };

        return collected_cluster;
      });
  };

  const name = Array.from(random_galaxy_form.getElementsByClassName("random-galaxy-name"))[0];

  const sprite = Array.from(random_galaxy_form.getElementsByClassName("random-galaxy-sprite"))[0];
  let sprite_name;
  let sprite_bytes;

  sprite.addEventListener("change", async () => {
    if (sprite.files.length > 0) {
      sprite_name = sprite.files.item(sprite.length - 1).name;
      sprite_bytes = await sprite.files.item(sprite.length - 1).bytes();
    }
  });

  const seed = Array.from(random_galaxy_form.getElementsByClassName("random-galaxy-seed"))[0];

  const reveal_all = Array.from(random_galaxy_form.getElementsByClassName("random-galaxy-reveal-all"))[0];

  random_galaxy_form.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (sprite_name === undefined || sprite_bytes === undefined) {
      sprite.setCustomValidity("You never chose a galaxy sprite!");
    } else {
      sprite.setCustomValidity("");
    }

    const star_groups = collect_star_groups();
    const planet_groups = collect_planet_groups();
    const example_name_groups = collect_example_name_groups();
    const clusters = collect_clusters();

    clusters.forEach((cluster) => {
      const system_name_group = cluster.system_names;

      if (!example_name_groups.some(
        (example_name_group) => example_name_group.group_name[0].value
          == system_name_group.group_name[0].value
      )) {
        system_name_group.group_name[0].setCustomValidity("You have not included an example name group with this name!");
      } else {
        system_name_group.group_name[0].setCustomValidity("");
      }

      cluster.star_groups
        .forEach((cluster_star_group) => {
          if (!star_groups.some(
            (star_group) => star_group.group_name[0].value
              == cluster_star_group.group_name[0].value
          )) {
            cluster_star_group.group_name[0].setCustomValidity("You have not included a star group with this name!");
          } else {
            cluster_star_group.group_name[0].setCustomValidity("");
          }
        });

      cluster.planet_groups
        .forEach((cluster_planet_group) => {
          if (!planet_groups.some(
            (planet_group) => planet_group.group_name[0].value
              == cluster_planet_group.group_name[0].value
          )) {
            cluster_planet_group.group_name[0].setCustomValidity("You have not included a planet group with this name!");
          } else {
            cluster_planet_group.group_name[0].setCustomValidity("");

            cluster_planet_group.moons.forEach((moon_planet_group) => {
              if (!planet_groups.some(
                (planet_group) => planet_group.group_name[0].value
                  == moon_planet_group.group_name[0].value
              )) {
                moon_planet_group.group_name[0].setCustomValidity("You have not included a planet group with this name!");
              } else {
                moon_planet_group.group_name[0].setCustomValidity("");
              }
            })
          }
        });
    });

    if (!random_galaxy_form.checkValidity()) {
      iterateElements(random_galaxy_form, (node) => {
        if (typeof node.checkValidity !== "undefined" && !node.checkValidity()) {
          let outer_node = node.parentElement;

          do {
            if (outer_node === null) {
              break;
            }

            if (outer_node.tagName === "DETAILS") {
              outer_node.toggleAttribute("open", true);
            }
          } while (outer_node = outer_node.parentElement);
        }
      });

      random_galaxy_form.reportValidity();

      return;
    }

    const paths_and_sources = await getPathsAndSources();

    let result;

    try {
      result = new Uint8Array(
        generate_random_galaxy(
          paths_and_sources.paths,
          paths_and_sources.sources,
          new RandomGalaxyConfig(
            name.value,
            seed.value,
            reveal_all.checked,
            clusters
              .map((cluster) => new Cluster(
                new SystemCapacity(
                  new Vec2f(
                    cluster.capacity.width[0].value,
                    cluster.capacity.height[0].value,
                  ),
                  cluster.capacity.system_count[0].value,
                ),
                new SystemPlacement(
                  new Vec2f(
                    cluster.placement.origin_x[0].value,
                    cluster.placement.origin_y[0].value,
                  ),
                  cluster.placement.wormhole[0].value,
                  cluster.placement.max_link_length[0].value,
                  cluster.placement.link_chance[0].value,
                  cluster.placement.minimum_distance[0].value,
                  new MinMax(
                    cluster.placement.step_size_min[0].value,
                    cluster.placement.step_size_max[0].value,
                  ),
                ),
                new SystemNames(
                  example_name_groups.findIndex(
                    (example_name_group) => example_name_group.group_name[0].value
                      == cluster.system_names.group_name[0].value
                  ),
                  cluster.system_names.max_length[0].value,
                ),
                new SystemContents(
                  cluster.star_groups
                    .map((cluster_star_group) => new ClusterStarGroup(
                      star_groups.findIndex(
                        (star_group) => star_group.group_name[0].value
                          == cluster_star_group.group_name[0].value
                      ),
                      cluster_star_group.can_be_binary[0].value,
                      cluster_star_group.weight[0].value,
                      cluster_star_group.max_planets[0].value,
                    )),
                  cluster.planet_groups
                    .map((cluster_planet_group) => new ClusterPlanetGroup(
                      planet_groups.findIndex(
                        (planet_group) => planet_group.group_name[0].value
                          == cluster_planet_group.group_name[0].value
                      ),
                      cluster_planet_group.weight[0].value,
                      new MinMax(
                        cluster_planet_group.distance_range_percentage.min[0].value,
                        cluster_planet_group.distance_range_percentage.max[0].value,
                      ),
                      new PlanetMoons(
                        cluster_planet_group.moon_chance[0].value,
                        cluster_planet_group.moons
                          .map((moon) => new PlanetMoon(
                            planet_groups.findIndex(
                              (planet_group) => planet_group.group_name[0].value
                                == moon.group_name[0].value
                            ),
                            moon.weight[0].value,
                          )),
                      ),
                    )),
                ),
              )),
            new SystemNameSources(
              collect_example_name_groups()
                .map((example_name_group) => new SystemNameSource(
                  example_name_group.group_name[0].value,
                  example_name_group.names
                    .map((example_name) => example_name.value),
                )),
            ),
            new Sprites(
              new GalaxySprite(
                sprite_name,
                sprite_bytes,
              ),
              new Stars(
                collect_star_groups()
                  .map((star_group) => new StarGroup(
                    star_group.group_name[0].value,
                    star_group.stars
                      .map((star) => new Star(
                        star.sprite[0].value,
                        star.habitable[0].value,
                        star.binary_distance[0].value,
                      )),
                  )),
              ),
              new Planets(
                collect_planet_groups()
                  .map((planet_group) => new PlanetGroup(
                    planet_group.group_name[0].value,
                    planet_group.planets
                      .map((planet) => planet.sprite[0].value),
                  )),
              ),
            )
          ),
        )
      );
    } catch(error) {
      console.error(error);
      return;
    }

    downloadZip("random_galaxy_" + name.value + ".zip", result);
  });
};
