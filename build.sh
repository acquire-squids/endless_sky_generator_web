#!/usr/bin/env bash

set -ex

# If default data is desired, it must be found at `./endless-sky/data/` !!
rustc -o "list_stable_data_paths" "list_stable_data_paths.rs"

./list_stable_data_paths

cargo clippy \
  --target wasm32-unknown-unknown \
  --release

cargo build \
  --target wasm32-unknown-unknown \
  --release

wasm-bindgen \
  --target web "target/wasm32-unknown-unknown/release/endless_sky_generator_web.wasm" \
  --no-typescript \
  --out-dir "./www"
