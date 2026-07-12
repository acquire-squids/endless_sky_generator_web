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

  const outfits = Array.from(chaos_form.getElementsByClassName("chaos-outfits"))[0];

  const ships = Array.from(chaos_form.getElementsByClassName("chaos-ships"))[0];

  const systems = Array.from(chaos_form.getElementsByClassName("chaos-systems"))[0];

  const planets = Array.from(chaos_form.getElementsByClassName("chaos-planets"))[0];

  chaos_form.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (!outfits.checked && !ships.checked && !systems.checked && !planets.checked) {
      const invalid = "You should enable at least one of these, otherwise the generator serves no purpose";

      outfits.setCustomValidity(invalid);
      ships.setCustomValidity(invalid);
      systems.setCustomValidity(invalid);
      planets.setCustomValidity(invalid);
    } else {
      outfits.setCustomValidity("");
      ships.setCustomValidity("");
      systems.setCustomValidity("");
      planets.setCustomValidity("");
    }

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
            outfits.checked,
            ships.checked,
            systems.checked,
            planets.checked,
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
