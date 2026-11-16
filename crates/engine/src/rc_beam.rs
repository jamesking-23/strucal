//! RC rectangular beam: flexure, shear, min/max reinforcement, bar spacing (EN 1992-1-1:2004).
//! Not implemented in this release: torsion, crack control, deflection, anchorage, lap lengths.
use crate::params;
use crate::trace::Trace;
use crate::units::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BeamInput {
    pub mode: String,            // "design" | "check"
    pub param_set: String,
    pub b: f64, pub h: f64,      // mm
    pub cover: f64,              // nominal cover to links, mm
    pub link_dia: f64,           // mm
    pub bar_dia: f64,            // main tension bar dia, mm
    pub fck: f64, pub fyk: f64,  // MPa
    pub m_ed: f64,               // kNm (sagging)
    pub v_ed: f64,               // kN
    #[serde(default)] pub n_bars: Option<f64>,       // check mode: number of tension bars (single layer)
    #[serde(default)] pub link_spacing: Option<f64>, // check mode: link spacing, mm
}

#[derive(Serialize, Debug, Clone)]
pub struct Check { pub name: String, pub reference: String, pub demand: String, pub capacity: String, pub utilisation: f64, pub pass: bool }
#[derive(Serialize, Debug, Clone)]
pub struct Obs { pub id: String, pub level: String, pub text: String }

#[derive(Serialize, Debug, Default)]
pub struct BeamResult {
    pub ok: bool, pub errors: Vec<String>, pub warnings: Vec<String>, pub param_status: String,
    pub d: f64, pub z: f64, pub mu: f64, pub mu_lim: f64,
    pub as_req: f64, pub as_comp_req: f64, pub as_min: f64, pub as_max: f64,
    pub bar_count: u32, pub bar_dia: f64, pub bars: String, pub as_prov: f64, pub top_steel: String,
    pub asw_s_req: f64, pub link_spacing: f64, pub links: String,
    pub checks: Vec<Check>, pub max_util: f64, pub overall_pass: bool, pub observations: Vec<Obs>,
    pub trace: Trace, pub trace_hash: String, pub engine_version: String, pub scope_notes: Vec<String>,
}

fn fail(errors: Vec<String>) -> BeamResult { BeamResult { errors, engine_version: crate::ENGINE_VERSION.into(), ..Default::default() } }

fn chk(name: &str, reference: &str, demand: String, capacity: String, util: f64, pass: bool) -> Check {
    let u = if util.is_finite() { util.min(9.99) } else { 9.99 };
    Check { name: name.into(), reference: reference.into(), demand, capacity, utilisation: u, pass }
}

pub fn run(i: &BeamInput) -> BeamResult {
    let mut errs = vec![];
    for (n, v) in [("b", i.b), ("h", i.h), ("cover", i.cover), ("link_dia", i.link_dia), ("bar_dia", i.bar_dia), ("fck", i.fck), ("fyk", i.fyk)] {
        if !(v.is_finite() && v > 0.0) { errs.push(format!("{} must be a positive number", n)); }
    }
    if !(i.m_ed.is_finite() && i.m_ed >= 0.0) { errs.push("M_Ed must be >= 0 (sagging)".into()); }
    if !(i.v_ed.is_finite() && i.v_ed >= 0.0) { errs.push("V_Ed must be >= 0".into()); }
    if i.fck > 50.0 { errs.push("fck > 50 MPa is outside the scope of this release (C50/60 max)".into()); }
    if i.fck < 12.0 { errs.push("fck < 12 MPa is outside EN 1992-1-1 Table 3.1".into()); }
    let check_mode = i.mode == "check";
    if check_mode {
        match i.n_bars { Some(n) if n >= 2.0 && n <= 20.0 && n.fract() == 0.0 => {}, _ => errs.push("Number of tension bars must be a whole number from 2 to 20".into()) }
        match i.link_spacing { Some(s) if s.is_finite() && s > 0.0 => {}, _ => errs.push("Link spacing must be a positive number".into()) }
    }
    let ps = match params::get(&i.param_set) { Some(p) => p, None => { errs.push("Unknown or unavailable parameter set - select one explicitly".into()); return fail(errs); } };
    if !errs.is_empty() { return fail(errs); }

    let mut t = Trace::new();
    let (s1, s2, s3, s4) = ("1. Design data", "2. Effective depth and design strengths", "3. Flexural design", "4. Shear design");
    let es = 200_000.0;
    let mut warnings: Vec<String> = vec![];

    // ---------------- 1. Design data ----------------
    t.step(s1, "Section geometry", "", "", &format!("b = {} mm;  h = {} mm;  nominal cover c = {} mm;  link φ_{{link}} = {} mm;  tension bar φ = {} mm", i.b, i.h, i.cover, i.link_dia, i.bar_dia), "", "Rectangular section, sagging moment about the major axis.");
    t.step(s1, "Materials", "EN 1992-1-1 Table 3.1", "", &format!("Concrete f_{{ck}} = {} MPa;  reinforcement f_{{yk}} = {} MPa;  E_{{s}} = 200 GPa", i.fck, i.fyk), "", "");
    t.step(s1, "Design actions (ULS)", "", "", &format!("M_{{Ed}} = {} kNm;  V_{{Ed}} = {} kN", i.m_ed, i.v_ed), "", "Design values after load combination (EN 1990), supplied by the user.");
    t.step(s1, "Partial factors and parameters", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6, 6.2.3", "", &format!("γ_{{c}} = {:.2};  γ_{{s}} = {:.2};  α_{{cc}} = {:.2};  α_{{cw}} = {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc, ps.alpha_cw), "", &format!("Parameter set: {} [{}]", ps.label, ps.status));

    // ---------------- 2. Effective depth, strengths ----------------
    let d = i.h - i.cover - i.link_dia - i.bar_dia / 2.0;
    if d <= 0.0 { return fail(vec!["Effective depth is not positive - check h, cover and bar sizes".into()]); }
    t.step(s2, "Effective depth", "Geometry", "d = h − c − φ_{link} − φ/2", &format!("= {} − {} − {} − {}/2", i.h, i.cover, i.link_dia, i.bar_dia), &format!("d = {:.1} mm", d), "");
    let fcd = ps.alpha_cc * i.fck / ps.gamma_c;
    let fyd = i.fyk / ps.gamma_s;
    t.step(s2, "Design compressive strength of concrete", "EN 1992-1-1 cl. 3.1.6(1)P, Eq 3.15", "f_{cd} = α_{cc}·f_{ck} / γ_{c}", &format!("= {:.2} × {} / {:.2}", ps.alpha_cc, i.fck, ps.gamma_c), &format!("f_{{cd}} = {:.2} MPa", fcd), "");
    t.step(s2, "Design yield strength of reinforcement", "EN 1992-1-1 cl. 2.4.2.4, 3.2.7", "f_{yd} = f_{yk} / γ_{s}", &format!("= {} / {:.2}", i.fyk, ps.gamma_s), &format!("f_{{yd}} = {:.2} MPa", fyd), "");
    let fctm = 0.30 * i.fck.powf(2.0 / 3.0);
    t.step(s2, "Mean axial tensile strength of concrete", "EN 1992-1-1 Table 3.1 (f_{ck} ≤ C50/60)", "f_{ctm} = 0.30·f_{ck}^{2/3}", &format!("= 0.30 × {}^{{2/3}}", i.fck), &format!("f_{{ctm}} = {:.2} MPa", fctm), "");

    // ---------------- 3. Flexure ----------------
    let b_q = Q::mm(i.b); let d_q = Q::mm(d); let fcd_q = Q::mpa(fcd);
    let m_q = Q::knm(i.m_ed);
    let mu = (m_q / (b_q * d_q * d_q * fcd_q)).expect(SCALAR).unwrap();
    t.step(s3, "Normalised moment", "EN 1992-1-1 cl. 3.1.7, 6.1", "μ = M_{Ed} / (b·d^{2}·f_{cd})", &format!("= {:.1}×10^{{6}} / ({} × {:.1}^{{2}} × {:.2})", i.m_ed, i.b, d, fcd), &format!("μ = {:.4}", mu), "Rectangular stress block: λ = 0.8, η = 1.0.");
    let xi_lim = 0.45;
    let mu_lim = 0.8 * xi_lim * (1.0 - 0.4 * xi_lim);
    t.step(s3, "Limiting moment coefficient", "EN 1992-1-1 cl. 5.5(4) (no redistribution, f_{ck} ≤ 50)", "μ_{lim} = 0.8·(x/d)_{lim}·[1 − 0.4·(x/d)_{lim}],  (x/d)_{lim} = 0.45", &format!("= 0.8 × 0.45 × (1 − 0.4 × 0.45)"), &format!("μ_{{lim}} = {:.4}", mu_lim), &if mu <= mu_lim { format!("μ = {:.4} ≤ μ_lim: compression reinforcement not required.", mu) } else { format!("μ = {:.4} > μ_lim: compression reinforcement required.", mu) });

    let m_nmm = i.m_ed * 1e6;
    let mut as_comp = 0.0;
    let as_req; let mut z_used = 0.0;
    if mu <= mu_lim {
        let z = (d / 2.0 * (1.0 + (1.0 - 2.0 * mu).sqrt())).min(0.95 * d);
        z_used = z;
        t.step(s3, "Lever arm", "EN 1992-1-1 cl. 6.1", "z = (d/2)·[1 + √(1 − 2μ)]  ≤ 0.95d", &format!("= ({:.1}/2) × [1 + √(1 − 2 × {:.4})]", d, mu), &format!("z = {:.1} mm  (0.95d = {:.1} mm)", z, 0.95 * d), "");
        as_req = m_nmm / (fyd * z);
        t.step(s3, "Required tension reinforcement", "EN 1992-1-1 cl. 6.1", "A_{s,req} = M_{Ed} / (f_{yd}·z)", &format!("= {:.1}×10^{{6}} / ({:.2} × {:.1})", i.m_ed, fyd, z), &format!("A_{{s,req}} = {:.0} mm^{{2}}", as_req), "");
        let x = (d - z) / 0.4;
        t.step(s3, "Neutral axis depth (ductility check)", "EN 1992-1-1 cl. 5.5(4)", "x = (d − z) / 0.4", &format!("= ({:.1} − {:.1}) / 0.4", d, z), &format!("x = {:.1} mm;  x/d = {:.3} ≤ 0.45", x, x / d), "");
    } else {
        let d2 = i.cover + i.link_dia + i.bar_dia / 2.0;
        let x_lim = xi_lim * d;
        let m_lim = mu_lim * i.b * d * d * fcd;
        let z_lim = d * (1.0 - 0.4 * xi_lim);
        z_used = z_lim;
        t.step(s3, "Moment resisted without compression steel", "EN 1992-1-1 cl. 6.1", "M_{lim} = μ_{lim}·b·d^{2}·f_{cd}", &format!("= {:.4} × {} × {:.1}^{{2}} × {:.2}", mu_lim, i.b, d, fcd), &format!("M_{{lim}} = {:.1} kNm", m_lim / 1e6), &format!("Excess moment ΔM = {:.1} − {:.1} = {:.1} kNm resisted by compression steel.", i.m_ed, m_lim / 1e6, (m_nmm - m_lim) / 1e6));
        let eps = 0.0035 * (1.0 - d2 / x_lim);
        let fsc = (es * eps).min(fyd);
        if fsc <= 0.0 { return fail(vec!["Compression steel not effective (d2 >= x). Increase section depth.".into()]); }
        t.step(s3, "Stress in compression steel", "EN 1992-1-1 cl. 3.1.7, 3.2.7", "d_{2} = c + φ_{link} + φ/2;   ε_{sc} = 0.0035·(1 − d_{2}/x_{lim});   f_{sc} = min(E_{s}·ε_{sc} ; f_{yd})", &format!("d_{{2}} = {:.1} mm;  x_{{lim}} = 0.45 × {:.1} = {:.1} mm;  ε_{{sc}} = 0.0035 × (1 − {:.1}/{:.1}) = {:.5};  f_{{sc}} = min({:.1} ; {:.2})", d2, d, x_lim, d2, x_lim, eps, es * eps, fyd), &format!("f_{{sc}} = {:.1} MPa", fsc), "Compression bars assumed at the same cover and size as the tension bars.");
        as_comp = (m_nmm - m_lim) / (fsc * (d - d2));
        t.step(s3, "Required compression reinforcement", "EN 1992-1-1 cl. 6.1", "A_{s2} = ΔM / (f_{sc}·(d − d_{2}))", &format!("= {:.2}×10^{{6}} / ({:.1} × ({:.1} − {:.1}))", (m_nmm - m_lim) / 1e6, fsc, d, d2), &format!("A_{{s2}} = {:.0} mm^{{2}}", as_comp), "");
        as_req = m_lim / (fyd * z_lim) + as_comp * fsc / fyd;
        t.step(s3, "Required tension reinforcement", "EN 1992-1-1 cl. 6.1", "A_{s1} = M_{lim}/(f_{yd}·z_{lim}) + A_{s2}·f_{sc}/f_{yd},   z_{lim} = d·(1 − 0.4×0.45)", &format!("= {:.1}×10^{{6}}/({:.2} × {:.1}) + {:.0} × {:.1}/{:.2}", m_lim / 1e6, fyd, z_lim, as_comp, fsc, fyd), &format!("A_{{s1}} = {:.0} mm^{{2}}", as_req), "");
        warnings.push("Doubly reinforced section: compression steel is required. Consider increasing the section depth.".into());
    }

    let as_min_a = 0.26 * fctm / i.fyk * i.b * d;
    let as_min_b = 0.0013 * i.b * d;
    let as_min = as_min_a.max(as_min_b);
    t.step(s3, "Minimum tension reinforcement", "EN 1992-1-1 cl. 9.2.1.1(1), Eq 9.1N", "A_{s,min} = max[ 0.26·(f_{ctm}/f_{yk})·b·d ;  0.0013·b·d ]", &format!("= max[ 0.26 × ({:.2}/{}) × {} × {:.1} ;  0.0013 × {} × {:.1} ] = max[ {:.0} ; {:.0} ]", fctm, i.fyk, i.b, d, i.b, d, as_min_a, as_min_b), &format!("A_{{s,min}} = {:.0} mm^{{2}}", as_min), "");
    let as_max = 0.04 * i.b * i.h;
    t.step(s3, "Maximum reinforcement", "EN 1992-1-1 cl. 9.2.1.1(3)", "A_{s,max} = 0.04·A_{c} = 0.04·b·h", &format!("= 0.04 × {} × {}", i.b, i.h), &format!("A_{{s,max}} = {:.0} mm^{{2}}", as_max), "");
    let as_design = as_req.max(as_min);
    t.step(s3, "Design area of tension reinforcement", "", "A_{s,design} = max(A_{s,req} ; A_{s,min})", &format!("= max({:.0} ; {:.0})", as_req, as_min), &format!("A_{{s,design}} = {:.0} mm^{{2}}", as_design), if as_min > as_req { "Minimum reinforcement governs." } else { "Strength governs." });

    let area1 = PI * i.bar_dia * i.bar_dia / 4.0;
    let n_bars: u32 = if check_mode { i.n_bars.unwrap() as u32 } else {
        let n = ((as_design / area1).ceil() as u32).max(2);
        t.step(s3, "Number of tension bars", "", "A_{bar} = π·φ^{2}/4;   n = ⌈A_{s,design} / A_{bar}⌉  (minimum 2)", &format!("A_{{bar}} = π × {}^{{2}}/4 = {:.1} mm^{{2}};  n = ⌈{:.0} / {:.1}⌉", i.bar_dia, area1, as_design, area1), &format!("n = {}", n), "");
        n
    };
    let as_prov = n_bars as f64 * area1;
    t.step(s3, if check_mode { "Provided tension reinforcement (from drawing)" } else { "Provided tension reinforcement" }, "", "A_{s,prov} = n·π·φ^{2}/4", &format!("= {} × π × {}^{{2}}/4", n_bars, i.bar_dia), &format!("A_{{s,prov}} = {:.0} mm^{{2}}  ({}T{})", as_prov, n_bars, i.bar_dia), "");
    let bars = format!("{}T{}", n_bars, i.bar_dia);

    let avail = i.b - 2.0 * (i.cover + i.link_dia);
    let gap = (avail - n_bars as f64 * i.bar_dia) / (n_bars as f64 - 1.0);
    let min_gap = i.bar_dia.max(25.0);
    t.step(s3, "Bar spacing (single layer)", "EN 1992-1-1 cl. 8.2(2)", "s_{clear} = [b − 2(c + φ_{link}) − n·φ] / (n − 1)  ≥  max(φ ; d_{g} + 5 ; 20),  d_{g} = 20 mm assumed", &format!("= [{} − 2 × ({} + {}) − {} × {}] / ({} − 1) ;  limit = max({} ; 25)", i.b, i.cover, i.link_dia, n_bars, i.bar_dia, n_bars, i.bar_dia), &format!("s_{{clear}} = {:.1} mm  {} {:.0} mm", gap, if gap >= min_gap { "≥" } else { "<" }, min_gap), if gap >= min_gap { "" } else { "Bars do not fit in one layer: use smaller or fewer bars, or two layers (not auto-designed)." });

    let mut checks: Vec<Check> = vec![];
    if as_comp > 0.0 && !check_mode {
        checks.push(chk("Flexural resistance", "EN 1992-1-1 6.1", format!("{:.0} mm² req.", as_req), format!("{:.0} mm² prov.", as_prov), as_req / as_prov, as_prov >= as_req));
    } else {
        let x = as_prov * fyd / (0.8 * i.b * fcd);
        let z = d - 0.4 * x;
        let mrd = as_prov * fyd * z;
        t.step(s3, "Moment resistance of provided steel", "EN 1992-1-1 cl. 3.1.7, 6.1", "x = A_{s,prov}·f_{yd} / (0.8·b·f_{cd});   z = d − 0.4x;   M_{Rd} = A_{s,prov}·f_{yd}·z", &format!("x = {:.0} × {:.2} / (0.8 × {} × {:.2}) = {:.1} mm;  z = {:.1} − 0.4 × {:.1} = {:.1} mm;  M_{{Rd}} = {:.0} × {:.2} × {:.1}", as_prov, fyd, i.b, fcd, x, d, x, z, as_prov, fyd, z), &format!("M_{{Rd}} = {:.1} kNm;  x/d = {:.3}", mrd / 1e6, x / d), if x / d <= xi_lim + 1e-9 { "Tension steel yields and x/d ≤ 0.45: ductile." } else { "x/d > 0.45: section is over-reinforced (compression steel or a larger section needed)." });
        let ok_x = x / d <= xi_lim + 1e-9;
        checks.push(chk("Flexural resistance", "EN 1992-1-1 6.1", format!("{:.1} kNm", i.m_ed), format!("{:.1} kNm", mrd / 1e6), m_nmm / mrd, m_nmm <= mrd && ok_x));
        if z_used == 0.0 { z_used = z; }
    }
    checks.push(chk("Minimum reinforcement", "EN 1992-1-1 9.2.1.1(1)", format!("{:.0} mm²", as_min), format!("{:.0} mm²", as_prov), as_min / as_prov, as_prov >= as_min));
    checks.push(chk("Maximum reinforcement", "EN 1992-1-1 9.2.1.1(3)", format!("{:.0} mm²", as_prov), format!("{:.0} mm²", as_max), as_prov / as_max, as_prov <= as_max));
    checks.push(chk("Bar spacing", "EN 1992-1-1 8.2(2)", format!("{:.0} mm min.", min_gap), format!("{:.0} mm clear", gap), min_gap / gap, gap >= min_gap));

    // ---------------- 4. Shear ----------------
    let ved = i.v_ed * 1e3;
    let z_v = 0.9 * d;
    let nu1 = 0.6 * (1.0 - i.fck / 250.0);
    t.step(s4, "Shear lever arm and strength reduction factor", "EN 1992-1-1 cl. 6.2.3(1), 6.2.2(6)", "z = 0.9d;   ν_{1} = 0.6·(1 − f_{ck}/250)", &format!("z = 0.9 × {:.1};  ν_{{1}} = 0.6 × (1 − {}/250)", d, i.fck), &format!("z = {:.1} mm;  ν_{{1}} = {:.3}", z_v, nu1), "");
    let vrd_max = |cot: f64| ps.alpha_cw * i.b * z_v * nu1 * fcd / (cot + 1.0 / cot);
    let mut cot = 2.5;
    t.step(s4, "Strut capacity, trial cot θ = 2.5 (θ = 21.8°)", "EN 1992-1-1 cl. 6.2.3(3), Eq 6.9", "V_{Rd,max} = α_{cw}·b·z·ν_{1}·f_{cd} / (cot θ + tan θ)", &format!("= {:.2} × {} × {:.1} × {:.3} × {:.2} / ({:.2} + {:.3})", ps.alpha_cw, i.b, z_v, nu1, fcd, cot, 1.0 / cot), &format!("V_{{Rd,max}} = {:.1} kN  {} V_{{Ed}} = {:.1} kN", vrd_max(cot) / 1e3, if vrd_max(cot) >= ved { "≥" } else { "<" }, i.v_ed), "");
    if vrd_max(cot) < ved {
        while cot > 1.0 && vrd_max(cot) < ved { cot -= 0.05; }
        cot = cot.max(1.0);
        t.step(s4, "Strut inclination increased until V_Rd,max ≥ V_Ed", "EN 1992-1-1 cl. 6.2.3(2): 1 ≤ cot θ ≤ 2.5", "V_{Rd,max}(cot θ) ≥ V_{Ed}", &format!("cot θ = {:.2}  (θ = {:.1}°)", cot, (1.0 / cot).atan().to_degrees()), &format!("V_{{Rd,max}} = {:.1} kN", vrd_max(cot) / 1e3), if vrd_max(cot) >= ved { "" } else { "Even at cot θ = 1.0 the strut capacity is insufficient: increase the section." });
    }
    let vmax = vrd_max(cot);
    let asw_s_req = ved / (z_v * fyd * cot);
    t.step(s4, "Required shear reinforcement", "EN 1992-1-1 cl. 6.2.3(3), Eq 6.8", "A_{sw}/s = V_{Ed} / (z·f_{ywd}·cot θ),   f_{ywd} = f_{yd}", &format!("= {:.1}×10^{{3}} / ({:.1} × {:.2} × {:.2})", i.v_ed, z_v, fyd, cot), &format!("A_{{sw}}/s = {:.3} mm^{{2}}/mm", asw_s_req), "");
    let rho_min = 0.08 * i.fck.sqrt() / i.fyk;
    let asw_s_min = rho_min * i.b;
    t.step(s4, "Minimum shear reinforcement", "EN 1992-1-1 cl. 9.2.2(5), Eq 9.4, 9.5N", "ρ_{w,min} = 0.08·√f_{ck} / f_{yk};   (A_{sw}/s)_{min} = ρ_{w,min}·b·sin α  (α = 90°)", &format!("ρ = 0.08 × √{} / {} = {:.5};  (A_{{sw}}/s)_{{min}} = {:.5} × {}", i.fck, i.fyk, rho_min, rho_min, i.b), &format!("(A_{{sw}}/s)_{{min}} = {:.3} mm^{{2}}/mm", asw_s_min), "");
    let asw_use = asw_s_req.max(asw_s_min);
    t.step(s4, "Design shear reinforcement", "", "(A_{sw}/s)_{design} = max[ required ; minimum ]", &format!("= max({:.3} ; {:.3})", asw_s_req, asw_s_min), &format!("(A_{{sw}}/s)_{{design}} = {:.3} mm^{{2}}/mm", asw_use), "");
    let a_leg = 2.0 * PI * i.link_dia * i.link_dia / 4.0;
    let s_max = 0.75 * d;
    let (link_s, asw_prov_s, links): (f64, f64, String);
    if check_mode {
        let s = i.link_spacing.unwrap();
        t.step(s4, "Provided links (from drawing)", "", "A_{sw} = 2 legs × π·φ_{link}^{2}/4;   (A_{sw}/s)_{prov} = A_{sw} / s", &format!("A_{{sw}} = 2 × π × {}^{{2}}/4 = {:.1} mm^{{2}};  = {:.1} / {}", i.link_dia, a_leg, a_leg, s), &format!("(A_{{sw}}/s)_{{prov}} = {:.3} mm^{{2}}/mm  (T{} @ {})", a_leg / s, i.link_dia, s), "");
        link_s = s; asw_prov_s = a_leg / s; links = format!("T{} 2-leg @ {} c/c", i.link_dia, s);
    } else {
        let s_calc = a_leg / asw_use;
        let s_lim = ((s_calc.min(s_max)) / 25.0).floor() * 25.0;
        t.step(s4, "Link size and spacing", "EN 1992-1-1 cl. 9.2.2(6)", "s ≤ A_{sw} / (A_{sw}/s)_{design};   s_{max} = 0.75d", &format!("A_{{sw}} = 2 × π × {}^{{2}}/4 = {:.1} mm^{{2}};  s ≤ {:.1}/{:.3} = {:.0} mm;  s_{{max}} = 0.75 × {:.1} = {:.0} mm", i.link_dia, a_leg, a_leg, asw_use, s_calc, d, s_max), &if s_lim >= 75.0 { format!("Use T{} 2-leg links @ {:.0} mm c/c", i.link_dia, s_lim) } else { "Spacing < 75 mm: increase link diameter".to_string() }, "Spacing rounded down to a multiple of 25 mm.");
        if s_lim >= 75.0 { link_s = s_lim; asw_prov_s = a_leg / s_lim; links = format!("T{} 2-leg @ {:.0} c/c", i.link_dia, s_lim); }
        else { link_s = 0.0; asw_prov_s = 0.0; links = "increase link size".into(); }
    }
    checks.push(chk("Shear: compression strut", "EN 1992-1-1 6.2.3 Eq 6.9", format!("{:.1} kN", i.v_ed), format!("{:.1} kN", vmax / 1e3), ved / vmax, ved <= vmax));
    checks.push(chk("Shear reinforcement", "EN 1992-1-1 6.2.3 Eq 6.8, 9.2.2", format!("{:.3} mm²/mm", asw_use), format!("{:.3} mm²/mm", asw_prov_s), if asw_prov_s > 0.0 { asw_use / asw_prov_s } else { 9.99 }, asw_prov_s >= asw_use && asw_prov_s > 0.0));
    checks.push(chk("Link spacing limit", "EN 1992-1-1 9.2.2(6)", format!("{:.0} mm", link_s), format!("{:.0} mm max", s_max), link_s / s_max, link_s > 0.0 && link_s <= s_max + 1e-9));

    let overall = checks.iter().all(|c| c.pass);
    let max_util = checks.iter().map(|c| c.utilisation).fold(0.0, f64::max);
    t.step("5. Conclusion", "Overall result of implemented checks", "", "", &format!("{} checks evaluated; maximum utilisation {:.2}", checks.len(), max_util), if overall { "PASS" } else { "FAIL" }, "Applies only to the checks listed, within the stated assumptions.");

    // observations
    let mut obs = vec![];
    if ps.status.contains("NOT VERIFIED") || ps.status.contains("NOT A UGANDAN") {
        obs.push(("AMBER", format!("Parameter set: {}. Confirm the governing National Annex before issue for construction.", ps.status)));
    }
    for c in &checks { if !c.pass { obs.push(("RED", format!("{} fails ({} vs {}).", c.name, c.demand, c.capacity))); }
        else if c.utilisation > 0.85 { obs.push(("AMBER", format!("{} utilisation {:.2} exceeds the 0.85 review threshold.", c.name, c.utilisation))); } }
    if as_comp > 0.0 { obs.push(("INFO", "Compression reinforcement is required in this section.".to_string())); }
    if as_min > as_req { obs.push(("INFO", "Minimum reinforcement governs the tension steel.".to_string())); }
    let observations = obs.into_iter().enumerate().map(|(k, (l, tx))| Obs { id: format!("OBS-{:02}", k + 1), level: l.into(), text: tx }).collect();

    let hash = t.hash();
    BeamResult {
        ok: true, errors: vec![], warnings, param_status: ps.status.clone(), d, z: z_used, mu, mu_lim,
        as_req, as_comp_req: as_comp, as_min, as_max, bar_count: n_bars, bar_dia: i.bar_dia, bars, as_prov,
        top_steel: format!("2T{} nominal", if i.link_dia <= 10.0 { 12 } else { 16 }),
        asw_s_req, link_spacing: link_s, links, checks, max_util, overall_pass: overall, observations,
        trace: t, trace_hash: hash, engine_version: crate::ENGINE_VERSION.into(),
        scope_notes: vec![
            "Implemented: flexure (rectangular stress block), min/max reinforcement, single-layer bar spacing, shear (variable strut inclination), link spacing limit.".into(),
            "NOT implemented in this release: torsion, crack control, deflection, anchorage, lap lengths, flanged sections, moment redistribution, hogging moments.".into(),
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
    fn base() -> BeamInput { BeamInput { mode: "design".into(), param_set: "EN1992-1-1:2004/RECOMMENDED".into(), b: 300.0, h: 500.0, cover: 30.0, link_dia: 8.0, bar_dia: 20.0, fck: 30.0, fyk: 500.0, m_ed: 200.0, v_ed: 120.0, n_bars: None, link_spacing: None } }

    #[test]
    fn flexure_matches_hand_calc() {
        // d=452, fcd=20, mu=0.1632, z=411.5, As=200e6/(434.78*411.5)=1118
        let r = run(&base());
        assert!(r.ok); assert!((r.d - 452.0).abs() < 1e-9);
        assert!((r.as_req - 1118.0).abs() < 3.0, "got {}", r.as_req);
        assert!((r.mu - 0.1632).abs() < 5e-4);
    }
    #[test] fn deterministic_hash() { assert_eq!(run(&base()).trace_hash, run(&base()).trace_hash); }
    #[test] fn rejects_bad_input() { let mut i = base(); i.b = -1.0; assert!(!run(&i).ok); }
    #[test] fn rejects_unknown_param_set() { let mut i = base(); i.param_set = "X".into(); assert!(!run(&i).ok); }
    #[test] fn design_passes_own_checks() { let r = run(&base()); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test] fn check_mode_requires_provided_steel() { let mut i = base(); i.mode = "check".into(); assert!(!run(&i).ok); }
    #[test]
    fn check_mode_fails_underprovided() {
        let mut i = base(); i.mode = "check".into(); i.n_bars = Some(2.0); i.link_spacing = Some(300.0);
        let r = run(&i); assert!(r.ok && !r.overall_pass);
    }
    #[test]
    fn check_mode_passes_adequate() {
        let mut i = base(); i.mode = "check".into(); i.n_bars = Some(4.0); i.link_spacing = Some(200.0);
        let r = run(&i); assert!(r.ok && r.overall_pass, "{:?}", r.checks);
    }
    #[test] fn shear_strut_fails_when_extreme() { let mut i = base(); i.v_ed = 2000.0; assert!(!run(&i).overall_pass); }
    #[test] fn doubly_reinforced_triggers() { let mut i = base(); i.m_ed = 380.0; let r = run(&i); assert!(r.ok && r.as_comp_req > 0.0); }
    #[test] fn min_steel_governs_light_load() { let mut i = base(); i.m_ed = 10.0; let r = run(&i); assert!(r.as_min > r.as_req); assert!(r.as_prov >= r.as_min); }
    #[test] fn steps_have_working() { let r = run(&base()); assert!(r.trace.steps.len() > 25); assert!(r.trace.steps.iter().any(|s| !s.formula.is_empty() && !s.substitution.is_empty())); }
    #[test] fn json_has_no_nulls_for_utilisation() { let s = run_json(&serde_json::to_string(&base()).unwrap()); assert!(!s.contains("\"utilisation\":null")); }
}
