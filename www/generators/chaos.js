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
  const form = document.getElementById("chaos-form");

  const seed = document.getElementById("chaos-seed");

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
