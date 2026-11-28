//! Strip footing under a concentric wall line load, per metre length (EN 1992-1-1:2004; bearing by allowable pressure).
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

pub fn def() -> ModDef {
    ModDef { id: "strip", name: "Strip footing", title: "Reinforced concrete strip footing", subtitle: "EN 1992-1-1:2004 · Concentric wall line load per metre, allowable bearing pressure supplied by geotechnical engineer", available: true, has_check_mode: true, fields: vec![
        fld("n_ed", "Wall load N Ed (ULS)", "kN/m", "Actions", 300.0), fld("n_k", "Wall load N k (SLS)", "kN/m", "Actions", 220.0), fld("q_allow", "Allowable bearing pressure", "kPa", "Ground", 150.0),
        fld("t", "Wall thickness t", "mm", "Wall", 225.0),
        fld("h", "Footing thickness h", "mm", "Footing", 400.0), fld("cover", "Cover c (to ground)", "mm", "Footing", 50.0), fld("bar_dia", "Transverse bar φ", "mm", "Footing", 12.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0),
        fld_chk("width", "Footing width B (drawing)", "mm", "Provided", 1800.0), fld_chk("bar_spacing", "Transverse bar spacing (drawing)", "mm", "Provided", 200.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let check = v.get("mode").and_then(|m| m.as_str()) == Some("check");
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, check) { Ok(x) => x, Err(e) => return fail(e) };
    let (n_ed, n_k, q_allow, wt, h, cover, phi, fck, fyk, w_in, s_in) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10]);
    if !(12.0..=50.0).contains(&fck) { return fail(vec!["fck must be within 12-50 MPa for this release".into()]); }
    let mut t = Trace::new();
    let (s1, s2, s3, s4) = ("1. Design data", "2. Width and bearing", "3. Flexural design (transverse)", "4. One-way shear");
    t.step(s1, "Loads and ground", "", "", &format!("N_{{Ed}} = {} kN/m (ULS);  N_{{k}} = {} kN/m (SLS);  q_{{allow}} = {} kPa", n_ed, n_k, q_allow), "", "q_allow is supplied by the geotechnical engineer.");
    t.step(s1, "Geometry and materials", "", "", &format!("Wall t = {} mm;  h = {} mm;  c = {} mm;  bar φ = {} mm;  f_{{ck}} = {} MPa;  f_{{yk}} = {} MPa", wt, h, cover, phi, fck, fyk), "", "Per metre length. Concentric vertical line load only.");
    t.step(s1, "Partial factors", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6", "", &format!("γ_{{c}} = {:.2};  γ_{{s}} = {:.2};  α_{{cc}} = {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc), "", &format!("Parameter set: {} [{}]", ps.label, ps.status));
    let sw = |bm: f64| 25.0 * (bm / 1e3) * (h / 1e3);
    let width = if check { w_in } else {
        let mut bw = (n_k / q_allow * 1000.0 / 50.0).ceil() * 50.0;
        while (n_k + sw(bw)) / (bw / 1e3) > q_allow && bw < 20000.0 { bw += 50.0; }
        t.step(s2, "Width from service bearing pressure", "EN 1997-1 cl. 6.5.2", "(N_k + G_{ftg}) / B ≤ q_{allow};   G_{ftg} = 25·B·h", "B increased in 50 mm steps from N_k/q_allow until the condition is met", &format!("B = {:.0} mm", bw), "");
        bw
    };
    if width <= wt { return fail(vec!["Footing must be wider than the wall".into()]); }
    let bm = width / 1e3; let g = sw(width); let q_ser = (n_k + g) / bm;
    t.step(s2, "Service bearing pressure", "EN 1997-1 cl. 6.5.2", "G_{ftg} = 25·B·h;   q_{ser} = (N_k + G_{ftg})/B", &format!("G_ftg = 25 × {:.3} × {:.3} = {:.1} kN/m;  q_ser = ({} + {:.1}) / {:.3}", bm, h / 1e3, g, n_k, g, bm), &format!("q_{{ser}} = {:.1} kPa  {} q_{{allow}} = {} kPa", q_ser, if q_ser <= q_allow { "≤" } else { ">" }, q_allow), "Uniform pressure under a concentric load.");
    let q_u = n_ed / bm;
    t.step(s2, "Net ULS ground pressure", "", "q_{u} = N_{Ed}/B", &format!("= {} / {:.3}", n_ed, bm), &format!("q_{{u}} = {:.1} kPa", q_u), "");
    let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    let d = h - cover - phi / 2.0;
    if d <= 0.0 { return fail(vec!["Effective depth not positive".into()]); }
    t.step(s3, "Effective depth and design strengths", "EN 1992-1-1 cl. 3.1.6, 2.4.2.4", "d = h − c − φ/2;  f_{cd} = α_{cc}·f_{ck}/γ_c;  f_{yd} = f_{yk}/γ_s", &format!("d = {} − {} − {}/2 = {:.1} mm;  f_cd = {:.2} × {} / {:.2};  f_yd = {} / {:.2}", h, cover, phi, d, ps.alpha_cc, fck, ps.gamma_c, fyk, ps.gamma_s), &format!("d = {:.1} mm;  f_{{cd}} = {:.2} MPa;  f_{{yd}} = {:.2} MPa", d, fcd, fyd), "");
    let a = (width - wt) / 2.0 / 1e3; let m_ed = q_u * a * a / 2.0;
    t.step(s3, "Design moment at the wall face (per metre length)", "EN 1992-1-1 cl. 5.3.2 (cantilever from wall face)", "a = (B − t)/2;   M_{Ed} = q_{u}·a^{2}/2", &format!("a = ({} − {})/2 = {:.3} m;  M_Ed = {:.1} × {:.3}^{{2}}/2", width, wt, a, q_u, a), &format!("M_{{Ed}} = {:.2} kNm/m", m_ed), "");
    let b = 1000.0;
    let (as_req, _, _) = match flex_single(&mut t, s3, b, d, fck, fcd, fyd, m_ed * 1e6, "") { Ok(r) => r, Err(e) => return fail(vec![e]) };
    let fctm = 0.30 * fck.powf(2.0 / 3.0);
    let as_min = (0.26 * fctm / fyk * b * d).max(0.0013 * b * d);
    t.step(s3, "Minimum reinforcement", "EN 1992-1-1 cl. 9.2.1.1(1)", "A_{s,min} = max[0.26·(f_{ctm}/f_{yk})·b·d ; 0.0013·b·d]", &format!("f_ctm = {:.2} MPa;  = max[{:.0} ; {:.0}]", fctm, 0.26 * fctm / fyk * b * d, 0.0013 * b * d), &format!("A_{{s,min}} = {:.0} mm^{{2}}/m", as_min), "");
    let as_design = as_req.max(as_min); let area1 = PI * phi * phi / 4.0; let s_max = (3.0 * h).min(400.0);
    let spacing = if check { s_in } else {
        let s_calc = 1000.0 * area1 / as_design; let s = (s_calc.min(s_max) / 25.0).floor() * 25.0;
        t.step(s3, "Transverse bar spacing", "EN 1992-1-1 cl. 9.3.1.1(3)", "s ≤ 1000·A_{bar}/A_{s,design};   s_{max} = min(3h ; 400)", &format!("A_bar = {:.1} mm^{{2}};  s ≤ 1000 × {:.1}/{:.0} = {:.0} mm;  s_max = {:.0} mm", area1, area1, as_design, s_calc, s_max), &format!("Use T{} @ {:.0} mm c/c", phi, s), "Rounded down to a multiple of 25 mm.");
        s
    };
    if spacing < 50.0 { return fail(vec!["Required spacing below 50 mm: increase bar size or thickness".into()]); }
    let as_prov = 1000.0 * area1 / spacing;
    let xx = as_prov * fyd / (0.8 * b * fcd); let zz = d - 0.4 * xx; let mrd = as_prov * fyd * zz;
    t.step(s3, "Moment resistance of provided steel", "EN 1992-1-1 cl. 3.1.7, 6.1", "A_{s,prov} = 1000·A_{bar}/s;   x = A_{s,prov}·f_{yd}/(0.8·b·f_{cd});   M_{Rd} = A_{s,prov}·f_{yd}·(d − 0.4x)", &format!("A_s,prov = {:.0} mm^{{2}}/m;  x = {:.1} mm;  z = {:.1} mm", as_prov, xx, zz), &format!("M_{{Rd}} = {:.1} kNm/m;  x/d = {:.3}", mrd / 1e6, xx / d), "");
    let mut checks = vec![];
    checks.push(chk("Bearing pressure (SLS)", "EN 1997-1 6.5.2", format!("{:.1} kPa", q_ser), format!("{:.0} kPa", q_allow), q_ser / q_allow, q_ser <= q_allow));
    checks.push(chk("Flexural resistance", "EN 1992-1-1 6.1", format!("{:.2} kNm/m", m_ed), format!("{:.1} kNm/m", mrd / 1e6), m_ed * 1e6 / mrd, m_ed * 1e6 <= mrd && xx / d <= 0.45 + 1e-9));
    checks.push(chk("Minimum reinforcement", "EN 1992-1-1 9.2.1.1(1)", format!("{:.0} mm²/m", as_min), format!("{:.0} mm²/m", as_prov), as_min / as_prov, as_prov >= as_min));
    checks.push(chk("Bar spacing (max)", "EN 1992-1-1 9.3.1.1(3)", format!("{:.0} mm", spacing), format!("{:.0} mm max", s_max), spacing / s_max, spacing <= s_max + 1e-9));
    checks.push(chk("Bar spacing (clear)", "EN 1992-1-1 8.2(2)", format!("{:.0} mm min", phi.max(25.0)), format!("{:.0} mm clear", spacing - phi), phi.max(25.0) / (spacing - phi).max(0.1), spacing - phi >= phi.max(25.0)));
    let v_ed = q_u * (a - d / 1e3).max(0.0);
    t.step(s4, "Design shear at d from the wall face (per metre length)", "EN 1992-1-1 cl. 6.2.1(8), 6.2.2", "V_{Ed} = q_{u}·(a − d)", &format!("= {:.1} × ({:.3} − {:.3})", q_u, a, d / 1e3), &format!("V_{{Ed}} = {:.1} kN/m", v_ed), "Enhancement for loads close to supports is conservatively ignored.");
    let (vrdc, _, _) = vrd_c(&mut t, s4, b, d, fck, ps.gamma_c, as_prov, "");
    checks.push(chk("One-way shear", "EN 1992-1-1 6.2.2(1)", format!("{:.1} kN/m", v_ed), format!("{:.1} kN/m", vrdc / 1e3), v_ed * 1e3 / vrdc, v_ed * 1e3 <= vrdc));
    let as_dist = 0.2 * as_prov; let dia2 = if phi >= 16.0 { 10.0 } else { 8.0 };
    let s2d = (((1000.0 * PI * dia2 * dia2 / 4.0 / as_dist) / 25.0).floor() * 25.0).min(450.0);
    t.step(s3, "Longitudinal (distribution) reinforcement", "EN 1992-1-1 cl. 9.3.1.1(2)", "A_{s,long} ≥ 0.20·A_{s,prov}", &format!("= 0.20 × {:.0} = {:.0} mm^{{2}}/m", as_prov, as_dist), &format!("Use T{} @ {:.0} mm c/c", dia2, s2d), "Not part of the PASS/FAIL set; detail on drawings.");
    let summary = vec![kv("Footing width B", format!("{:.0} mm", width)), kv("Thickness / depth d", format!("{:.0} / {:.1} mm", h, d)), kv("Service pressure", format!("{:.1} kPa (allow {:.0})", q_ser, q_allow)), kv("Net ULS pressure q u", format!("{:.1} kPa", q_u)), kv("Design moment M Ed", format!("{:.2} kNm/m", m_ed)), kv("Transverse steel", format!("T{} @ {:.0}  ({:.0} mm²/m)", phi, spacing, as_prov)), kv("Longitudinal steel", format!("T{} @ {:.0}", dia2, s2d))];
    finish(t, checks, &ps.status, summary, vec![], vec![("INFO", "Allowable bearing pressure must come from a geotechnical report. Sliding, overturning and settlement are not assessed.".to_string())], vec![
        "Implemented: bearing (allowable pressure), transverse flexure, minimum steel, bar spacing, one-way shear, distribution steel.".into(),
        "NOT implemented: eccentric or moment loads, ULS geotechnical bearing (EN 1997-1), settlement, sliding, overturning, wall stem design, anchorage.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Strip { width, t: wt, h })
}
