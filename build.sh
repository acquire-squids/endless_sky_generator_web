#!/usr/bin/env bash

set -ex

cargo clippy \
  --release

cargo build \
  --release

cargo run \
  --target host-tuple \
  --release \
  --bin page_generator

mv output/index.html www/index.html

wasm-bindgen \
  --target web "target/wasm32-unknown-unknown/release/endless_sky_generator_web.wasm" \
  --no-typescript \
  --out-dir "./www"
