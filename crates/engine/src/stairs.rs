//! One-way simply supported RC slab, per metre width (EN 1992-1-1:2004).
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

pub fn def() -> ModDef {
    ModDef { id: "stairs", name: "Staircase", title: "Reinforced concrete staircase (waist slab flight)", subtitle: "EN 1992-1-1:2004 · Simply supported flight, design on the horizontal span, 1 m width", available: true, has_check_mode: true, fields: vec![
        fld("span", "Horizontal span L (between supports)", "mm", "Geometry", 3000.0), fld("h", "Waist thickness h (perpendicular)", "mm", "Geometry", 150.0), fld("rise", "Riser height", "mm", "Geometry", 175.0), fld("going", "Going (tread)", "mm", "Geometry", 250.0),
        fld("cover", "Cover c", "mm", "Geometry", 25.0), fld("bar_dia", "Main bar φ", "mm", "Geometry", 12.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0),
        fld("gk", "Finishes gk (per m² of plan)", "kN/m²", "Actions", 1.5), fld("qk", "Imposed qk (per m² of plan)", "kN/m²", "Actions", 4.0),
        fld_chk("bar_spacing", "Main bar spacing (drawing)", "mm", "Provided reinforcement", 150.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let check = v.get("mode").and_then(|m| m.as_str()) == Some("check");
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, check) { Ok(x) => x, Err(e) => return fail(e) };
    let (span, h, rise, going, cover, phi, fck, fyk, gk, qk, s_in) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10]);
    if fck > 50.0 || fck < 12.0 { return fail(vec!["fck must be within 12-50 MPa for this release".into()]); }
    let b = 1000.0;
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5, s6) = ("1. Design data", "2. Actions", "3. Effective depth and strengths", "4. Flexural design", "5. Shear", "6. Serviceability: deflection");
    t.step(s1, "Geometry and materials", "", "", &format!("Span L = {} mm;  h = {} mm;  cover c = {} mm;  main bar φ = {} mm;  f_{{ck}} = {} MPa;  f_{{yk}} = {} MPa", span, h, cover, phi, fck, fyk), "", "Simply supported, one-way spanning, 1.0 m design strip (b = 1000 mm).");
    t.step(s1, "Partial factors", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6; EN 1990 Eq 6.10", "", &format!("γ_{{c}} = {:.2};  γ_{{s}} = {:.2};  α_{{cc}} = {:.2};  γ_{{G}} = {:.2};  γ_{{Q}} = {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc, ps.gamma_g, ps.gamma_q), "", &format!("Parameter set: {} [{}]", ps.label, ps.status));
    let cosa = going / (rise * rise + going * going).sqrt();
    let sw_w = 25.0 * (h / 1000.0) / cosa; let sw_s = 25.0 * (rise / 1000.0) / 2.0; let sw = sw_w + sw_s;
    t.step(s2, "Self-weight per m² of plan", "EN 1991-1-1 Table A.1 (25 kN/m³)", "cos α = G/√(R^{2} + G^{2});   g_{sw} = 25·(h/cos α) + 25·R/2", &format!("cos α = {} / √({}^{{2}} + {}^{{2}}) = {:.4};  g_sw = 25 × {:.3}/{:.4} + 25 × {:.3}/2 = {:.2} + {:.2}", going, rise, going, cosa, h / 1000.0, cosa, rise / 1000.0, sw_w, sw_s), &format!("g_{{sw}} = {:.2} kN/m²", sw), "Waist weight on the slope plus the average triangular step weight.");
    let w = ps.gamma_g * (gk + sw) + ps.gamma_q * qk;
    t.step(s2, "Design ULS load per metre width", "EN 1990 Eq 6.10", "w_{Ed} = γ_G·(g_k + g_{sw}) + γ_Q·q_k", &format!("= {:.2} × ({} + {:.2}) + {:.2} × {}", ps.gamma_g, gk, sw, ps.gamma_q, qk), &format!("w_{{Ed}} = {:.2} kN/m", w), "Single combination (permanent + one variable action).");
    let l = span / 1000.0;
    let m_ed = w * l * l / 8.0; let v_ed = w * l / 2.0;
    t.step(s2, "Design moment and shear", "Statics, simply supported", "M_{Ed} = w_{Ed}·L^{2}/8;   V_{Ed} = w_{Ed}·L/2", &format!("= {:.2} × {:.2}^{{2}}/8;   = {:.2} × {:.2}/2", w, l, w, l), &format!("M_{{Ed}} = {:.2} kNm/m;  V_{{Ed}} = {:.2} kN/m", m_ed, v_ed), "Shear taken at the support face (conservative).");
    let d = h - cover - phi / 2.0;
    if d <= 0.0 { return fail(vec!["Effective depth not positive".into()]); }
    let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    t.step(s3, "Effective depth and design strengths", "EN 1992-1-1 cl. 3.1.6, 2.4.2.4", "d = h − c − φ/2;  f_{cd} = α_{cc}·f_{ck}/γ_c;  f_{yd} = f_{yk}/γ_s", &format!("d = {} − {} − {}/2 = {:.1} mm;  f_cd = {:.2} × {} / {:.2};  f_yd = {} / {:.2}", h, cover, phi, d, ps.alpha_cc, fck, ps.gamma_c, fyk, ps.gamma_s), &format!("d = {:.1} mm;  f_{{cd}} = {:.2} MPa;  f_{{yd}} = {:.2} MPa", d, fcd, fyd), "");
    let (as_req, _z, _mu) = match flex_single(&mut t, s4, b, d, fck, fcd, fyd, m_ed * 1e6, "") { Ok(r) => r, Err(e) => return fail(vec![e]) };
    let fctm = 0.30 * fck.powf(2.0 / 3.0);
    let as_min = (0.26 * fctm / fyk * b * d).max(0.0013 * b * d);
    let as_max = 0.04 * b * h;
    t.step(s4, "Minimum and maximum reinforcement", "EN 1992-1-1 cl. 9.2.1.1(1), 9.2.1.1(3), 9.3.1.1(1)", "f_{ctm} = 0.30·f_{ck}^{2/3};  A_{s,min} = max[0.26·(f_{ctm}/f_{yk})·b·d ; 0.0013·b·d];  A_{s,max} = 0.04·b·h", &format!("f_ctm = {:.2} MPa;  A_s,min = max[{:.0} ; {:.0}];  A_s,max = 0.04 × 1000 × {}", fctm, 0.26 * fctm / fyk * b * d, 0.0013 * b * d, h), &format!("A_{{s,min}} = {:.0} mm^{{2}}/m;  A_{{s,max}} = {:.0} mm^{{2}}/m", as_min, as_max), "");
    let as_design = as_req.max(as_min);
    let area1 = PI * phi * phi / 4.0;
    let s_max = (3.0 * h).min(400.0);
    let spacing = if check { s_in } else {
        let s_calc = 1000.0 * area1 / as_design;
        let s = (s_calc.min(s_max) / 25.0).floor() * 25.0;
        t.step(s4, "Main bar spacing", "EN 1992-1-1 cl. 9.3.1.1(3)", "s ≤ 1000·A_{bar}/A_{s,design};   s_{max} = min(3h ; 400)", &format!("A_bar = π × {}^{{2}}/4 = {:.1} mm^{{2}};  s ≤ 1000 × {:.1} / {:.0} = {:.0} mm;  s_max = min({} ; 400) = {:.0} mm", phi, area1, area1, as_design, s_calc, 3.0 * h, s_max), &format!("Use T{} @ {:.0} mm c/c", phi, s), "Rounded down to a multiple of 25 mm.");
        s
    };
    if spacing < 50.0 { return fail(vec!["Required bar spacing below 50 mm: increase bar size or slab thickness".into()]); }
    let as_prov = 1000.0 * area1 / spacing;
    t.step(s4, if check { "Provided reinforcement (from drawing)" } else { "Provided reinforcement" }, "", "A_{s,prov} = 1000·π·φ^{2}/(4·s)", &format!("= 1000 × π × {}^{{2}} / (4 × {:.0})", phi, spacing), &format!("A_{{s,prov}} = {:.0} mm^{{2}}/m  (T{} @ {:.0})", as_prov, phi, spacing), "");
    let x = as_prov * fyd / (0.8 * b * fcd); let zz = d - 0.4 * x; let mrd = as_prov * fyd * zz;
    t.step(s4, "Moment resistance of provided steel", "EN 1992-1-1 cl. 3.1.7, 6.1", "x = A_{s,prov}·f_{yd}/(0.8·b·f_{cd});  z = d − 0.4x;  M_{Rd} = A_{s,prov}·f_{yd}·z", &format!("x = {:.0} × {:.2}/(0.8 × 1000 × {:.2}) = {:.1} mm;  z = {:.1} − 0.4 × {:.1} = {:.1} mm", as_prov, fyd, fcd, x, d, x, zz), &format!("M_{{Rd}} = {:.2} kNm/m;  x/d = {:.3}", mrd / 1e6, x / d), "");
    let mut checks = vec![];
    checks.push(chk("Flexural resistance", "EN 1992-1-1 6.1", format!("{:.2} kNm/m", m_ed), format!("{:.2} kNm/m", mrd / 1e6), m_ed * 1e6 / mrd, m_ed * 1e6 <= mrd && x / d <= 0.45 + 1e-9));
    checks.push(chk("Minimum reinforcement", "EN 1992-1-1 9.2.1.1(1)", format!("{:.0} mm²/m", as_min), format!("{:.0} mm²/m", as_prov), as_min / as_prov, as_prov >= as_min));
    checks.push(chk("Maximum reinforcement", "EN 1992-1-1 9.2.1.1(3)", format!("{:.0} mm²/m", as_prov), format!("{:.0} mm²/m", as_max), as_prov / as_max, as_prov <= as_max));
    checks.push(chk("Bar spacing (max)", "EN 1992-1-1 9.3.1.1(3)", format!("{:.0} mm", spacing), format!("{:.0} mm max", s_max), spacing / s_max, spacing <= s_max + 1e-9));
    checks.push(chk("Bar spacing (clear)", "EN 1992-1-1 8.2(2)", format!("{:.0} mm min", phi.max(25.0)), format!("{:.0} mm clear", spacing - phi), phi.max(25.0) / (spacing - phi).max(0.1), spacing - phi >= phi.max(25.0)));
    // shear
    let (vrdc, _, _) = vrd_c(&mut t, s5, b, d, fck, ps.gamma_c, as_prov, "");
    checks.push(chk("Shear (no links)", "EN 1992-1-1 6.2.2(1)", format!("{:.1} kN/m", v_ed), format!("{:.1} kN/m", vrdc / 1e3), v_ed * 1e3 / vrdc, v_ed * 1e3 <= vrdc));
    // deflection
    let rho = as_req / (b * d); let rho0 = fck.sqrt() / 1000.0; let kf = 1.0;
    let basic = if rho <= rho0 { kf * (11.0 + 1.5 * fck.sqrt() * rho0 / rho + 3.2 * fck.sqrt() * (rho0 / rho - 1.0).powf(1.5)) } else { kf * (11.0 + 1.5 * fck.sqrt() * rho0 / rho) };
    let basic = if rho <= rho0 { basic } else { basic + 0.0 };
    let modf = ((500.0 / fyk) * (as_prov / as_req.max(1e-9))).min(1.5);
    let allow = basic * modf;
    let actual = span / d;
    t.step(s6, "Span/effective depth limit", "EN 1992-1-1 cl. 7.4.2, Eq 7.16a/b, Table 7.4N (K = 1.0 simply supported)", "ρ_0 = √f_{ck}/1000;  ρ = A_{s,req}/(b·d);  if ρ ≤ ρ_0:  l/d = K·[11 + 1.5·√f_{ck}·ρ_0/ρ + 3.2·√f_{ck}·(ρ_0/ρ − 1)^{3/2}],  else l/d = K·[11 + 1.5·√f_{ck}·ρ_0/ρ]",
        &format!("ρ_0 = {:.5};  ρ = {:.5};  basic l/d = {:.1};  modification = (500/f_yk)(A_s,prov/A_s,req) ≤ 1.5 = {:.2};  allowable = {:.1} × {:.2}", rho0, rho, basic, modf, basic, modf), &format!("(l/d)_{{allow}} = {:.1};  (l/d)_{{actual}} = {:.0}/{:.1} = {:.1}", allow, span, d, actual), "Compression steel ignored (ρ' = 0). The cap of 1.5 is a National Annex value and is not verified for Uganda.");
    checks.push(chk("Deflection (span/depth)", "EN 1992-1-1 7.4.2", format!("{:.1}", actual), format!("{:.1} allowed", allow), actual / allow, actual <= allow));
    // distribution steel
    let as_dist = 0.2 * as_prov;
    let dia2 = if phi >= 16.0 { 10.0 } else { 8.0 };
    let s2d = ((1000.0 * PI * dia2 * dia2 / 4.0 / as_dist) / 25.0).floor() * 25.0;
    t.step(s4, "Secondary (distribution) reinforcement", "EN 1992-1-1 cl. 9.3.1.1(2)", "A_{s,sec} ≥ 0.20·A_{s,prov}", &format!("= 0.20 × {:.0} = {:.0} mm^{{2}}/m;  T{} @ s ≤ {:.0} mm", as_prov, as_dist, dia2, s2d), &format!("Use T{} @ {:.0} mm c/c (secondary)", dia2, s2d.min(3.5 * h).min(450.0)), "Not part of the PASS/FAIL set; detail on drawings.");
    let summary = vec![kv("Effective depth d", format!("{:.1} mm", d)), kv("Design load w Ed", format!("{:.2} kN/m", w)), kv("M Ed / V Ed", format!("{:.2} kNm / {:.2} kN", m_ed, v_ed)), kv("Required steel As,req", format!("{:.0} mm²/m", as_req)), kv("Main reinforcement", format!("T{} @ {:.0}  ({:.0} mm²/m)", phi, spacing, as_prov)), kv("Secondary", format!("T{} @ {:.0}", dia2, s2d.min(3.5 * h).min(450.0))), kv("Span/depth", format!("{:.1} (limit {:.1})", actual, allow))];
    let mut res =     finish(t, checks, &ps.status, summary, vec![], { let tr = 2.0 * rise + going; if tr < 550.0 || tr > 700.0 { vec![("AMBER", format!("2R + G = {:.0} mm is outside the common 550-700 mm comfort range. Check against the applicable building code.", tr))] } else { vec![] } }, vec![
        "Implemented: loads on plan, flexure, min/max steel, bar spacing, shear without links, span/depth deflection check, distribution steel.".into(),
        "NOT implemented: landings, flights spanning longitudinally with landings, dynamic (vibration) check, cracking, anchorage and curtailment, stair geometry rules of the national building code.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Slab { h, cover, bar_dia: phi, spacing });
    res.outputs = vec![out("m_ed", "Design moment M Ed", m_ed, "kNm/m"), out("v_ed", "Support reaction (ULS)", v_ed, "kN/m"), out("reaction", "Support reaction (ULS)", v_ed, "kN/m")];
    res
}
