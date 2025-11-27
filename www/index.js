import init from "./endless_sky_generator_web.js";

import {
  generate_template,
  generate_full_map,
  generate_system_shuffler,
  SystemShufflerConfig
} from "./endless_sky_generator_web.js";

import { readFileAsText } from "./export_to_rust.js";

const wasm = await init();

const input = document.getElementById("input");

const template_output = document.getElementById("template-output");
const full_map_output = document.getElementById("full-map-output");

const generateAndDownload = (fileName, rustFn) => {
  return async () => {
    const paths = [];
    const sources = [];

    for (const file of input.files) {
      if (file.type.startsWith("text")) {
        paths.push(file.name);
        sources.push(await readFileAsText(file));
      }
    }

    let result;

    try {
      result = new Uint8Array(rustFn(paths, sources));
    } catch(error) {
      console.error(error);
      return;
    }

    const zipBlob = new Blob(
      [result.buffer],
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
  }
};

template_output.addEventListener("click", generateAndDownload("template_plugin.zip", generate_template));
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

    const paths = [];
    const sources = [];

    for (const file of input.files) {
      if (file.type.startsWith("text")) {
        paths.push(file.name);
        sources.push(await readFileAsText(file));
      }
    }

    let result;

    try {
      result = new Uint8Array(
        generate_system_shuffler(
          paths,
          sources,
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

    const zipBlob = new Blob(
      [result.buffer],
      {
        type: "application/zip"
      }
    );

    const blobUrl = URL.createObjectURL(zipBlob);

    const downloadLink = document.createElement("a");

    downloadLink.setAttribute("href", blobUrl);
    downloadLink.setAttribute("download", "system_shuffler.zip");
    downloadLink.click();

    downloadLink.remove();

    URL.revokeObjectURL(blobUrl);
  });
}
