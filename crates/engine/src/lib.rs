//! StruCal deterministic calculation engine.
//! Shared by the web app (compiled to WASM) and the native desktop app.
pub mod bolted;
pub mod column;
pub mod combos;
pub mod masonry;
pub mod module;
pub mod modules;
pub mod pad;
pub mod params;
pub mod project;
pub mod punching;
pub mod raft;
pub mod rc_beam;
pub mod registry;
pub mod report;
pub mod retaining;
pub mod slab;
pub mod stairs;
pub mod steel_beam;
pub mod steel_beamcol;
pub mod steel_column;
pub mod steel_tension;
pub mod strip;
pub mod timber_beam;
pub mod timber_col;
pub mod timber_data;
pub mod timber_tension;
pub mod trace;
pub mod units;
pub mod wall;
pub mod weld;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod modules_tests;
#[cfg(test)]
mod robustness_tests;
