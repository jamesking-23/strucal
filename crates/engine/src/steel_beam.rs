//! Simply supported hot-rolled I/H steel beam under UDL (EN 1993-1-1:2005): classification, bending, shear, LTB, deflection.
//! Section properties are computed from the entered dimensions (including root radius) - no section table is used.
use crate::module::*;
use crate::params;
use crate::rc_beam::Obs;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

const E: f64 = 210_000.0; const G: f64 = 81_000.0;
const FA: f64 = 0.214602; // 1 - pi/4  (fillet area / r^2)
const FZ: f64 = 0.2234;   // fillet centroid offset / r
const FI: f64 = 0.007544; // fillet own second moment / r^4

pub struct Props { pub a: f64, pub iy: f64, pub iz: f64, pub wel: f64, pub wpl: f64, pub it: f64, pub iw: f64, pub av: f64 }
pub fn props(h: f64, b: f64, tw: f64, tf: f64, r: f64) -> Props {
    let hw = h - 2.0 * tf;
    let a = 2.0 * b * tf + hw * tw + (4.0 - PI) * r * r;
    let zf = hw / 2.0 - FZ * r;
    let iy = (b * h.powi(3) - (b - tw) * hw.powi(3)) / 12.0 + 4.0 * (FI * r.powi(4) + FA * r * r * zf * zf);
    let yf = tw / 2.0 + FZ * r;
    let iz = (2.0 * tf * b.powi(3) + hw * tw.powi(3)) / 12.0 + 4.0 * (FI * r.powi(4) + FA * r * r * yf * yf);
    let wpl = b * tf * (h - tf) + tw * (hw / 2.0).powi(2) + 4.0 * FA * r * r * zf;
    let it = (2.0 * b * tf.powi(3) + (h - tf) * tw.powi(3)) / 3.0;
    let iw = iz * (h - tf).powi(2) / 4.0;
    let av = a - 2.0 * b * tf + (tw + 2.0 * r) * tf;
    Props { a, iy, iz, wel: iy / (h / 2.0), wpl, it, iw, av }
}

pub fn def() -> ModDef {
    ModDef { id: "steelbeam", name: "Steel beam", title: "Steel I-beam verification (hot-rolled, UDL)", subtitle: "EN 1993-1-1:2005 · Simply supported, uniformly distributed load, lateral-torsional buckling and deflection", available: true, has_check_mode: false, fields: vec![
        fld("span", "Span L", "mm", "Geometry", 6000.0), fld("lcr", "Unrestrained length for LTB L cr", "mm", "Geometry", 2000.0), fld("load_level", "Load level (1 = top flange, 2 = shear centre / bottom)", "-", "Geometry", 1.0),
        fld("h", "Section depth h", "mm", "Section (IPE 300 shown)", 300.0), fld("b", "Flange width b", "mm", "Section (IPE 300 shown)", 150.0), fld("tw", "Web thickness t w", "mm", "Section (IPE 300 shown)", 7.1), fld("tf", "Flange thickness t f", "mm", "Section (IPE 300 shown)", 10.7), fld("r", "Root radius r", "mm", "Section (IPE 300 shown)", 15.0),
        fld("fy", "Yield strength f y", "MPa", "Materials", 275.0),
        fld("gk", "Permanent load g k (excl. beam self-weight)", "kN/m", "Actions", 10.0), fld("qk", "Variable load q k", "kN/m", "Actions", 12.0), fld("defl_ratio", "Deflection limit, span / ratio (variable load)", "-", "Serviceability", 360.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, false) { Ok(x) => x, Err(e) => return fail(e) };
    let (span, lcr, level, h, b, tw, tf, r, fy, gk, qk, dr) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11]);
    let mut errs = vec![];
    if 2.0 * tf >= h { errs.push("2·tf must be less than h".to_string()); }
    if tw >= b { errs.push("tw must be less than b".to_string()); }
    if r > (b - tw) / 2.0 || r > (h - 2.0 * tf) / 2.0 { errs.push("Root radius is too large for the section".to_string()); }
    if !(level == 1.0 || level == 2.0) { errs.push("Load level must be 1 (top flange) or 2 (shear centre / bottom)".to_string()); }
    if fy < 235.0 || fy > 460.0 { errs.push("fy must be between 235 and 460 MPa (EN 1993-1-1 Table 3.1, S235-S460)".to_string()); }
    if lcr > span * 1.0001 { errs.push("Unrestrained length cannot exceed the span in this release".to_string()); }
    if !errs.is_empty() { return fail(errs); }
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5, s6, s7, s8) = ("1. Design data", "2. Section properties", "3. Section classification", "4. Actions", "5. Bending resistance", "6. Shear resistance", "7. Lateral-torsional buckling", "8. Serviceability: deflection");
    t.step(s1, "Geometry and material", "", "", &format!("L = {} mm;  L_cr = {} mm;  h = {} mm;  b = {} mm;  t_w = {} mm;  t_f = {} mm;  r = {} mm;  f_y = {} MPa", span, lcr, h, b, tw, tf, r, fy), "", "Doubly symmetric rolled I/H section, simply supported with fork supports (torsional restraint at the ends).");
    t.step(s1, "Partial factors", "EN 1993-1-1 cl. 6.1(1)", "", &format!("γ_{{M0}} = {:.2};  γ_{{M1}} = {:.2};  γ_G = {:.2};  γ_Q = {:.2};  E = 210000 MPa;  G = 81000 MPa", ps.gamma_m0, ps.gamma_m1, ps.gamma_g, ps.gamma_q), "", &format!("Parameter set: {} [{}]", ps.label, ps.status));
    let p = props(h, b, tw, tf, r);
    let hw = h - 2.0 * tf;
    t.step(s2, "Area and second moments of area", "Section geometry (rectangles + root fillets)", "A = 2·b·t_f + h_w·t_w + (4 − π)·r^{2};   I_y = [b·h^{3} − (b − t_w)·h_w^{3}]/12 + fillets;   I_z = [2·t_f·b^{3} + h_w·t_w^{3}]/12 + fillets", &format!("h_w = {} − 2 × {} = {:.1} mm;  A = {:.0} mm^{{2}}", h, tf, hw, p.a), &format!("A = {:.0} mm^{{2}};  I_{{y}} = {:.3}×10^{{6}} mm^{{4}};  I_{{z}} = {:.3}×10^{{6}} mm^{{4}}", p.a, p.iy / 1e6, p.iz / 1e6), "Root fillets included by the exact centroid method. Compare against the manufacturer's table.");
    t.step(s2, "Section moduli, torsion and warping constants", "Section geometry", "W_{el,y} = I_y/(h/2);   W_{pl,y} = b·t_f·(h − t_f) + t_w·(h_w/2)^{2} + fillets;   I_t ≈ [2·b·t_f^{3} + (h − t_f)·t_w^{3}]/3;   I_w = I_z·(h − t_f)^{2}/4", &format!("W_el = {:.0}/{:.1}", p.iy, h / 2.0), &format!("W_{{el,y}} = {:.1}×10^{{3}} mm^{{3}};  W_{{pl,y}} = {:.1}×10^{{3}} mm^{{3}};  I_t = {:.1}×10^{{3}} mm^{{4}};  I_w = {:.3}×10^{{9}} mm^{{6}}", p.wel / 1e3, p.wpl / 1e3, p.it / 1e3, p.iw / 1e9), "I_t omits the fillet contribution (slightly conservative).");
    // classification
    let eps = (235.0 / fy).sqrt();
    let cf = (b - tw - 2.0 * r) / 2.0; let cw = hw - 2.0 * r;
    let class_of = |ratio: f64, l1: f64, l2: f64, l3: f64| if ratio <= l1 * eps { 1 } else if ratio <= l2 * eps { 2 } else if ratio <= l3 * eps { 3 } else { 4 };
    let cls_f = class_of(cf / tf, 9.0, 10.0, 14.0); let cls_w = class_of(cw / tw, 72.0, 83.0, 124.0);
    let cls = cls_f.max(cls_w);
    t.step(s3, "Slenderness of flange outstand and web", "EN 1993-1-1 Table 5.2", "ε = √(235/f_y);   c_f = (b − t_w − 2r)/2;   c_w = h − 2t_f − 2r;   flange (compression outstand): c_f/t_f ≤ 9ε / 10ε / 14ε;   web (bending): c_w/t_w ≤ 72ε / 83ε / 124ε", &format!("ε = {:.3};  c_f/t_f = {:.1}/{} = {:.2};  c_w/t_w = {:.1}/{} = {:.2}", eps, cf, tf, cf / tf, cw, tw, cw / tw), &format!("Flange class {};  web class {};  section class {}", cls_f, cls_w, cls), "Limits shown for classes 1 / 2 / 3. Web assumed in pure bending, flange fully compressed.");
    if cls == 4 { return fail(vec![format!("Class 4 section (flange class {}, web class {}): effective section properties are not implemented. Choose a more compact section.", cls_f, cls_w)]); }
    // actions
    let sw = p.a / 1e6 * 78.5;
    let w = ps.gamma_g * (gk + sw) + ps.gamma_q * qk;
    let l = span / 1000.0;
    let m_ed = w * l * l / 8.0; let v_ed = w * l / 2.0;
    t.step(s4, "Beam self-weight", "EN 1991-1-1 Table A.4 (steel 78.5 kN/m³)", "g_{sw} = A × 78.5×10^{-6}", &format!("= {:.0} × 78.5 / 10^{{6}}", p.a), &format!("g_{{sw}} = {:.3} kN/m", sw), "");
    t.step(s4, "Design ULS load and action effects", "EN 1990 Eq 6.10", "w_{Ed} = γ_G·(g_k + g_{sw}) + γ_Q·q_k;   M_{Ed} = w_{Ed}·L^{2}/8;   V_{Ed} = w_{Ed}·L/2", &format!("w_Ed = {:.2} × ({} + {:.3}) + {:.2} × {} = {:.2} kN/m;  M_Ed = {:.2} × {:.2}^{{2}}/8;  V_Ed = {:.2} × {:.2}/2", ps.gamma_g, gk, sw, ps.gamma_q, qk, w, w, l, w, l), &format!("M_{{Ed}} = {:.1} kNm;  V_{{Ed}} = {:.1} kN", m_ed, v_ed), "");
    // bending
    let wy = if cls <= 2 { p.wpl } else { p.wel };
    let mc = wy * fy / ps.gamma_m0;
    t.step(s5, "Cross-section bending resistance", "EN 1993-1-1 cl. 6.2.5(2), Eq 6.13/6.14", "M_{c,Rd} = W·f_y/γ_{M0}   (W = W_{pl,y} for class 1-2, W_{el,y} for class 3)", &format!("= {:.1}×10^{{3}} × {} / {:.2}", wy / 1e3, fy, ps.gamma_m0), &format!("M_{{c,Rd}} = {:.1} kNm", mc / 1e6), &format!("Class {} section: {} used.", cls, if cls <= 2 { "plastic modulus" } else { "elastic modulus" }));
    // shear
    let eta = 1.2;
    let vpl = p.av * (fy / 3f64.sqrt()) / ps.gamma_m0;
    t.step(s6, "Plastic shear resistance", "EN 1993-1-1 cl. 6.2.6(2), (3)", "A_v = A − 2·b·t_f + (t_w + 2r)·t_f;   V_{pl,Rd} = A_v·(f_y/√3)/γ_{M0}", &format!("A_v = {:.0} − 2 × {} × {} + ({} + 2 × {}) × {} = {:.0} mm^{{2}};  = {:.0} × ({}/√3)/{:.2}", p.a, b, tf, tw, r, tf, p.av, p.av, fy, ps.gamma_m0), &format!("V_{{pl,Rd}} = {:.1} kN", vpl / 1e3), "");
    let web_ok = hw / tw <= 72.0 * eps / eta;
    t.step(s6, "Shear buckling of the web", "EN 1993-1-5 cl. 5.1(2)", "h_w/t_w ≤ 72·ε/η  (η = 1.2 for f_y ≤ 460 MPa)", &format!("{:.1}/{} = {:.1}  vs  72 × {:.3}/1.2 = {:.1}", hw, tw, hw / tw, eps, 72.0 * eps / eta), if web_ok { "Shear buckling need not be checked" } else { "Web too slender: shear buckling check required (not implemented)" }, "");
    let mut m_eff = mc; let mut reduced = false;
    if v_ed * 1e3 > 0.5 * vpl {
        let rho = (2.0 * v_ed * 1e3 / vpl - 1.0).powi(2);
        let aw = hw * tw;
        let mv = ((p.wpl - rho * aw * aw / (4.0 * tw)) * fy / ps.gamma_m0).min(mc);
        t.step(s6, "Bending resistance reduced for high shear", "EN 1993-1-1 cl. 6.2.8(3), Eq 6.29", "ρ = (2V_{Ed}/V_{pl,Rd} − 1)^{2};   M_{y,V,Rd} = [W_{pl,y} − ρ·A_w^{2}/(4·t_w)]·f_y/γ_{M0} ≤ M_{c,Rd}", &format!("ρ = (2 × {:.1}/{:.1} − 1)^{{2}} = {:.3};  A_w = {:.0} × {} = {:.0} mm^{{2}}", v_ed, vpl / 1e3, rho, hw, tw, aw), &format!("M_{{y,V,Rd}} = {:.1} kNm", mv / 1e6), "V_Ed > 0.5·V_pl,Rd");
        m_eff = mv; reduced = true;
    }
    // LTB
    let c1 = 1.127; let c2 = 0.454;
    let zg = if level == 1.0 { h / 2.0 } else { 0.0 };
    let lc = lcr;
    let k1 = PI * PI * E * p.iz / (lc * lc);
    let term = p.iw / p.iz + lc * lc * G * p.it / (PI * PI * E * p.iz);
    let mcr = c1 * k1 * ((term + (c2 * zg).powi(2)).sqrt() - c2 * zg);
    t.step(s7, "Elastic critical moment for lateral-torsional buckling", "NCCI SN003b (EN 1993-1-1 Annex-style closed form), k = k_z = k_w = 1", "M_{cr} = C_1·(π^{2}·E·I_z/L_{cr}^{2})·{ √[ I_w/I_z + L_{cr}^{2}·G·I_t/(π^{2}·E·I_z) + (C_2·z_g)^{2} ] − C_2·z_g }",
        &format!("C_1 = {} and C_2 = {} (UDL, simply supported);  z_g = {:.1} mm ({});  π²EI_z/L² = {:.1} kN;  I_w/I_z + L²GI_t/(π²EI_z) = {:.1}", c1, c2, zg, if level == 1.0 { "load on top flange, destabilising" } else { "load at shear centre" }, k1 / 1e3, term), &format!("M_{{cr}} = {:.1} kNm", mcr / 1e6), "C1, C2 are taken from NCCI SN003b-EN-EU and are not part of the Eurocode text. Verify before use.");
    let lam = (wy * fy / mcr).sqrt();
    let (lam0, beta) = (0.4, 0.75);
    let curve = if h / b <= 2.0 { ("b", 0.34) } else { ("c", 0.49) };
    let phi = 0.5 * (1.0 + curve.1 * (lam - lam0) + beta * lam * lam);
    let chi = (1.0 / (phi + (phi * phi - beta * lam * lam).sqrt())).min(1.0).min(1.0 / (lam * lam));
    let skip = lam <= lam0 || (m_ed * 1e6 / mcr) <= lam0 * lam0;
    let chi_use = if skip { 1.0 } else { chi };
    t.step(s7, "Non-dimensional slenderness", "EN 1993-1-1 cl. 6.3.2.2(1), Eq 6.56", "λ_{LT} = √(W·f_y/M_{cr})", &format!("= √({:.1}×10^{{3}} × {} / {:.1}×10^{{6}})", wy / 1e3, fy, mcr / 1e6), &format!("λ_{{LT}} = {:.3}", lam), "");
    t.step(s7, "Reduction factor, rolled sections", "EN 1993-1-1 cl. 6.3.2.3, Eq 6.57, Table 6.3 and 6.5", "Φ_{LT} = 0.5·[1 + α_{LT}·(λ_{LT} − 0.4) + 0.75·λ_{LT}^{2}];   χ_{LT} = 1/[Φ_{LT} + √(Φ_{LT}^{2} − 0.75·λ_{LT}^{2})] ≤ 1,  ≤ 1/λ_{LT}^{2}",
        &format!("h/b = {:.2} → buckling curve {} (α_LT = {});  Φ_LT = 0.5 × [1 + {} × ({:.3} − 0.4) + 0.75 × {:.3}^{{2}}] = {:.3}", h / b, curve.0, curve.1, curve.1, lam, lam, phi), &format!("χ_{{LT}} = {:.3}{}", chi, if skip { "  (LTB check not needed: λ_LT ≤ 0.4 or M_Ed/M_cr ≤ 0.16, χ_LT = 1.0 used)" } else { "" }), "The modification factor f for the moment distribution (Eq 6.58) is conservatively not applied.");
    let mb = chi_use * wy * fy / ps.gamma_m1;
    t.step(s7, "Design buckling resistance moment", "EN 1993-1-1 cl. 6.3.2.1, Eq 6.55", "M_{b,Rd} = χ_{LT}·W·f_y/γ_{M1}", &format!("= {:.3} × {:.1}×10^{{3}} × {} / {:.2}", chi_use, wy / 1e3, fy, ps.gamma_m1), &format!("M_{{b,Rd}} = {:.1} kNm", mb / 1e6), "");
    // deflection
    let w_var = qk;
    let delta = 5.0 * w_var * span.powi(4) / (384.0 * E * p.iy);
    let lim = span / dr;
    t.step(s8, "Deflection under the variable load", "EN 1993-1-1 cl. 7.2.1; EN 1990 A1.4", "δ = 5·q_k·L^{4}/(384·E·I_y);   limit = L/ratio", &format!("= 5 × {} × {}^{{4}} / (384 × 210000 × {:.3}×10^{{6}});  limit = {}/{}", qk, span, p.iy / 1e6, span, dr), &format!("δ = {:.1} mm  {} L/{} = {:.1} mm", delta, if delta <= lim { "≤" } else { ">" }, dr, lim), "The deflection limit is a project/National Annex value supplied by the user.");
    let mut checks = vec![];
    checks.push(chk("Bending resistance", "EN 1993-1-1 6.2.5", format!("{:.1} kNm", m_ed), format!("{:.1} kNm", m_eff / 1e6), m_ed * 1e6 / m_eff, m_ed * 1e6 <= m_eff));
    checks.push(chk("Shear resistance", "EN 1993-1-1 6.2.6", format!("{:.1} kN", v_ed), format!("{:.1} kN", vpl / 1e3), v_ed * 1e3 / vpl, v_ed * 1e3 <= vpl));
    checks.push(chk("Web shear buckling", "EN 1993-1-5 5.1(2)", format!("{:.1}", hw / tw), format!("{:.1} max", 72.0 * eps / eta), (hw / tw) / (72.0 * eps / eta), web_ok));
    checks.push(chk("Lateral-torsional buckling", "EN 1993-1-1 6.3.2", format!("{:.1} kNm", m_ed), format!("{:.1} kNm", mb / 1e6), m_ed * 1e6 / mb, m_ed * 1e6 <= mb));
    checks.push(chk("Deflection", "EN 1993-1-1 7.2.1", format!("{:.1} mm", delta), format!("{:.1} mm", lim), delta / lim, delta <= lim));
    let mut obs: Vec<(&str, String)> = vec![("AMBER", "LTB coefficients C1 = 1.127 and C2 = 0.454 come from NCCI SN003b, not the Eurocode text. Verify, or confirm the Mcr with an independent source.".to_string()), ("INFO", "Section properties are computed from the entered dimensions and should be compared with the manufacturer's table.".to_string())];
    if reduced { obs.push(("INFO", "Bending resistance reduced for high shear (V_Ed > 0.5·V_pl,Rd).".to_string())); }
    if cls == 3 { obs.push(("INFO", "Class 3 section: elastic modulus used.".to_string())); }
    let summary = vec![kv("Section class", format!("{} (flange {}, web {})", cls, cls_f, cls_w)), kv("A / I y", format!("{:.0} mm² / {:.2}×10⁶ mm⁴", p.a, p.iy / 1e6)), kv("W pl,y / W el,y", format!("{:.1} / {:.1} ×10³ mm³", p.wpl / 1e3, p.wel / 1e3)), kv("M Ed / V Ed", format!("{:.1} kNm / {:.1} kN", m_ed, v_ed)), kv("M c,Rd / M b,Rd", format!("{:.1} / {:.1} kNm", m_eff / 1e6, mb / 1e6)), kv("M cr / χ LT", format!("{:.1} kNm / {:.3}", mcr / 1e6, chi_use)), kv("Deflection", format!("{:.1} mm (limit {:.1})", delta, lim))];
    let mut res = finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: classification (classes 1-3), bending, shear and web shear buckling screening, bending-shear interaction, LTB for rolled sections, deflection.".into(),
        "NOT implemented: class 4 sections, section selection from tables, point loads and other load cases, intermediate lateral restraints, axial load, bearing and web resistance to transverse forces, fire.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::ISection { h, b, tw, tf });
    res.engine_version = crate::ENGINE_VERSION.into();
    res
}
