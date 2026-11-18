//! Generic module interface shared by all calculation modules (web + desktop render from this).
use crate::rc_beam::{Check, Obs};
use crate::trace::Trace;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, Debug)]
pub struct Field { pub key: &'static str, pub label: &'static str, pub unit: &'static str, pub group: &'static str, pub default: f64, pub check_only: bool }
#[derive(Serialize, Clone, Debug)]
pub struct ModDef { pub id: &'static str, pub name: &'static str, pub title: &'static str, pub subtitle: &'static str, pub available: bool, pub has_check_mode: bool, pub fields: Vec<Field> }
#[derive(Serialize, Clone, Debug)]
pub struct KV { pub label: String, pub value: String }

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum Drawing {
    Beam { b: f64, h: f64, cover: f64, link_dia: f64, bar_dia: f64, n_bars: u32 },
    Slab { h: f64, cover: f64, bar_dia: f64, spacing: f64 },
    Pad { bx: f64, by: f64, cx: f64, cy: f64, h: f64 },
    Bolts { w: f64, n1: u32, n2: u32, e1: f64, p1: f64, e2: f64, p2: f64, d: f64 },
    Weld { lw: f64, a: f64 },
    Raft { l: f64, w: f64, n: u32, spacing: f64, edge: f64 },
    Retain { hs: f64, tb: f64, ts: f64, lt: f64, b: f64 },
    ISection { h: f64, b: f64, tw: f64, tf: f64 },
    Wall { h: f64, cover: f64, bar_dia: f64, spacing: f64 },
    Strip { width: f64, t: f64, h: f64 },
    Column { b: f64, h: f64, cover: f64, link_dia: f64, bar_dia: f64, n_face: u32 },
}

#[derive(Serialize, Debug, Default)]
pub struct ModResult {
    pub ok: bool, pub errors: Vec<String>, pub warnings: Vec<String>, pub param_status: String,
    pub summary: Vec<KV>, pub checks: Vec<Check>, pub max_util: f64, pub overall_pass: bool, pub observations: Vec<Obs>,
    pub trace: Trace, pub trace_hash: String, pub engine_version: String, pub scope_notes: Vec<String>, pub drawing: Option<Drawing>,
}

pub fn kv(l: &str, v: String) -> KV { KV { label: l.into(), value: v } }
pub fn fail(errors: Vec<String>) -> ModResult { ModResult { errors, engine_version: crate::ENGINE_VERSION.into(), ..Default::default() } }
pub fn num(v: &Value, k: &str) -> Option<f64> { v.get(k).and_then(|x| x.as_f64()) }
pub fn chk(name: &str, reference: &str, demand: String, capacity: String, util: f64, pass: bool) -> Check {
    let u = if util.is_finite() { util.min(9.99) } else { 9.99 };
    Check { name: name.into(), reference: reference.into(), demand, capacity, utilisation: u, pass }
}
pub fn fld(key: &'static str, label: &'static str, unit: &'static str, group: &'static str, default: f64) -> Field { Field { key, label, unit, group, default, check_only: false } }
pub fn fld_chk(key: &'static str, label: &'static str, unit: &'static str, group: &'static str, default: f64) -> Field { Field { key, label, unit, group, default, check_only: true } }

/// Validates the common required numeric fields; returns values or errors.
pub fn require(v: &Value, def: &ModDef, check_mode: bool) -> Result<Vec<f64>, Vec<String>> {
    let mut out = vec![]; let mut errs = vec![];
    for f in &def.fields {
        if f.check_only && !check_mode { out.push(f.default); continue; }
        match num(v, f.key) { Some(x) if x.is_finite() && x > 0.0 => out.push(x), Some(x) if x == 0.0 && matches!(f.key, "qk" | "gk" | "q_sur" | "hw") => out.push(0.0), _ => { errs.push(format!("{} must be a positive number", f.label)); out.push(0.0) } }
    }
    if errs.is_empty() { Ok(out) } else { Err(errs) }
}

/// Assemble the final result: pass/fail, observations, hash.
pub fn finish(mut t: Trace, checks: Vec<Check>, param_status: &str, summary: Vec<KV>, warnings: Vec<String>, extra_obs: Vec<(&str, String)>, notes: Vec<String>, drawing: Drawing) -> ModResult {
    let overall = checks.iter().all(|c| c.pass);
    let max_util = checks.iter().map(|c| c.utilisation).fold(0.0, f64::max);
    t.step("Conclusion", "Overall result of implemented checks", "", "", &format!("{} checks evaluated; maximum utilisation {:.2}", checks.len(), max_util), if overall { "PASS" } else { "FAIL" }, "Applies only to the checks listed, within the stated assumptions.");
    let mut obs: Vec<(&str, String)> = vec![];
    if param_status.contains("NOT") { obs.push(("AMBER", format!("Parameter set: {}. Confirm the governing National Annex before issue for construction.", param_status))); }
    for c in &checks { if !c.pass { obs.push(("RED", format!("{} fails ({} vs {}).", c.name, c.demand, c.capacity))); } else if c.utilisation > 0.85 { obs.push(("AMBER", format!("{} utilisation {:.2} exceeds the 0.85 review threshold.", c.name, c.utilisation))); } }
    obs.extend(extra_obs);
    let observations = obs.into_iter().enumerate().map(|(k, (l, tx))| Obs { id: format!("OBS-{:02}", k + 1), level: l.into(), text: tx }).collect();
    let hash = t.hash();
    ModResult { ok: true, errors: vec![], warnings, param_status: param_status.into(), summary, checks, max_util, overall_pass: overall, observations, trace: t, trace_hash: hash, engine_version: crate::ENGINE_VERSION.into(), scope_notes: notes, drawing: Some(drawing) }
}

/// Singly-reinforced rectangular flexure per EN 1992-1-1 6.1. Returns (As_req, z, mu).
pub fn flex_single(t: &mut Trace, sec: &str, b: f64, d: f64, fck: f64, fcd: f64, fyd: f64, m_nmm: f64, label: &str) -> Result<(f64, f64, f64), String> {
    let mu = m_nmm / (b * d * d * fcd);
    let mu_lim = 0.8 * 0.45 * (1.0 - 0.4 * 0.45);
    t.step(sec, &format!("Normalised moment{}", label), "EN 1992-1-1 cl. 3.1.7, 6.1", "μ = M_{Ed} / (b·d^{2}·f_{cd})", &format!("= {:.2}×10^{{6}} / ({} × {:.1}^{{2}} × {:.2})", m_nmm / 1e6, b, d, fcd), &format!("μ = {:.4}  ({} μ_{{lim}} = {:.4})", mu, if mu <= mu_lim { "≤" } else { ">" }, mu_lim), "Limit from x/d ≤ 0.45, cl. 5.5(4).");
    let _ = fck;
    if mu > mu_lim { return Err("Section too shallow for singly reinforced design (μ > μ_lim): increase thickness".into()); }
    let z = (d / 2.0 * (1.0 + (1.0 - 2.0 * mu).sqrt())).min(0.95 * d);
    let as_req = m_nmm / (fyd * z);
    t.step(sec, &format!("Lever arm and required steel{}", label), "EN 1992-1-1 cl. 6.1", "z = (d/2)·[1 + √(1 − 2μ)] ≤ 0.95d;   A_{s,req} = M_{Ed}/(f_{yd}·z)", &format!("z = ({:.1}/2) × [1 + √(1 − 2 × {:.4})] = {:.1} mm;  = {:.2}×10^{{6}} / ({:.2} × {:.1})", d, mu, z, m_nmm / 1e6, fyd, z), &format!("A_{{s,req}} = {:.0} mm^{{2}}", as_req), "");
    Ok((as_req, z, mu))
}

/// Shear resistance without shear reinforcement, EN 1992-1-1 6.2.2(1). Returns (VRd,c [N], k, rho).
pub fn vrd_c(t: &mut Trace, sec: &str, b: f64, d: f64, fck: f64, gamma_c: f64, as_prov: f64, label: &str) -> (f64, f64, f64) {
    let k = (1.0 + (200.0 / d).sqrt()).min(2.0);
    let rho = (as_prov / (b * d)).min(0.02);
    let crd = 0.18 / gamma_c;
    let vmin = 0.035 * k.powf(1.5) * fck.sqrt();
    let v1 = crd * k * (100.0 * rho * fck).powf(1.0 / 3.0);
    let v = v1.max(vmin) * b * d;
    t.step(sec, &format!("Shear resistance without shear reinforcement{}", label), "EN 1992-1-1 cl. 6.2.2(1), Eq 6.2a, 6.3N", "k = 1 + √(200/d) ≤ 2;  ρ_l = A_{s}/(b·d) ≤ 0.02;  C_{Rd,c} = 0.18/γ_c;  v_{min} = 0.035·k^{3/2}·√f_{ck};  V_{Rd,c} = max[C_{Rd,c}·k·(100ρ_l f_{ck})^{1/3} ; v_{min}]·b·d",
        &format!("k = 1 + √(200/{:.1}) = {:.3};  ρ_l = {:.0}/({} × {:.1}) = {:.5};  C_Rd,c = 0.18/{:.2} = {:.4};  v_min = {:.3};  first term = {:.3} MPa", d, k, as_prov, b, d, rho, gamma_c, crd, vmin, v1), &format!("V_{{Rd,c}} = {:.1} kN", v / 1e3), "");
    (v, k, rho)
}

/// Singly reinforced one-way member per metre width: flexure design, bar spacing, resistance, shear. Used by slab-type members.
/// Returns (As,prov mm2/m, spacing mm, d mm) and appends flexure and shear checks.
#[allow(clippy::too_many_arguments)]
pub fn slab_member(t: &mut Trace, sec: &str, name: &str, m_knm: f64, v_kn: f64, h_mm: f64, cover: f64, bar: f64, fck: f64, fcd: f64, fyd: f64, fyk: f64, gamma_c: f64, checks: &mut Vec<Check>) -> Result<(f64, f64, f64), String> {
    let d = h_mm - cover - bar / 2.0; let b = 1000.0;
    let m_use = m_knm.max(0.0);
    let (as_req, _, _) = flex_single(t, sec, b, d, fck, fcd, fyd, m_use * 1e6, &format!(" ({})", name))?;
    let fctm = 0.30 * fck.powf(2.0 / 3.0);
    let as_min = (0.26 * fctm / fyk * b * d).max(0.0013 * b * d);
    let as_des = as_req.max(as_min); let a1 = std::f64::consts::PI * bar * bar / 4.0; let s_max = (3.0 * h_mm).min(400.0);
    let s = ((1000.0 * a1 / as_des).min(s_max) / 25.0).floor() * 25.0;
    if s < 50.0 { return Err(format!("{}: required bar spacing below 50 mm - increase bar size or thickness", name)); }
    let as_prov = 1000.0 * a1 / s;
    let xx = as_prov * fyd / (0.8 * b * fcd); let mrd = as_prov * fyd * (d - 0.4 * xx);
    t.step(sec, &format!("Reinforcement ({})", name), "EN 1992-1-1 cl. 9.2.1.1, 9.3.1.1(3)", "A_{s,min} = max[0.26·(f_{ctm}/f_{yk})·b·d ; 0.0013·b·d];   s ≤ 1000·A_{bar}/A_{s,design};  s ≤ min(3h ; 400);   M_{Rd} = A_{s,prov}·f_{yd}·(d − 0.4x)", &format!("d = {:.1} mm;  A_s,min = {:.0};  A_s,req = {:.0};  A_bar = {:.1} mm^{{2}};  s = {:.0} mm;  A_s,prov = {:.0} mm^{{2}}/m;  x = {:.1} mm", d, as_min, as_req, a1, s, as_prov, xx), &format!("T{} @ {:.0} mm c/c;  M_{{Rd}} = {:.1} kNm/m", bar, s, mrd / 1e6), "");
    checks.push(chk(&format!("{}: flexure", name), "EN 1992-1-1 6.1", format!("{:.1} kNm/m", m_use), format!("{:.1} kNm/m", mrd / 1e6), m_use * 1e6 / mrd, m_use * 1e6 <= mrd && xx / d <= 0.45 + 1e-9));
    let (vrdc, _, _) = vrd_c(t, sec, b, d, fck, gamma_c, as_prov, &format!(" ({})", name));
    checks.push(chk(&format!("{}: shear", name), "EN 1992-1-1 6.2.2(1)", format!("{:.1} kN/m", v_kn), format!("{:.1} kN/m", vrdc / 1e3), v_kn * 1e3 / vrdc, v_kn * 1e3 <= vrdc));
    Ok((as_prov, s, d))
}
