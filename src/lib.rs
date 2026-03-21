pub mod config;
pub mod generators;
mod macros;
mod markov;
mod wandom;
mod zippy;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod web;
