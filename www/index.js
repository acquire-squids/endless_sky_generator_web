import init from "./endless_sky_generator_web.js";

const wasm = await init();

import {
  preparation as full_map_preparation
} from "./generators/full_map.js";

import {
  preparation as system_shuffler_preparation
} from "./generators/system_shuffler.js";

import {
  preparation as chaos_preparation
} from "./generators/chaos.js";

full_map_preparation();
system_shuffler_preparation();
chaos_preparation();

