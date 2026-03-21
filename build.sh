#!/usr/bin/env bash

set -ex

cargo clippy \
  --release

cargo build \
  --release

wasm-bindgen \
  --target web "target/wasm32-unknown-unknown/release/endless_sky_generator_web.wasm" \
  --no-typescript \
  --out-dir "./www"
