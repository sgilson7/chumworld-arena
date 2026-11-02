//! tool_core: every rule of the duel. No input or output.
//!
//! Builder requirement: this crate performs no input or output. It takes
//! values and returns values, so `cargo test` reaches every rule without a
//! browser. `tests/boundary.rs` enforces that, along with the dependency
//! allowlist and the reference ban on floats, hash maps and clocks.
//!
//! M0: CHUMWORLD's combat (commit d9cb5b5), copied in and cut down to one
//! stone ring and two fighters (CA-2; SECOND-ORDER.md R2.x). Where to start
//! reading: `api.rs` is the door the shim calls through, `game.rs` the
//! screens and the clock, `world.rs` the ring and the order of one tick,
//! `render.rs` what the page draws.

pub mod api;
pub mod arm;
pub mod blade;
pub mod camera;
pub mod feel;
pub mod flicker;
pub mod fx;
pub mod game;
pub mod geom;
pub mod glossary;
pub mod health;
pub mod input;
pub mod look;
pub mod mesh;
pub mod models;
pub mod motion;
pub mod page;
pub mod pilot;
pub mod player;
pub mod pose;
pub mod prim;
pub mod render;
pub mod ring;
pub mod rng;
pub mod sky;
pub mod swing;
pub mod ticks;
pub mod world;
pub mod wounds;
