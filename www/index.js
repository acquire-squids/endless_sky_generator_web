import init from "./endless_sky_generator_web.js";

import {
  generate_template,
  generate_full_map
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
