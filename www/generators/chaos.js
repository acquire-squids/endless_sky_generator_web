import {
  getPathsAndSources,
  downloadZip,
  generateAndDownload,
  iterateElements,
  defaultEventListeners
} from "../export_to_rust.js";

import {
  generate_chaos,
  ChaosConfig
} from "../endless_sky_generator_web.js";

export const preparation = () => {
  const chaos_form = document.getElementById("chaos-form");

  iterateElements(chaos_form, (node) => {
    defaultEventListeners(node);
  });

  const seed = Array.from(chaos_form.getElementsByClassName("chaos-seed"))[0];

  chaos_form.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (!chaos_form.checkValidity()) {
      chaos_form.reportValidity();
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
