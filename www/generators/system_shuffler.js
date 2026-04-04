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
  const form = document.getElementById("system-shuffler-form");

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

  form.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (!form.checkValidity()) {
      form.reportValidity();
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
