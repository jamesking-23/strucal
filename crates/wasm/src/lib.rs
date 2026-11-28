use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn modules() -> String {
    strucal_engine::modules::defs_json()
}
#[wasm_bindgen]
pub fn upcoming() -> String {
    strucal_engine::modules::upcoming_json()
}
#[wasm_bindgen]
pub fn run_module(id: &str, input_json: &str) -> String {
    strucal_engine::modules::run_json(id, input_json)
}
#[wasm_bindgen]
pub fn report_html(id: &str, input_json: &str) -> String {
    strucal_engine::report::html_json(id, input_json)
}
#[wasm_bindgen]
pub fn param_sets() -> String {
    serde_json::to_string(&strucal_engine::params::available()).unwrap()
}
#[wasm_bindgen]
pub fn formula_registry() -> String {
    serde_json::to_string(&strucal_engine::registry::formulas()).unwrap()
}
#[wasm_bindgen]
pub fn engine_version() -> String {
    strucal_engine::ENGINE_VERSION.to_string()
}
#[wasm_bindgen]
pub fn analyse_project(project_json: &str, previous_json: &str) -> String {
    strucal_engine::project::analyse_json(project_json, previous_json)
}
