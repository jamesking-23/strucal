//! StruCal deterministic calculation engine.
//! Shared by the web app (compiled to WASM) and the native desktop app.
pub mod units;
pub mod trace;
pub mod registry;
pub mod params;
pub mod rc_beam;
pub mod report;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
