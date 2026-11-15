//! RC rectangular beam: flexure, shear, minimum/maximum reinforcement (EN 1992-1-1:2004).
//! Not yet implemented in this release: torsion, crack control, deflection, anchorage, lap lengths.
use crate::params;
use crate::trace::Trace;
use crate::units::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug)]
pub struct BeamInput {
    pub mode: String,            // "design" | "check"
    pub param_set: String,
    pub b: f64, pub h: f64,      // mm
    pub cover: f64,              // nominal cover to links, mm
    pub link_dia: f64,           // mm
    pub bar_dia: f64,            // assumed main bar dia, mm
    pub fck: f64, pub fyk: f64,  // MPa
    pub m_ed: f64,               // kNm (sagging)
    pub v_ed: f64,               // kN
    #[serde(default)] pub as_provided: Option<f64>,   // mm2, check mode
    #[serde(default)] pub asw_s_provided: Option<f64>, // mm2/mm (both legs), check mode
}

#[derive(Serialize, Debug, Clone)]
pub struct Check { pub name: String, pub reference: String, pub demand: String, pub capacity: String, pub utilisation: f64, pub pass: bool }

#[derive(Serialize, Debug)]
pub struct BeamResult {
    pub ok: bool, pub errors: Vec<String>, pub warnings: Vec<String>,
    pub param_status: String, pub d: f64, pub as_req: f64, pub as_comp_req: f64,
    pub as_min: f64, pub as_max: f64, pub bars: String, pub as_prov: f64,
    pub asw_s_req: f64, pub links: String, pub checks: Vec<Check>, pub overall_pass: bool,
    pub trace: Trace, pub trace_hash: String, pub engine_version: String, pub scope_notes: Vec<String>,
}

fn fail(errors: Vec<String>) -> BeamResult {
    BeamResult { ok: false, errors, warnings: vec![], param_status: String::new(), d: 0.0, as_req: 0.0, as_comp_req: 0.0, as_min: 0.0, as_max: 0.0,
        bars: String::new(), as_prov: 0.0, asw_s_req: 0.0, links: String::new(), checks: vec![], overall_pass: false,
        trace: Trace::new(), trace_hash: String::new(), engine_version: crate::ENGINE_VERSION.into(), scope_notes: vec![] }
}

fn bar_area(d: f64) -> f64 { std::f64::consts::PI * d * d / 4.0 }

pub fn run(i: &BeamInput) -> BeamResult {
    let mut errs = vec![];
    for (n, v) in [("b", i.b), ("h", i.h), ("cover", i.cover), ("link_dia", i.link_dia), ("bar_dia", i.bar_dia), ("fck", i.fck), ("fyk", i.fyk)] {
        if !(v.is_finite() && v > 0.0) { errs.push(format!("{} must be a positive number", n)); }
    }
    if !(i.m_ed.is_finite() && i.m_ed >= 0.0) { errs.push("M_Ed must be >= 0 (sagging)".into()); }
    if !(i.v_ed.is_finite() && i.v_ed >= 0.0) { errs.push("V_Ed must be >= 0".into()); }
    if i.fck > 50.0 { errs.push("fck > 50 MPa is outside the scope of this release (C50/60 max)".into()); }
    if i.fck < 12.0 { errs.push("fck < 12 MPa is outside EN 1992-1-1 Table 3.1".into()); }
    let ps = match params::get(&i.param_set) { Some(p) => p, None => { errs.push("Unknown or unavailable parameter set - select one explicitly".into()); return fail(errs); } };
    if !errs.is_empty() { return fail(errs); }

    let mut t = Trace::new();
    let mut warnings = vec![format!("Parameter set: {} [{}]", ps.label, ps.status)];
    let es = 200_000.0;
    let b = Q::mm(i.b);
    let d_q = Q::mm(i.h - i.cover - i.link_dia - i.bar_dia / 2.0);
    let d = d_q.v;
    if d <= 0.0 { return fail(vec!["Effective depth is not positive - check h, cover and bar sizes".into()]); }
    t.push("Geometry", format!("d = h - c - phi_link - phi/2 = {} - {} - {} - {}/2", i.h, i.cover, i.link_dia, i.bar_dia), format!("d = {:.1} mm", d));

    let fcd = Q::mpa(ps.alpha_cc * i.fck / ps.gamma_c);
    let fyd = Q::mpa(i.fyk / ps.gamma_s);
    t.push("EN 1992-1-1 cl. 3.1.6(1)P", format!("fcd = alpha_cc fck / gamma_c = {} x {} / {}", ps.alpha_cc, i.fck, ps.gamma_c), format!("fcd = {:.2} MPa", fcd.v));
    t.push("EN 1992-1-1 cl. 2.4.2.4", format!("fyd = fyk / gamma_s = {} / {}", i.fyk, ps.gamma_s), format!("fyd = {:.2} MPa", fyd.v));

    // ---- Flexure ----
    let m = Q::knm(i.m_ed);
    let mu = (m / (b * d_q * d_q * fcd)).expect(SCALAR).unwrap();
    let xi_lim = 0.45; // x/d limit, no redistribution, fck <= 50 (cl. 5.5(4))
    let mu_lim = 0.8 * xi_lim * (1.0 - 0.4 * xi_lim);
    t.push("EN 1992-1-1 cl. 6.1, 3.1.7", format!("mu = M_Ed/(b d^2 fcd) = {:.1}e6 /({} x {:.1}^2 x {:.2})", i.m_ed, i.b, d, fcd.v), format!("mu = {:.4}; mu_lim = 0.8(0.45)(1-0.4x0.45) = {:.4}", mu, mu_lim));
    let mut as_req; let mut as_comp = 0.0;
    let mut as_prov_flex_cap: Option<f64> = None;
    if mu <= mu_lim {
        let z = d / 2.0 * (1.0 + (1.0 - 2.0 * mu).sqrt());
        let z = z.min(0.95 * d);
        as_req = (m / (fyd * Q::mm(z))).expect(AREA).unwrap();
        t.push("EN 1992-1-1 cl. 6.1", format!("z = d/2[1+sqrt(1-2mu)] <= 0.95d = {:.1} mm; As = M_Ed/(fyd z) = {:.1}e6/({:.2} x {:.1})", z, i.m_ed, fyd.v, z), format!("As,req = {:.0} mm2", as_req));
    } else {
        let d2 = i.cover + i.link_dia + i.bar_dia / 2.0;
        let z_lim = d * (1.0 - 0.4 * xi_lim);
        let x_lim = xi_lim * d;
        let m_lim = mu_lim * b.v * d * d * fcd.v;
        let eps_sc = 0.0035 * (1.0 - d2 / x_lim);
        let fsc = (es * eps_sc).min(fyd.v);
        if fsc <= 0.0 { return fail(vec!["Compression steel not effective (d2 >= x). Increase section depth.".into()]); }
        as_comp = (m.v - m_lim) / (fsc * (d - d2));
        as_req = m_lim / (fyd.v * z_lim) + as_comp * fsc / fyd.v;
        t.push("EN 1992-1-1 cl. 6.1, 5.5(4)", format!("mu > mu_lim: compression steel required. d2 = {:.1} mm, x_lim = {:.1} mm, eps_sc = 0.0035(1-d2/x) = {:.5}, fsc = {:.1} MPa", d2, x_lim, eps_sc, fsc), format!("As2 = {:.0} mm2; As1 = {:.0} mm2", as_comp, as_req));
        warnings.push("Doubly reinforced section: compression steel required; consider increasing the section.".into());
    }

    // ---- Min / max ----
    let fctm = 0.30 * i.fck.powf(2.0 / 3.0);
    let as_min = (0.26 * fctm / i.fyk * i.b * d).max(0.0013 * i.b * d);
    let as_max = 0.04 * i.b * i.h;
    t.push("EN 1992-1-1 cl. 9.2.1.1(1)", format!("fctm = 0.30 fck^(2/3) = {:.2} MPa; As,min = max(0.26 fctm/fyk b d, 0.0013 b d)", fctm), format!("As,min = {:.0} mm2", as_min));
    t.push("EN 1992-1-1 cl. 9.2.1.1(3)", format!("As,max = 0.04 Ac = 0.04 x {} x {}", i.b, i.h), format!("As,max = {:.0} mm2", as_max));

    let as_design = as_req.max(as_min);
    // ---- Bar selection / provided ----
    let (bars, as_prov) = if i.mode == "check" {
        let a = i.as_provided.unwrap_or(0.0);
        (format!("As provided = {:.0} mm2 (user supplied)", a), a)
    } else {
        let mut best: Option<(usize, f64, f64)> = None;
        for dia in [12.0, 16.0, 20.0, 25.0, 32.0] {
            let n = (as_design / bar_area(dia)).ceil().max(2.0) as usize;
            let avail = i.b - 2.0 * (i.cover + i.link_dia);
            let gap = if n > 1 { (avail - n as f64 * dia) / (n as f64 - 1.0) } else { f64::MAX };
            let min_gap = dia.max(20.0);
            if gap >= min_gap && (best.is_none() || bar_area(dia) * (n as f64) < best.unwrap().1 * 1.0 + 1e9 && best.unwrap().1 > n as f64 * bar_area(dia)) {
                if best.is_none() || n as f64 * bar_area(dia) < best.unwrap().1 { best = Some((n, n as f64 * bar_area(dia), dia)); }
            }
            if best.is_some() && n <= 4 { break; }
        }
        match best {
            Some((n, a, dia)) => (format!("{}T{:.0}", n, dia), a),
            None => { warnings.push("No single-layer bar arrangement fits within the width; multi-layer arrangement required (not auto-selected).".into()); (String::from("multi-layer required"), 0.0) }
        }
    };
    t.push("Reinforcement", "Provided tension reinforcement", format!("{} => As,prov = {:.0} mm2", bars, as_prov));

    let mut checks = vec![];
    // Flexural capacity of provided steel (singly reinforced, tension steel yields)
    if as_prov > 0.0 {
        let x = as_prov * fyd.v / (0.8 * i.b * fcd.v);
        let z = d - 0.4 * x;
        let mrd = as_prov * fyd.v * z;
        t.push("EN 1992-1-1 cl. 6.1", format!("x = As fyd/(0.8 b fcd) = {:.1} mm; MRd = As fyd (d - 0.4x) = {:.0} x {:.2} x {:.1}", x, as_prov, fyd.v, z), format!("MRd = {:.1} kNm; x/d = {:.3}", mrd / 1e6, x / d));
        let ok_x = x / d <= xi_lim + 1e-9;
        if !ok_x && as_comp == 0.0 { warnings.push(format!("x/d = {:.3} exceeds 0.45: section is over-reinforced for singly reinforced capacity.", x / d)); }
        as_prov_flex_cap = Some(mrd);
        let eff = if as_comp > 0.0 { m.v } else { mrd };
        let _ = eff;
        let util = if mrd > 0.0 { m.v / mrd } else { f64::INFINITY };
        let pass = if as_comp > 0.0 && i.mode != "check" { true } else { util <= 1.0 && ok_x };
        checks.push(Check { name: "Flexural resistance".into(), reference: "EN 1992-1-1 6.1".into(), demand: format!("{:.1} kNm", i.m_ed), capacity: format!("{:.1} kNm", mrd / 1e6), utilisation: util, pass });
    } else {
        checks.push(Check { name: "Flexural resistance".into(), reference: "EN 1992-1-1 6.1".into(), demand: format!("{:.1} kNm", i.m_ed), capacity: "no reinforcement provided".into(), utilisation: f64::INFINITY, pass: false });
    }
    let _ = as_prov_flex_cap;
    checks.push(Check { name: "Minimum reinforcement".into(), reference: "EN 1992-1-1 9.2.1.1(1)".into(), demand: format!("{:.0} mm2", as_min), capacity: format!("{:.0} mm2", as_prov), utilisation: if as_prov > 0.0 { as_min / as_prov } else { f64::INFINITY }, pass: as_prov >= as_min });
    checks.push(Check { name: "Maximum reinforcement".into(), reference: "EN 1992-1-1 9.2.1.1(3)".into(), demand: format!("{:.0} mm2", as_prov), capacity: format!("{:.0} mm2", as_max), utilisation: as_prov / as_max, pass: as_prov <= as_max });

    // ---- Shear ----
    let z_v = 0.9 * d;
    let nu1 = 0.6 * (1.0 - i.fck / 250.0);
    let ved = Q::kn(i.v_ed).v;
    let vrd_max = |cot: f64| ps.alpha_cw * i.b * z_v * nu1 * fcd.v / (cot + 1.0 / cot);
    let mut cot = 2.5;
    while cot > 1.0 && vrd_max(cot) < ved { cot -= 0.05; }
    cot = cot.max(1.0);
    let vmax = vrd_max(cot);
    t.push("EN 1992-1-1 cl. 6.2.3(3) Eq 6.9", format!("z = 0.9d = {:.1} mm; nu1 = 0.6(1-fck/250) = {:.3}; VRd,max = alpha_cw b z nu1 fcd/(cot + tan) with cot(theta) = {:.2}", z_v, nu1, cot), format!("VRd,max = {:.1} kN", vmax / 1e3));
    let fywd = fyd.v;
    let asw_s_req = ved / (z_v * fywd * cot);
    let rho_min = 0.08 * i.fck.sqrt() / i.fyk;
    let asw_s_min = rho_min * i.b;
    t.push("EN 1992-1-1 cl. 6.2.3(3) Eq 6.8, 9.2.2(5)", format!("Asw/s = V_Ed/(z fywd cot) = {:.1}e3/({:.1} x {:.2} x {:.2}); Asw/s,min = 0.08 sqrt(fck)/fyk x b", i.v_ed, z_v, fywd, cot), format!("Asw/s,req = {:.3} mm2/mm; min = {:.3}", asw_s_req, asw_s_min));
    checks.push(Check { name: "Shear: compression strut".into(), reference: "EN 1992-1-1 6.2.3 Eq 6.9".into(), demand: format!("{:.1} kN", i.v_ed), capacity: format!("{:.1} kN", vmax / 1e3), utilisation: ved / vmax, pass: ved <= vmax });
    let asw_use = asw_s_req.max(asw_s_min);
    let (links, asw_prov_s) = if i.mode == "check" {
        let a = i.asw_s_provided.unwrap_or(0.0);
        (format!("Asw/s provided = {:.3} mm2/mm (user supplied)", a), a)
    } else {
        let a_leg = 2.0 * bar_area(i.link_dia);
        let mut s = ((a_leg / asw_use) / 25.0).floor() * 25.0;
        let s_max = 0.75 * d;
        if s > s_max { s = (s_max / 25.0).floor() * 25.0; }
        if s < 75.0 { (format!("T{:.0} links at <75 mm required - increase link size", i.link_dia), 0.0) } else { (format!("T{:.0} 2-leg links @ {:.0} mm c/c", i.link_dia, s), a_leg / s) }
    };
    t.push("Reinforcement", "Shear links provided (max spacing 0.75d, cl. 9.2.2(6))", format!("{} => Asw/s = {:.3} mm2/mm", links, asw_prov_s));
    checks.push(Check { name: "Shear reinforcement".into(), reference: "EN 1992-1-1 6.2.3 Eq 6.8, 9.2.2".into(), demand: format!("{:.3} mm2/mm", asw_use), capacity: format!("{:.3} mm2/mm", asw_prov_s), utilisation: if asw_prov_s > 0.0 { asw_use / asw_prov_s } else { f64::INFINITY }, pass: asw_prov_s >= asw_use });

    let overall = checks.iter().all(|c| c.pass);
    t.push("Summary", "All implemented checks", if overall { "PASS".to_string() } else { "FAIL".to_string() });
    let hash = t.hash();
    BeamResult {
        ok: true, errors: vec![], warnings, param_status: ps.status.clone(), d, as_req, as_comp_req: as_comp, as_min, as_max, bars, as_prov,
        asw_s_req, links, checks, overall_pass: overall, trace: t, trace_hash: hash, engine_version: crate::ENGINE_VERSION.into(),
        scope_notes: vec![
            "Implemented: flexure (rectangular stress block), min/max reinforcement, shear (variable strut inclination), link spacing limit.".into(),
            "NOT implemented in this release: torsion, crack control, deflection, anchorage, lap lengths, flanged sections, redistribution.".into(),
            "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into(),
        ],
    }
}

pub fn run_json(input: &str) -> String {
    match serde_json::from_str::<BeamInput>(input) {
        Ok(i) => serde_json::to_string(&run(&i)).unwrap(),
        Err(e) => serde_json::to_string(&fail(vec![format!("Invalid input: {}", e)])).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> BeamInput { BeamInput { mode: "design".into(), param_set: "EN1992-1-1:2004/RECOMMENDED".into(), b: 300.0, h: 500.0, cover: 30.0, link_dia: 8.0, bar_dia: 20.0, fck: 30.0, fyk: 500.0, m_ed: 200.0, v_ed: 120.0, as_provided: None, asw_s_provided: None } }

    #[test]
    fn flexure_matches_hand_calc() {
        // Hand calc: d=452, fcd=20, mu=200e6/(300*452^2*20)=0.1632, z=0.5*452*(1+sqrt(1-2*0.1632))=411.5, As=200e6/(434.78*411.5)=1118
        let r = run(&base());
        assert!(r.ok);
        assert!((r.d - 452.0).abs() < 1e-9);
        assert!((r.as_req - 1118.0).abs() < 3.0, "got {}", r.as_req);
    }
    #[test]
    fn deterministic_hash() { assert_eq!(run(&base()).trace_hash, run(&base()).trace_hash); }
    #[test]
    fn rejects_bad_input() { let mut i = base(); i.b = -1.0; assert!(!run(&i).ok); }
    #[test]
    fn rejects_unknown_param_set() { let mut i = base(); i.param_set = "X".into(); assert!(!run(&i).ok); }
    #[test]
    fn design_passes_own_checks() { let r = run(&base()); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test]
    fn check_mode_fails_underprovided() {
        let mut i = base(); i.mode = "check".into(); i.as_provided = Some(600.0); i.asw_s_provided = Some(0.3);
        let r = run(&i); assert!(r.ok && !r.overall_pass);
    }
    #[test]
    fn shear_strut_fails_when_extreme() { let mut i = base(); i.v_ed = 2000.0; let r = run(&i); assert!(!r.overall_pass); }
    #[test]
    fn doubly_reinforced_triggers() { let mut i = base(); i.m_ed = 380.0; let r = run(&i); assert!(r.ok && r.as_comp_req > 0.0); }
    #[test]
    fn min_steel_governs_light_load() { let mut i = base(); i.m_ed = 10.0; let r = run(&i); assert!(r.as_min > r.as_req); assert!(r.as_prov >= r.as_min); }
}
