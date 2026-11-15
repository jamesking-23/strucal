use wasm_bindgen::prelude::*;

#[wasm_bindgen] pub fn run_rc_beam(input_json: &str) -> String { strucal_engine::rc_beam::run_json(input_json) }
#[wasm_bindgen] pub fn param_sets() -> String { serde_json::to_string(&strucal_engine::params::available()).unwrap() }
#[wasm_bindgen] pub fn formula_registry() -> String { serde_json::to_string(&strucal_engine::registry::formulas()).unwrap() }
#[wasm_bindgen] pub fn engine_version() -> String { strucal_engine::ENGINE_VERSION.to_string() }
