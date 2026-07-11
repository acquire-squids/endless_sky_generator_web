import {
  getPathsAndSources,
  downloadZip,
  generateAndDownload,
  iterateElements,
  defaultEventListeners
} from "../export_to_rust.js";

import {
  generate_system_shuffler,
  SystemShufflerConfig
} from "../endless_sky_generator_web.js";

export const preparation = () => {
  const system_shuffler_form = document.getElementById("system-shuffler-form");

  iterateElements(system_shuffler_form, (node) => {
    defaultEventListeners(node);
  });

  const seed = Array.from(system_shuffler_form.getElementsByClassName("system-shuffler-seed"))[0];

  const max_presets = Array.from(system_shuffler_form.getElementsByClassName("system-shuffler-max-presets"))[0];

  const shuffle_once_on_install = Array.from(system_shuffler_form.getElementsByClassName("system-shuffler-shuffle-once-on-install"))[0];

  const shuffle_chance = Array.from(system_shuffler_form.getElementsByClassName("system-shuffler-shuffle-chance"))[0];

  const fixed_shuffle_days = Array.from(system_shuffler_form.getElementsByClassName("system-shuffler-fixed-shuffle-days"))[0];

  system_shuffler_form.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (!system_shuffler_form.checkValidity()) {
      system_shuffler_form.reportValidity();
      return;
    }

    const paths_and_sources = await getPathsAndSources();

    let result;

    try {
      result = new Uint8Array(
        generate_system_shuffler(
          paths_and_sources.paths,
          paths_and_sources.sources,
          new SystemShufflerConfig(
            seed.value,
            max_presets.value,
            shuffle_chance.value,
            fixed_shuffle_days.value,
            shuffle_once_on_install.checked,
          )
        )
      );
    } catch(error) {
      console.error(error);
      return;
    }

    downloadZip("system_shuffler.zip", result);
  });
};
