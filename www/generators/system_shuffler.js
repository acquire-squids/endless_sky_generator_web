import {
  getPathsAndSources,
  downloadZip,
  generateAndDownload
} from "../export_to_rust.js";

import {
  generate_system_shuffler,
  SystemShufflerConfig
} from "../endless_sky_generator_web.js";

export const preparation = () => {
  const output = document.getElementById("system-shuffler-output");

  const seed = document.getElementById("system-shuffler-seed");
  const max_presets = document.getElementById("system-shuffler-max-presets");
  const shuffle_once_on_install = document.getElementById("system-shuffler-shuffle-once-on-install");

  const shuffle_chance = document.getElementById("system-shuffler-shuffle-chance");
  const shuffle_chance_output = document.getElementById("system-shuffler-shuffle-chance-output");

  const share_value = (source, target) => {
    source.addEventListener("input", () => {
      if (source.checkValidity()) {
        target.value = source.value;
      }
    });
  };

  share_value(shuffle_chance, shuffle_chance_output);
  share_value(shuffle_chance_output, shuffle_chance);

  const fixed_shuffle_days = document.getElementById("system-shuffler-fixed-shuffle-days");
  const fixed_shuffle_days_output = document.getElementById("system-shuffler-fixed-shuffle-days-output");

  share_value(fixed_shuffle_days, fixed_shuffle_days_output);
  share_value(fixed_shuffle_days_output, fixed_shuffle_days);

  output.addEventListener("click", async () => {
    let errored = false;

    if (!seed.checkValidity()) {
      console.error("ERROR: System Shuffler seed is not a valid value");
      errored = true;
    }

    if (!max_presets.checkValidity()) {
      console.error("ERROR: System Shuffler max presets is not a valid value");
      errored = true;
    }

    if (!shuffle_chance_output.checkValidity()) {
      console.error("ERROR: System Shuffler chance is not a valid value");
      errored = true;
    }

    if (!fixed_shuffle_days_output.checkValidity()) {
      console.error("ERROR: System Shuffler fixed shuffle days is not a valid value");
      errored = true;
    }

    if (errored) {
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
            shuffle_chance_output.value,
            fixed_shuffle_days_output.value,
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
