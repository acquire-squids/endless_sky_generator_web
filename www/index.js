import init, { find_ships } from "./endless_sky_generator_web.js";

import { println, readFileAsText } from "./export_to_rust.js";

const wasm = await init();

const input = document.getElementById("input");
const output = document.getElementById("output");
const log = document.getElementById("log");

output.addEventListener("click", async () => {
  log.innerText = "";

  const paths = [];
  const sources = [];

  for (const file of input.files) {
    if (file.type.startsWith("text")) {
      paths.push(file.name);
      sources.push(await readFileAsText(file));
    }
  }

  const result = find_ships(paths, sources);

  if (result.errors.length === 0) {
    for (const ship of result.text) {
      println(ship);
    }
  } else {
    println(result.errors);
  }
});
