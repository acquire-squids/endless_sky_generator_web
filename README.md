# Endless Sky Generator (web)
Web version of a generator for some [Endless Sky](<https://github.com/endless-sky/endless-sky>) plugins I made that would be annoying to create by hand.

## Usage
1. Go to [this page](https://acquire-squids.github.io/endless_sky_generator_web/) , or clone the repository and host it as a web server\*.
2. Click the upload button and choose an Endless Sky data folder (you can try to include plugins in the folder, though I offer no guarantees).
3. Press the download button for the plugin(s) you want to generate.
4. Move the zip files you generated to the game's plugin directory.
5. Play the game.

\* Python is an easy way to do this with the command `python3 -m http.server 8080`.
  Current working directory must be the `www/` folder.

## Build Instructions
It's probably possible to build without cargo and rustup, but I wouldn't know.
I use them because I am casual and unconcerned.

If you have cargo and rustup:
1. `rustup target add wasm32-unknown-unknown`
2. `cargo install --version =0.2.105 wasm-bindgen-cli`
3. `cargo build --target wasm32-unknown-unknown --release`
4. `wasm-bindgen --target web target/wasm32-unknown-unknown/release/endless_sky_generator_web.wasm --no-typescript --out-dir "./www"`
5. `rustc -o "list_stable_data_paths" "list_stable_data_paths.rs"`

Steps 3 and 4 can be achieved by running `./build.sh`

You must also place an Endless Sky data folder at `endless-sky/data/` so there is default data.
This can be done with the following commands:
1. `git clone --no-checkout --depth=1 --filter=tree:0 https://github.com/endless-sky/endless-sky.git`
2. `cd endless-sky/`
3. `git sparse-checkout set --no-cone /data`
4. `git checkout`
5. `cp -r data/ ../www/es_stable_data/`
6. `../list_stable_data_paths`

Use `--branch v0.10.16` in the `git clone` to get a tagged release, where `v0.10.16` is your target tag.
Steps 1 through 5 can be achieved by running `./get_stable_es_data.sh`.

## Notes
I'll probably add a few more generators.  The initial goal was just System Shuffler.

Please forgive my code quality.  This is a hobby and nothing more.
