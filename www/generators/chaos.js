import {
  generateAndDownload,
  getPathsAndSources,
  downloadZip
} from "../export_to_rust.js";

import {
  generate_chaos,
  ChaosConfig
} from "../endless_sky_generator_web.js";

export const preparation = () => {
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
};
