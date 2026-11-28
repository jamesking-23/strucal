//! Module registry: the single entry point used by the web (WASM) and desktop apps.
use crate::module::*;
use crate::rc_beam::{self, BeamInput};
use serde_json::Value;

pub fn beam_def() -> ModDef {
    ModDef {
        id: "beam",
        name: "RC beam",
        title: "Reinforced concrete beam",
        subtitle: "EN 1992-1-1:2004 · Rectangular section, sagging moment and shear, crack control and deflection",
        available: true,
        has_check_mode: true,
        fields: vec![
            fld("b", "Width b", "mm", "Geometry", 300.0),
            fld("h", "Depth h", "mm", "Geometry", 500.0),
            fld("cover", "Cover c", "mm", "Geometry", 30.0),
            fld("link_dia", "Link φ", "mm", "Geometry", 8.0),
            fld("bar_dia", "Tension bar φ", "mm", "Geometry", 20.0),
            fld("fck", "fck", "MPa", "Materials", 30.0),
            fld("fyk", "fyk", "MPa", "Materials", 500.0),
            fld("m_ed", "Moment M Ed", "kNm", "Actions - ULS", 200.0),
            fld("v_ed", "Shear V Ed", "kN", "Actions - ULS", 120.0),
            fld_chk("n_bars", "Tension bars n (drawing)", "bars", "Provided reinforcement", 4.0),
            fld_chk("link_spacing", "Link spacing s (drawing)", "mm", "Provided reinforcement", 200.0),
            fld("span", "Effective span l (deflection)", "mm", "Serviceability", 6000.0),
            fld_opt(
                "k_sys",
                "Structural system K (Table 7.4N)",
                "Serviceability",
                1.0,
                vec![
                    (1.0, "Simply supported (K = 1.0)"),
                    (1.3, "End span of continuous beam (K = 1.3)"),
                    (1.5, "Interior span (K = 1.5)"),
                    (0.4, "Cantilever (K = 0.4)"),
                ],
            ),
            fld("m_qp", "Quasi-permanent moment M qp (SLS)", "kNm", "Serviceability", 120.0),
        ],
    }
}

pub struct Upcoming {
    pub name: &'static str,
    pub note: &'static str,
}
pub fn upcoming() -> Vec<Upcoming> {
    vec![
        Upcoming { name: "Two-way slab", note: "Needs a verified method source" },
        Upcoming { name: "Prestressed concrete", note: "Out of scope for this release" },
    ]
}

pub fn defs() -> Vec<ModDef> {
    vec![
        crate::combos::def(),
        beam_def(),
        crate::steel_beam::def(),
        crate::steel_column::def(),
        crate::steel_beamcol::def(),
        crate::steel_tension::def(),
        crate::timber_beam::def(),
        crate::timber_col::def(),
        crate::timber_tension::def(),
        crate::masonry::def(),
        crate::retaining::def(),
        crate::raft::def(),
        crate::bolted::def(),
        crate::weld::def(),
        crate::column::def(),
        crate::wall::def(),
        crate::slab::def(),
        crate::punching::def(),
        crate::stairs::def(),
        crate::pad::def(),
        crate::strip::def(),
    ]
}

fn beam(v: &Value) -> ModResult {
    let i: BeamInput = match serde_json::from_value(v.clone()) {
        Ok(i) => i,
        Err(e) => return fail(vec![format!("Invalid input: {}", e)]),
    };
    let r = rc_beam::run(&i);
    if !r.ok {
        return fail(r.errors);
    }
    let summary = vec![
        kv("Effective depth d", format!("{:.1} mm", r.d)),
        kv("Required tension steel As,req", format!("{:.0} mm²", r.as_req)),
        kv("Provided tension steel As,prov", format!("{:.0} mm²  ({})", r.as_prov, r.bars)),
        kv("Top steel", r.top_steel.clone()),
        kv("Links", r.links.clone()),
        kv("Lever arm z", format!("{:.1} mm", r.z)),
        kv("μ / μlim", format!("{:.4} / {:.4}", r.mu, r.mu_lim)),
    ];
    ModResult {
        outputs: vec![
            out("m_ed", "Design moment M Ed", i.m_ed, "kNm"),
            out("v_ed", "Support reaction (ULS)", i.v_ed, "kN"),
            out("reaction", "Support reaction (ULS)", i.v_ed, "kN"),
        ],
        ok: true,
        errors: vec![],
        warnings: r.warnings,
        param_status: r.param_status,
        summary,
        checks: r.checks,
        max_util: r.max_util,
        overall_pass: r.overall_pass,
        observations: r.observations,
        trace: r.trace,
        trace_hash: r.trace_hash,
        engine_version: r.engine_version,
        scope_notes: r.scope_notes,
        drawing: Some(Drawing::Beam { b: i.b, h: i.h, cover: i.cover, link_dia: i.link_dia, bar_dia: i.bar_dia, n_bars: r.bar_count }),
    }
}

/// Runs a module by id. Every entry point (WASM, desktop, project engine) goes through here.
pub fn run(id: &str, v: &Value) -> ModResult {
    if !v.is_object() {
        return fail(vec!["Invalid input: expected an object of named input values".into()]);
    }
    guard_finite(dispatch(id, v))
}

/// True when a formatted value contains a non-finite number ("NaN", "inf", "-inf").
pub fn has_non_finite_text(s: &str) -> bool {
    s.split(|c: char| !c.is_ascii_alphanumeric()).any(|w| w.eq_ignore_ascii_case("nan") || w.eq_ignore_ascii_case("inf"))
}

/// Converts a result that contains non-finite numbers into an explicit error, so no UI ever shows or links NaN/inf.
/// Only reachable with physically meaningless input magnitudes; valid results pass through unchanged.
fn guard_finite(r: ModResult) -> ModResult {
    if !r.ok {
        return r;
    }
    let bad = !r.max_util.is_finite()
        || r.outputs.iter().any(|o| !o.value.is_finite())
        || r.checks.iter().any(|c| !c.utilisation.is_finite())
        || r.summary.iter().any(|k| has_non_finite_text(&k.value));
    if bad {
        fail(vec![
            "Numerical result out of range: one or more inputs are outside a physically meaningful magnitude. Check units and values."
                .into(),
        ])
    } else {
        r
    }
}

fn dispatch(id: &str, v: &Value) -> ModResult {
    match id {
        "beam" => beam(v),
        "slab" => crate::slab::run(v),
        "combos" => crate::combos::run(v),
        "steelbeam" => crate::steel_beam::run(v),
        "steelcol" => crate::steel_column::run(v),
        "timber" => crate::timber_beam::run(v),
        "timbercol" => crate::timber_col::run(v),
        "retaining" => crate::retaining::run(v),
        "raft" => crate::raft::run(v),
        "bolted" => crate::bolted::run(v),
        "weld" => crate::weld::run(v),
        "stairs" => crate::stairs::run(v),
        "strip" => crate::strip::run(v),
        "pad" => crate::pad::run(v),
        "column" => crate::column::run(v),
        "wall" => crate::wall::run(v),
        "beamcol" => crate::steel_beamcol::run(v),
        "steeltension" => crate::steel_tension::run(v),
        "timbertension" => crate::timber_tension::run(v),
        "masonry" => crate::masonry::run(v),
        "punching" => crate::punching::run(v),
        _ => fail(vec![format!("Module '{}' is not available yet", id)]),
    }
}
pub fn run_json(id: &str, input: &str) -> String {
    match serde_json::from_str::<Value>(input) {
        Ok(v) => serde_json::to_string(&run(id, &v)).unwrap(),
        Err(e) => serde_json::to_string(&fail(vec![format!("Invalid input: {}", e)])).unwrap(),
    }
}
pub fn defs_json() -> String {
    serde_json::to_string(&defs()).unwrap()
}
pub fn upcoming_json() -> String {
    serde_json::to_string(&upcoming().iter().map(|u| serde_json::json!({"name":u.name,"note":u.note})).collect::<Vec<_>>()).unwrap()
}
pub fn def(id: &str) -> Option<ModDef> {
    defs().into_iter().find(|d| d.id == id)
}
