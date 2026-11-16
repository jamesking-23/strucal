//! Isolated RC pad footing under a concentric axial column load (EN 1992-1-1:2004, EN 1997-1 bearing by allowable pressure).
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

pub fn def() -> ModDef {
    ModDef { id: "pad", name: "Pad footing", title: "Reinforced concrete pad footing", subtitle: "EN 1992-1-1:2004 · Concentric axial load, allowable bearing pressure supplied by geotechnical engineer", available: true, has_check_mode: true, fields: vec![
        fld("n_ed", "Column load N Ed (ULS)", "kN", "Actions", 1200.0), fld("n_k", "Column load N k (SLS)", "kN", "Actions", 870.0), fld("q_allow", "Allowable bearing pressure", "kPa", "Ground", 200.0),
        fld("cx", "Column width cx", "mm", "Column", 300.0), fld("cy", "Column depth cy", "mm", "Column", 300.0),
        fld("h", "Footing thickness h", "mm", "Footing", 500.0), fld("cover", "Cover c (to ground)", "mm", "Footing", 50.0), fld("bar_dia", "Bar φ", "mm", "Footing", 16.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0),
        fld_chk("bx", "Footing width B (drawing)", "mm", "Provided", 2000.0), fld_chk("by", "Footing length L (drawing)", "mm", "Provided", 2000.0), fld_chk("bar_spacing", "Bar spacing, both ways (drawing)", "mm", "Provided", 150.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let check = v.get("mode").and_then(|m| m.as_str()) == Some("check");
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, check) { Ok(x) => x, Err(e) => return fail(e) };
    let (n_ed, n_k, q_allow, cx, cy, h, cover, phi, fck, fyk, bx_in, by_in, s_in) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11], x[12]);
    if fck > 50.0 || fck < 12.0 { return fail(vec!["fck must be within 12-50 MPa for this release".into()]); }
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = ("1. Design data", "2. Plan size and bearing", "3. Flexural design", "4. One-way shear", "5. Punching shear");
    t.step(s1, "Loads and ground", "", "", &format!("N_{{Ed}} = {} kN (ULS);  N_{{k}} = {} kN (SLS);  allowable bearing pressure q_{{allow}} = {} kPa", n_ed, n_k, q_allow), "", "q_allow is supplied by the geotechnical engineer. StruCal does not derive it from soil data.");
    t.step(s1, "Geometry and materials", "", "", &format!("Column {} × {} mm;  h = {} mm;  cover c = {} mm;  bar φ = {} mm;  f_{{ck}} = {} MPa;  f_{{yk}} = {} MPa", cx, cy, h, cover, phi, fck, fyk), "", "Concentric vertical load only. No moments, backfill or surcharge.");
    t.step(s1, "Partial factors", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6", "", &format!("γ_{{c}} = {:.2};  γ_{{s}} = {:.2};  α_{{cc}} = {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc), "", &format!("Parameter set: {} [{}]", ps.label, ps.status));

    let sw_of = |b: f64, l: f64| 25.0 * (b / 1e3) * (l / 1e3) * (h / 1e3);
    let (bx, by) = if check { (bx_in, by_in) } else {
        let mut b = ((n_k / q_allow).sqrt() * 1000.0 / 50.0).ceil() * 50.0;
        while (n_k + sw_of(b, b)) / ((b / 1e3) * (b / 1e3)) > q_allow && b < 20000.0 { b += 50.0; }
        t.step(s2, "Plan size from service bearing pressure", "EN 1997-1 cl. 6.5.2 (allowable pressure approach)", "(N_k + G_{ftg}) / (B·L) ≤ q_{allow};   G_{ftg} = 25·B·L·h", &format!("Square footing, B increased in 50 mm steps from √(N_k/q_allow) until the condition is met"), &format!("B = L = {:.0} mm", b), "");
        (b, b)
    };
    if bx < cx || by < cy { return fail(vec!["Footing must be larger than the column".into()]); }
    let a_ftg = (bx / 1e3) * (by / 1e3);
    let g = sw_of(bx, by);
    let q_ser = (n_k + g) / a_ftg;
    t.step(s2, "Service bearing pressure", "EN 1997-1 cl. 6.5.2", "G_{ftg} = 25·B·L·h;   q_{ser} = (N_k + G_{ftg}) / (B·L)", &format!("G_ftg = 25 × {:.2} × {:.2} × {:.3} = {:.1} kN;  q_ser = ({} + {:.1}) / {:.3}", bx / 1e3, by / 1e3, h / 1e3, g, n_k, g, a_ftg), &format!("q_{{ser}} = {:.1} kPa  {} q_{{allow}} = {} kPa", q_ser, if q_ser <= q_allow { "≤" } else { ">" }, q_allow), "Concentric load: uniform pressure assumed.");
    let q_u = n_ed / a_ftg; // kN/m2 = kPa
    t.step(s2, "Net ULS ground pressure", "", "q_{u} = N_{Ed} / (B·L)", &format!("= {} / {:.3}", n_ed, a_ftg), &format!("q_{{u}} = {:.1} kPa", q_u), "Footing self-weight does not cause bending (acts with the ground reaction).");

    let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    let d = h - cover - 1.5 * phi;
    if d <= 0.0 { return fail(vec!["Effective depth not positive".into()]); }
    t.step(s3, "Effective depth and design strengths", "EN 1992-1-1 cl. 3.1.6, 2.4.2.4", "d = h − c − 1.5φ  (lower of the two bar layers, used both ways);  f_{cd} = α_{cc}·f_{ck}/γ_c;  f_{yd} = f_{yk}/γ_s", &format!("d = {} − {} − 1.5 × {} = {:.1} mm;  f_cd = {:.2} × {} / {:.2};  f_yd = {} / {:.2}", h, cover, phi, d, ps.alpha_cc, fck, ps.gamma_c, fyk, ps.gamma_s), &format!("d = {:.1} mm;  f_{{cd}} = {:.2} MPa;  f_{{yd}} = {:.2} MPa", d, fcd, fyd), "");
    let ax = (bx - cx) / 2.0 / 1e3; let ay = (by - cy) / 2.0 / 1e3; // m
    let mx = q_u * ax * ax / 2.0; let my = q_u * ay * ay / 2.0; // kNm per m
    t.step(s3, "Design moments at column faces (per metre width)", "EN 1992-1-1 cl. 5.3.2 (cantilever from column face)", "a = (B − c)/2;   M_{Ed} = q_{u}·a^{2}/2", &format!("a_x = ({} − {})/2 = {:.3} m;  M_x = {:.1} × {:.3}^{{2}}/2;  a_y = ({} − {})/2 = {:.3} m;  M_y = {:.1} × {:.3}^{{2}}/2", bx, cx, ax, q_u, ax, by, cy, ay, q_u, ay), &format!("M_x = {:.1} kNm/m;  M_y = {:.1} kNm/m", mx, my), "The larger moment governs both directions.");
    let m_ed = mx.max(my);
    let b = 1000.0;
    let (as_req, _, _) = match flex_single(&mut t, s3, b, d, fck, fcd, fyd, m_ed * 1e6, " (governing direction)") { Ok(r) => r, Err(e) => return fail(vec![e]) };
    let fctm = 0.30 * fck.powf(2.0 / 3.0);
    let as_min = (0.26 * fctm / fyk * b * d).max(0.0013 * b * d);
    t.step(s3, "Minimum reinforcement", "EN 1992-1-1 cl. 9.2.1.1(1)", "A_{s,min} = max[0.26·(f_{ctm}/f_{yk})·b·d ; 0.0013·b·d]", &format!("f_ctm = {:.2} MPa;  = max[{:.0} ; {:.0}]", fctm, 0.26 * fctm / fyk * b * d, 0.0013 * b * d), &format!("A_{{s,min}} = {:.0} mm^{{2}}/m", as_min), "");
    let as_design = as_req.max(as_min);
    let area1 = PI * phi * phi / 4.0;
    let s_max = (3.0 * h).min(400.0);
    let spacing = if check { s_in } else {
        let s_calc = 1000.0 * area1 / as_design;
        let s = (s_calc.min(s_max) / 25.0).floor() * 25.0;
        t.step(s3, "Bar spacing (both directions)", "EN 1992-1-1 cl. 9.3.1.1(3)", "s ≤ 1000·A_{bar}/A_{s,design};   s_{max} = min(3h ; 400)", &format!("A_bar = {:.1} mm^{{2}};  s ≤ 1000 × {:.1}/{:.0} = {:.0} mm;  s_max = {:.0} mm", area1, area1, as_design, s_calc, s_max), &format!("Use T{} @ {:.0} mm c/c each way", phi, s), "Rounded down to a multiple of 25 mm.");
        s
    };
    if spacing < 50.0 { return fail(vec!["Required spacing below 50 mm: increase bar size or thickness".into()]); }
    let as_prov = 1000.0 * area1 / spacing;
    let xx = as_prov * fyd / (0.8 * b * fcd); let zz = d - 0.4 * xx; let mrd = as_prov * fyd * zz;
    t.step(s3, "Moment resistance of provided steel", "EN 1992-1-1 cl. 3.1.7, 6.1", "A_{s,prov} = 1000·A_{bar}/s;  x = A_{s,prov}·f_{yd}/(0.8·b·f_{cd});  M_{Rd} = A_{s,prov}·f_{yd}·(d − 0.4x)", &format!("A_s,prov = {:.0} mm^{{2}}/m;  x = {:.1} mm;  z = {:.1} mm", as_prov, xx, zz), &format!("M_{{Rd}} = {:.1} kNm/m;  x/d = {:.3}", mrd / 1e6, xx / d), "");
    let mut checks = vec![];
    checks.push(chk("Bearing pressure (SLS)", "EN 1997-1 6.5.2", format!("{:.1} kPa", q_ser), format!("{:.0} kPa", q_allow), q_ser / q_allow, q_ser <= q_allow));
    checks.push(chk("Flexural resistance", "EN 1992-1-1 6.1", format!("{:.1} kNm/m", m_ed), format!("{:.1} kNm/m", mrd / 1e6), m_ed * 1e6 / mrd, m_ed * 1e6 <= mrd && xx / d <= 0.45 + 1e-9));
    checks.push(chk("Minimum reinforcement", "EN 1992-1-1 9.2.1.1(1)", format!("{:.0} mm²/m", as_min), format!("{:.0} mm²/m", as_prov), as_min / as_prov, as_prov >= as_min));
    checks.push(chk("Bar spacing (max)", "EN 1992-1-1 9.3.1.1(3)", format!("{:.0} mm", spacing), format!("{:.0} mm max", s_max), spacing / s_max, spacing <= s_max + 1e-9));
    checks.push(chk("Bar spacing (clear)", "EN 1992-1-1 8.2(2)", format!("{:.0} mm min", phi.max(25.0)), format!("{:.0} mm clear", spacing - phi), phi.max(25.0) / (spacing - phi).max(0.1), spacing - phi >= phi.max(25.0)));

    // one-way shear at d from column face
    let vx = q_u * (ax - d / 1e3).max(0.0); let vy = q_u * (ay - d / 1e3).max(0.0);
    let v_ed = vx.max(vy);
    t.step(s4, "Design shear at distance d from the column face (per metre width)", "EN 1992-1-1 cl. 6.2.1(8), 6.2.2", "V_{Ed} = q_{u}·(a − d)", &format!("V_x = {:.1} × ({:.3} − {:.3}) = {:.1} kN/m;  V_y = {:.1} × ({:.3} − {:.3}) = {:.1} kN/m", q_u, ax, d / 1e3, vx, q_u, ay, d / 1e3, vy), &format!("V_{{Ed}} = {:.1} kN/m", v_ed), "Enhancement for loads close to supports (β) is conservatively ignored.");
    let (vrdc, _, rho) = vrd_c(&mut t, s4, b, d, fck, ps.gamma_c, as_prov, "");
    checks.push(chk("One-way shear", "EN 1992-1-1 6.2.2(1)", format!("{:.1} kN/m", v_ed), format!("{:.1} kN/m", vrdc / 1e3), v_ed * 1e3 / vrdc, v_ed * 1e3 <= vrdc));

    // punching
    let nu = 0.6 * (1.0 - fck / 250.0);
    let u0 = 2.0 * (cx + cy);
    let ved0 = n_ed * 1e3 / (u0 * d);
    let vmax = 0.5 * nu * fcd;
    t.step(s5, "Punching: maximum resistance at the column face", "EN 1992-1-1 cl. 6.4.5(3), Eq 6.53", "u_0 = 2(c_x + c_y);  v_{Ed,0} = β·N_{Ed}/(u_0·d) with β = 1.0;  v_{Rd,max} = 0.5·ν·f_{cd},  ν = 0.6(1 − f_{ck}/250)", &format!("u_0 = 2 × ({} + {}) = {:.0} mm;  v_Ed,0 = {} × 10^{{3}} / ({:.0} × {:.1}) ;  ν = {:.3}", cx, cy, u0, n_ed, u0, d, nu), &format!("v_{{Ed,0}} = {:.3} MPa;  v_{{Rd,max}} = {:.3} MPa", ved0, vmax), "β = 1.0 for concentric load.");
    let u1 = 2.0 * (cx + cy) + 4.0 * PI * d;
    let a_in = ((cx * cy) + 4.0 * d * (cx + cy) + PI * (2.0 * d) * (2.0 * d)) / 1e6;
    let v_red = (n_ed - q_u * a_in.min(a_ftg)).max(0.0);
    let ved1 = v_red * 1e3 / (u1 * d);
    let k = (1.0 + (200.0 / d).sqrt()).min(2.0);
    let vmin = 0.035 * k.powf(1.5) * fck.sqrt();
    let vrd_p = (0.18 / ps.gamma_c * k * (100.0 * rho * fck).powf(1.0 / 3.0)).max(vmin);
    t.step(s5, "Punching: control perimeter at 2d", "EN 1992-1-1 cl. 6.4.2, 6.4.3, 6.4.4(1)", "u_1 = 2(c_x + c_y) + 2π·(2d);   V_{Ed,red} = N_{Ed} − q_{u}·A_{in};   v_{Ed} = V_{Ed,red}/(u_1·d);   v_{Rd,c} = max[C_{Rd,c}·k·(100ρ_l f_{ck})^{1/3} ; v_{min}]",
        &format!("u_1 = 2 × ({} + {}) + 4π × {:.1} = {:.0} mm;  A_in = {:.3} m^{{2}};  V_Ed,red = {} − {:.1} × {:.3} = {:.1} kN;  v_Ed = {:.1}×10^{{3}} / ({:.0} × {:.1});  k = {:.3};  ρ_l = {:.5}", cx, cy, d, u1, a_in, n_ed, q_u, a_in, v_red, v_red, u1, d, k, rho), &format!("v_{{Ed}} = {:.3} MPa;  v_{{Rd,c}} = {:.3} MPa", ved1, vrd_p), "The enhancement 2d/a for punching near the column is conservatively ignored.");
    checks.push(chk("Punching: column face", "EN 1992-1-1 6.4.5(3)", format!("{:.3} MPa", ved0), format!("{:.3} MPa", vmax), ved0 / vmax, ved0 <= vmax));
    checks.push(chk("Punching: perimeter at 2d", "EN 1992-1-1 6.4.4(1)", format!("{:.3} MPa", ved1), format!("{:.3} MPa", vrd_p), ved1 / vrd_p, ved1 <= vrd_p));

    let summary = vec![kv("Footing plan size", format!("{:.0} × {:.0} mm", bx, by)), kv("Thickness / depth d", format!("{:.0} / {:.1} mm", h, d)), kv("Service pressure", format!("{:.1} kPa (allow {:.0})", q_ser, q_allow)), kv("Net ULS pressure q u", format!("{:.1} kPa", q_u)), kv("Governing moment", format!("{:.1} kNm/m", m_ed)), kv("Reinforcement (each way)", format!("T{} @ {:.0}  ({:.0} mm²/m)", phi, spacing, as_prov)), kv("Footing self-weight", format!("{:.1} kN", g))];
    finish(t, checks, &ps.status, summary, vec![], vec![("INFO", "Sliding, overturning and settlement are not applicable to concentric vertical load and are not assessed. Allowable bearing pressure must come from a geotechnical report.".to_string())], vec![
        "Implemented: bearing (allowable pressure), flexure, minimum steel, bar spacing, one-way shear, punching at the column face and at 2d.".into(),
        "NOT implemented: eccentric loads and moments, ULS geotechnical bearing resistance (EN 1997-1), settlement, sliding, overturning, anchorage, strip/combined footings.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Pad { bx, by, cx, cy, h })
}
