//! StruCal deterministic calculation engine.
//! Shared by the web app (compiled to WASM) and the native desktop app.
pub mod units;
pub mod trace;
pub mod registry;
pub mod params;
pub mod rc_beam;
pub mod report;
pub mod module;
pub mod slab;
pub mod pad;
pub mod column;
pub mod combos;
pub mod steel_beam;
pub mod retaining;
pub mod raft;
pub mod bolted;
pub mod weld;
pub mod steel_column;
pub mod wall;
pub mod stairs;
pub mod strip;
pub mod modules;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod modules_tests;
