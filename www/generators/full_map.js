import {
  generateAndDownload
} from "../export_to_rust.js";

import {
  generate_full_map
} from "../endless_sky_generator_web.js";

export const preparation = () => {
  const full_map_output = document.getElementById("full-map-output");

  full_map_output.addEventListener("click", generateAndDownload("full_map.zip", generate_full_map));
};
