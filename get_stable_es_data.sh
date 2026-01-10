#!/usr/bin/env bash

set -ex

# current stable at time of writing is v0.10.16
git clone \
  --no-checkout \
  --depth=1 \
  --filter=tree:0 \
  --branch v0.10.16 \
  https://github.com/endless-sky/endless-sky.git

cd endless-sky/

git sparse-checkout set --no-cone /data

git checkout

cp -r data/ ../www/es_stable_data/
