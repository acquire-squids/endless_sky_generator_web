import init from "./endless_sky_generator_web.js";

import {
  generate_full_map,
  generate_system_shuffler,
  SystemShufflerConfig,
  generate_chaos,
  ChaosConfig
} from "./endless_sky_generator_web.js";

import { readFileAsText } from "./export_to_rust.js";

const wasm = await init();

const input = document.getElementById("input");

const full_map_output = document.getElementById("full-map-output");

let defaults_checked = false;

const default_paths = [];
const default_sources = [];

const loaded_paths = [];
const loaded_sources = [];

input.addEventListener("change", async () => {
  for (const file of event.target.files) {
    if (file.type.startsWith("text")) {
      loaded_paths.push(file.name);
      loaded_sources.push(await readFileAsText(file));
    }
  }
});

const include_defaults = document.getElementById("include-defaults");
const clear_uploads = document.getElementById("clear-uploads");

clear_uploads.addEventListener("click", () => {
  loaded_paths.splice(0, loaded_paths.length);
  loaded_sources.splice(0, loaded_sources.length);
});

const getPathsAndSources = async () => {
  if (!defaults_checked && include_defaults.checked) {
    const es_stable_data_paths = await fetch(new Request("es_stable_data_paths.txt"))
      .then(async (response) => {
        if (!response.ok) {
          // poor man's `Option<T>`: an empty array
          return [];
        } else {
          return [await response.text()];
        }
      });

    if (es_stable_data_paths.length >= 1) {
      for (const path of es_stable_data_paths[0].split("\n")) {
        let sources = await fetch(path)
          .then(async (response) => {
            if (!response.ok) {
              return [];
            } else {
              return [await response.text()];
            }
          });

        if (sources.length >= 1) {
          default_paths.push(path);
          default_sources.push(sources[0]);
        }
      }

      defaults_checked = true;
    }
  }

  return {
    paths: (include_defaults.checked ? default_paths.concat(loaded_paths) : loaded_paths),
    sources: (include_defaults.checked ? default_sources.concat(loaded_sources) : loaded_sources),
  };
};

const downloadZip = (fileName, bytes) => {
  const zipBlob = new Blob(
    [bytes.buffer],
    {
      type: "application/zip"
    }
  );

  const blobUrl = URL.createObjectURL(zipBlob);

  const downloadLink = document.createElement("a");

  downloadLink.setAttribute("href", blobUrl);
  downloadLink.setAttribute("download", fileName);
  downloadLink.click();

  downloadLink.remove();

  URL.revokeObjectURL(blobUrl);
};

const generateAndDownload = (fileName, rustFn) => {
  return async () => {
    const paths_and_sources = await getPathsAndSources();

    let result;

    try {
      result = new Uint8Array(rustFn(paths_and_sources.paths, paths_and_sources.sources));
    } catch(error) {
      console.error(error);
      return;
    }

    downloadZip(fileName, result);
  }
};

full_map_output.addEventListener("click", generateAndDownload("full_map.zip", generate_full_map));

{
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
}

{
  const output = document.getElementById("chaos-output");

  const seed = document.getElementById("chaos-seed");

  output.addEventListener("click", async () => {
    let errored = false;

    if (!seed.checkValidity()) {
      console.error("ERROR: Chaos seed is not a valid value");
      errored = true;
    }

    if (errored) {
      return;
    }

    const paths_and_sources = await getPathsAndSources();

    let result;

    try {
      result = new Uint8Array(
        generate_chaos(
          paths_and_sources.paths,
          paths_and_sources.sources,
          new ChaosConfig(
            seed.value,
          )
        )
      );
    } catch(error) {
      console.error(error);
      return;
    }

    downloadZip("chaos.zip", result);
  });
}
