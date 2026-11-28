//! Simplified rigid raft: conventional rigid method (statics) on one column-line strip, EN 1992-1-1:2004.
//! Equal column loads, symmetric about the raft centre, so the soil pressure is uniform. One direction only.
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

pub fn def() -> ModDef {
    ModDef { id: "raft", name: "Raft foundation", title: "Reinforced concrete raft foundation (simplified rigid method)", subtitle: "EN 1992-1-1:2004 · One column-line strip, equal symmetric column loads, uniform ground pressure, allowable bearing pressure supplied", available: true, has_check_mode: false, fields: vec![
        fld("n_cols", "Number of columns in the line", "-", "Layout", 3.0), fld("spacing", "Column spacing s", "mm", "Layout", 5000.0), fld("edge", "Overhang from end column to raft edge", "mm", "Layout", 1000.0), fld("width", "Strip width w (tributary width of the line)", "mm", "Layout", 5000.0),
        fld("n_ed", "Load per column N Ed (ULS)", "kN", "Actions", 1500.0), fld("n_k", "Load per column N k (SLS)", "kN", "Actions", 1100.0), fld("q_allow", "Allowable bearing pressure", "kPa", "Ground", 150.0),
        fld("cx", "Column size c x (along the line)", "mm", "Columns", 400.0), fld("cy", "Column size c y", "mm", "Columns", 400.0),
        fld("h", "Raft thickness h", "mm", "Raft", 800.0), fld("cover", "Cover c (top and bottom)", "mm", "Raft", 50.0), fld("bar_dia", "Bar φ", "mm", "Raft", 20.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, false) { Ok(x) => x, Err(e) => return fail(e) };
    let (nf, s, edge, w, n_ed, n_k, q_allow, cx, cy, h, cover, phi, fck, fyk) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11], x[12], x[13]);
    let mut errs = vec![];
    if nf.fract() != 0.0 || !(2.0..=12.0).contains(&nf) { errs.push("Number of columns must be a whole number from 2 to 12".to_string()); }
    if !(12.0..=50.0).contains(&fck) { errs.push("fck must be within 12-50 MPa for this release".to_string()); }
    if edge < cx / 2.0 { errs.push("Overhang must be at least half the column size".to_string()); }
    if !errs.is_empty() { return fail(errs); }
    let n = nf as usize;
    let l = ((nf - 1.0) * s + 2.0 * edge) / 1e3; let wm = w / 1e3; let area = l * wm;
    let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = ("1. Design data", "2. Bearing pressure", "3. Strip analysis (rigid method)", "4. Flexural and shear design", "5. Punching shear");
    t.step(s1, "Layout, loads and ground", "", "", &format!("{} columns at s = {} mm;  overhang = {} mm;  strip width w = {} mm;  raft length L = {:.2} m;  N_{{Ed}} = {} kN;  N_{{k}} = {} kN per column;  q_{{allow}} = {} kPa", n, s, edge, w, l, n_ed, n_k, q_allow), "", "Equal column loads placed symmetrically: the resultant coincides with the raft centroid, so a uniform ground pressure is a valid assumption. q_allow is supplied by the geotechnical engineer.");
    t.step(s1, "Section and materials", "", "", &format!("Columns {} × {} mm;  h = {} mm;  cover c = {} mm;  bar φ = {} mm;  f_{{ck}} = {} MPa;  f_{{yk}} = {} MPa", cx, cy, h, cover, phi, fck, fyk), "", "Bars in two layers; d is measured to the outer layer (top for hogging, bottom for sagging).");
    t.step(s1, "Partial factors", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6", "", &format!("γ_{{c}} = {:.2};  γ_{{s}} = {:.2};  α_{{cc}} = {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc), "", &format!("Parameter set: {} [{}]", ps.label, ps.status));
    let g = 25.0 * area * (h / 1e3);
    let q_ser = (nf * n_k + g) / area;
    t.step(s2, "Service bearing pressure", "EN 1997-1 cl. 6.5.2", "q_{ser} = (n·N_k + G_{raft}) / (L·w);   G_{raft} = 25·L·w·h", &format!("G_raft = 25 × {:.2} × {:.2} × {:.3} = {:.0} kN;  q_ser = ({} × {} + {:.0}) / {:.2}", l, wm, h / 1e3, g, n, n_k, g, area), &format!("q_{{ser}} = {:.1} kPa  {} q_{{allow}} = {} kPa", q_ser, if q_ser <= q_allow { "≤" } else { ">" }, q_allow), "Uniform pressure under symmetric, equal column loads.");
    let q_u = nf * n_ed / area;
    t.step(s2, "Net ULS ground pressure", "", "q_u = n·N_{Ed} / (L·w)", &format!("= {} × {} / {:.2}", n, n_ed, area), &format!("q_{{u}} = {:.1} kPa", q_u), "Self-weight of the raft does not cause bending (it is carried directly by the ground).");
    // strip analysis: p = q_u*w kN/m up; columns N_ed down at x_i
    let p = q_u * wm;
    let xs: Vec<f64> = (0..n).map(|i| edge / 1e3 + i as f64 * s / 1e3).collect();
    let steps = 4000; let dx = l / steps as f64;
    let (mut m_sag, mut m_hog, mut x_sag, mut x_hog) = (0.0f64, 0.0f64, 0.0, 0.0);
    let m_at = |xx: f64| -> f64 { p * xx * xx / 2.0 - xs.iter().filter(|&&xi| xi < xx).map(|&xi| n_ed * (xx - xi)).sum::<f64>() };
    let v_at = |xx: f64| -> f64 { p * xx - xs.iter().filter(|&&xi| xi < xx).map(|_| n_ed).sum::<f64>() };
    for i in 0..=steps { let xx = i as f64 * dx; let m = m_at(xx); if m > m_sag { m_sag = m; x_sag = xx; } if -m > m_hog { m_hog = -m; x_hog = xx; } }
    t.step(s3, "Strip statics", "Equilibrium of the strip (conventional rigid method)", "p = q_u·w (upward);   V(x) = p·x − Σ N_{Ed,i}  (columns left of x);   M(x) = p·x²/2 − Σ N_{Ed,i}·(x − x_i);   M > 0: tension at the bottom", &format!("p = {:.1} × {:.2} = {:.1} kN/m;  column positions x_i = {} m", q_u, wm, p, xs.iter().map(|x| format!("{:.2}", x)).collect::<Vec<_>>().join(", ")), &format!("M_{{sag}} = {:.0} kNm at x = {:.2} m (bottom tension);  M_{{hog}} = {:.0} kNm at x = {:.2} m (top tension)", m_sag, x_sag, m_hog, x_hog), "Peak moments are taken at the column centre lines (no reduction to the column face): conservative.");
    // shear at d from column faces
    let d_bot = h - cover - phi / 2.0; let dm = d_bot / 1e3;
    let mut v_max = 0.0f64;
    for &xi in &xs { for sgn in [-1.0, 1.0] { let xx = xi + sgn * (cx / 2000.0 + dm); if xx > 0.0 && xx < l { v_max = v_max.max(v_at(xx).abs()); } } }
    t.step(s3, "Design shear at distance d from the column faces", "EN 1992-1-1 cl. 6.2.1(8)", "V_{Ed} = max |V(x)| at x_i ± (c_x/2 + d)", &format!("d = {:.1} mm", d_bot), &format!("V_{{Ed}} = {:.0} kN on the strip = {:.1} kN/m", v_max, v_max / wm), "");
    let mut checks: Vec<crate::rc_beam::Check> = vec![];
    checks.push(chk("Bearing pressure (SLS)", "EN 1997-1 6.5.2", format!("{:.1} kPa", q_ser), format!("{:.0} kPa", q_allow), q_ser / q_allow, q_ser <= q_allow));
    // member design per metre width
    let m_bot = m_sag / wm; let m_top = m_hog / wm; let v_m = v_max / wm;
    let (as_b, s_b, _) = match slab_member(&mut t, s4, "Bottom steel (at columns)", m_bot, v_m, h, cover, phi, fck, fcd, fyd, fyk, ps.gamma_c, &mut checks) { Ok(r) => r, Err(e) => return fail(vec![e]) };
    let (as_t, s_t, _) = match slab_member(&mut t, s4, "Top steel (between columns)", m_top, v_m, h, cover, phi, fck, fcd, fyd, fyk, ps.gamma_c, &mut checks) { Ok(r) => r, Err(e) => return fail(vec![e]) };
    // punching at interior columns
    let nu = 0.6 * (1.0 - fck / 250.0);
    let rho = (as_b / (1000.0 * d_bot)).min(0.02).max(as_t / (1000.0 * d_bot)).min(0.02);
    let mut interior = false;
    if n >= 3 {
        interior = true;
        let u0 = 2.0 * (cx + cy); let beta = 1.15;
        let ved0 = beta * n_ed * 1e3 / (u0 * d_bot); let vmax = 0.5 * nu * fcd;
        t.step(s5, "Interior column: maximum punching resistance at the column face", "EN 1992-1-1 cl. 6.4.5(3), 6.4.3(6)", "u_0 = 2(c_x + c_y);   β = 1.15 (interior column);   v_{Ed,0} = β·N_{Ed}/(u_0·d);   v_{Rd,max} = 0.5·ν·f_{cd}", &format!("u_0 = {:.0} mm;  v_Ed,0 = 1.15 × {}×10^{{3}} / ({:.0} × {:.1});  ν = {:.3}", u0, n_ed, u0, d_bot, nu), &format!("v_{{Ed,0}} = {:.3} MPa;  v_{{Rd,max}} = {:.3} MPa", ved0, vmax), "");
        let u1 = 2.0 * (cx + cy) + 4.0 * PI * d_bot;
        let a_in = (cx * cy + 4.0 * d_bot * (cx + cy) + PI * (2.0 * d_bot).powi(2)) / 1e6;
        let v_red = (n_ed - q_u * a_in).max(0.0);
        let ved1 = 1.15 * v_red * 1e3 / (u1 * d_bot);
        let k = (1.0 + (200.0 / d_bot).sqrt()).min(2.0); let vmin = 0.035 * k.powf(1.5) * fck.sqrt();
        let vrdc = (0.18 / ps.gamma_c * k * (100.0 * rho * fck).powf(1.0 / 3.0)).max(vmin);
        t.step(s5, "Interior column: control perimeter at 2d", "EN 1992-1-1 cl. 6.4.2, 6.4.3, 6.4.4(1)", "u_1 = 2(c_x + c_y) + 2π·(2d);   V_{Ed,red} = N_{Ed} − q_u·A_{in};   v_{Ed} = β·V_{Ed,red}/(u_1·d);   v_{Rd,c} = max[C_{Rd,c}·k·(100ρ_l·f_{ck})^{1/3} ; v_{min}]", &format!("u_1 = {:.0} mm;  A_in = {:.3} m^{{2}};  V_Ed,red = {} − {:.1} × {:.3} = {:.1} kN;  v_Ed = 1.15 × {:.1}×10^{{3}}/({:.0} × {:.1});  k = {:.3};  ρ_l = {:.5}", u1, a_in, n_ed, q_u, a_in, v_red, v_red, u1, d_bot, k, rho), &format!("v_{{Ed}} = {:.3} MPa;  v_{{Rd,c}} = {:.3} MPa", ved1, vrdc), "Punching reinforcement is not designed. The enhancement 2d/a is ignored (conservative).");
        checks.push(chk("Punching, interior column face", "EN 1992-1-1 6.4.5(3)", format!("{:.3} MPa", ved0), format!("{:.3} MPa", vmax), ved0 / vmax, ved0 <= vmax));
        checks.push(chk("Punching, interior column at 2d", "EN 1992-1-1 6.4.4(1)", format!("{:.3} MPa", ved1), format!("{:.3} MPa", vrdc), ved1 / vrdc, ved1 <= vrdc));
    }
    let mut obs: Vec<(&str, String)> = vec![("AMBER", "The rigid-method assumption (rigid raft, uniform pressure) is not verified. For flexible rafts or uneven loads use a soil-structure interaction analysis.".to_string()), ("AMBER", "Only the strip along the column line is designed. The perpendicular direction must be designed separately and bar layers arranged accordingly.".to_string())];
    if !interior { obs.push(("AMBER", "Fewer than three columns: no interior column, so punching is not checked.".to_string())); }
    obs.push(("AMBER", "End (edge) columns: punching with the reduced perimeter near the free edge (β = 1.4, EN 1992-1-1 6.4.3) is not checked. Verify the end columns separately.".to_string()));
    obs.push(("INFO", "Settlement and differential settlement are not assessed. The allowable bearing pressure must come from a geotechnical report.".to_string()));
    let summary = vec![kv("Raft size L × w", format!("{:.2} m × {:.2} m", l, wm)), kv("Service / ULS pressure", format!("{:.1} / {:.1} kPa", q_ser, q_u)), kv("Max sagging moment (bottom)", format!("{:.0} kNm on strip ({:.1} kNm/m)", m_sag, m_bot)), kv("Max hogging moment (top)", format!("{:.0} kNm on strip ({:.1} kNm/m)", m_hog, m_top)), kv("Bottom steel", format!("T{} @ {:.0} ({:.0} mm²/m)", phi, s_b, as_b)), kv("Top steel", format!("T{} @ {:.0} ({:.0} mm²/m)", phi, s_t, as_t)), kv("Design shear", format!("{:.1} kN/m", v_m))];
    finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: service bearing pressure, conventional rigid-method strip statics, flexure (top and bottom), shear at d, punching at interior columns, reinforcement design.".into(),
        "NOT implemented: flexible raft analysis, unequal or eccentric loads, edge and corner column punching, the perpendicular direction, settlement and differential settlement, crack control.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Raft { l: l * 1000.0, w, n: n as u32, spacing: s, edge })
}
