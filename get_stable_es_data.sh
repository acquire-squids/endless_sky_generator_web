#!/usr/bin/env bash

set -ex

git clone \
  --no-checkout \
  --depth=1 \
  --filter=tree:0 \
  --branch $(< "stable_version.txt") \
  https://github.com/endless-sky/endless-sky.git

cd endless-sky/

git sparse-checkout set --no-cone /data

git checkout

cp -r data/ ../www/es_stable_data/
