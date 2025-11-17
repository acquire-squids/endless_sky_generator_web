import init, { generate_template } from "./endless_sky_generator_web.js";

import { println, readFileAsText } from "./export_to_rust.js";

const wasm = await init();

const input = document.getElementById("input");
const log = document.getElementById("log");

const template_output = document.getElementById("template-output");

const generateAndDownload = (fileName, rustFn) => {
  return async () => {
    log.innerText = "";

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
      println("ERROR: " + error);
      return;
    }

    const zipBlob = new Blob(result);

    const blobUrl = URL.createObjectURL(zipBlob);

    const downloadLink = document.createElement("a");

    downloadLink.setAttribute("href", blobUrl);
    downloadLink.setAttribute("download", fileName);
    downloadLink.click();

    downloadLink.remove();

    URL.revokeObjectURL(blobUrl);
  }
}

template_output.addEventListener("click", generateAndDownload("template_plugin.zip", generate_template));
