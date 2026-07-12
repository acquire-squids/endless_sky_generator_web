pub mod config;
pub mod generators;
pub mod html;
mod macros;
mod markov;
mod wandom;
mod zippy;

const GAME_VERSION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/www/stable_version.txt"
));

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod web;
