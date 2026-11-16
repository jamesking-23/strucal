//! Module registry: the single entry point used by the web (WASM) and desktop apps.
use crate::module::*;
use crate::rc_beam::{self, BeamInput};
use serde_json::Value;

pub fn beam_def() -> ModDef {
    ModDef { id: "beam", name: "RC beam", title: "Reinforced concrete beam", subtitle: "EN 1992-1-1:2004 · Rectangular section, sagging moment and shear", available: true, has_check_mode: true, fields: vec![
        fld("b", "Width b", "mm", "Geometry", 300.0), fld("h", "Depth h", "mm", "Geometry", 500.0), fld("cover", "Cover c", "mm", "Geometry", 30.0), fld("link_dia", "Link φ", "mm", "Geometry", 8.0), fld("bar_dia", "Tension bar φ", "mm", "Geometry", 20.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0),
        fld("m_ed", "Moment M Ed", "kNm", "Actions - ULS", 200.0), fld("v_ed", "Shear V Ed", "kN", "Actions - ULS", 120.0),
        fld_chk("n_bars", "Tension bars n (drawing)", "bars", "Provided reinforcement", 4.0), fld_chk("link_spacing", "Link spacing s (drawing)", "mm", "Provided reinforcement", 200.0),
    ] }
}

pub struct Upcoming { pub name: &'static str, pub note: &'static str }
pub fn upcoming() -> Vec<Upcoming> { vec![ Upcoming { name: "Two-way slab", note: "Needs a verified method source" }] }

pub fn defs() -> Vec<ModDef> { vec![beam_def(), crate::column::def(), crate::wall::def(), crate::slab::def(), crate::stairs::def(), crate::pad::def(), crate::strip::def()] }

fn beam(v: &Value) -> ModResult {
    let i: BeamInput = match serde_json::from_value(v.clone()) { Ok(i) => i, Err(e) => return fail(vec![format!("Invalid input: {}", e)]) };
    let r = rc_beam::run(&i);
    if !r.ok { return fail(r.errors); }
    let summary = vec![kv("Effective depth d", format!("{:.1} mm", r.d)), kv("Required tension steel As,req", format!("{:.0} mm²", r.as_req)), kv("Provided tension steel As,prov", format!("{:.0} mm²  ({})", r.as_prov, r.bars)), kv("Top steel", r.top_steel.clone()), kv("Links", r.links.clone()), kv("Lever arm z", format!("{:.1} mm", r.z)), kv("μ / μlim", format!("{:.4} / {:.4}", r.mu, r.mu_lim))];
    ModResult { ok: true, errors: vec![], warnings: r.warnings, param_status: r.param_status, summary, checks: r.checks, max_util: r.max_util, overall_pass: r.overall_pass, observations: r.observations, trace: r.trace, trace_hash: r.trace_hash, engine_version: r.engine_version, scope_notes: r.scope_notes,
        drawing: Some(Drawing::Beam { b: i.b, h: i.h, cover: i.cover, link_dia: i.link_dia, bar_dia: i.bar_dia, n_bars: r.bar_count }) }
}

pub fn run(id: &str, v: &Value) -> ModResult {
    match id { "beam" => beam(v), "slab" => crate::slab::run(v), "stairs" => crate::stairs::run(v), "strip" => crate::strip::run(v), "pad" => crate::pad::run(v), "column" => crate::column::run(v), "wall" => crate::wall::run(v), _ => fail(vec![format!("Module '{}' is not available yet", id)]) }
}
pub fn run_json(id: &str, input: &str) -> String {
    match serde_json::from_str::<Value>(input) { Ok(v) => serde_json::to_string(&run(id, &v)).unwrap(), Err(e) => serde_json::to_string(&fail(vec![format!("Invalid input: {}", e)])).unwrap() }
}
pub fn defs_json() -> String { serde_json::to_string(&defs()).unwrap() }
pub fn upcoming_json() -> String { serde_json::to_string(&upcoming().iter().map(|u| serde_json::json!({"name":u.name,"note":u.note})).collect::<Vec<_>>()).unwrap() }
pub fn def(id: &str) -> Option<ModDef> { defs().into_iter().find(|d| d.id == id) }
