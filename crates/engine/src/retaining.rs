//! Cantilever RC retaining wall (EN 1997-1 Design Approach 1 recommended factors; EN 1992-1-1 structural design). Per metre run.
//! Rankine active pressure on a vertical plane through the heel, horizontal backfill. Passive resistance in front of the toe is ignored.
use crate::module::*;
use crate::params;
use crate::rc_beam::Check;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

const GW: f64 = 9.81;

pub fn def() -> ModDef {
    ModDef { id: "retaining", name: "Retaining wall", title: "Reinforced concrete cantilever retaining wall", subtitle: "EN 1997-1 / EN 1992-1-1 · Rankine active pressure, horizontal backfill, per metre run", available: true, has_check_mode: false, fields: vec![
        fld("hs", "Stem height H s", "mm", "Wall geometry", 3000.0), fld("ts", "Stem thickness t s (constant)", "mm", "Wall geometry", 350.0), fld("tb", "Base thickness t b", "mm", "Wall geometry", 400.0), fld("lt", "Toe length L t", "mm", "Wall geometry", 800.0), fld("bw", "Total base width B", "mm", "Wall geometry", 3200.0),
        fld("gamma", "Backfill unit weight γ", "kN/m³", "Soil", 18.0), fld("phi", "Backfill friction angle φ'", "deg", "Soil", 30.0), fld("delta", "Base friction angle δ", "deg", "Soil", 30.0), fld("q_allow", "Allowable bearing pressure (service)", "kPa", "Soil", 200.0),
        fld("q_sur", "Surcharge on backfill q (variable)", "kPa", "Actions", 10.0), fld("hw", "Water height behind wall above base underside (0 = fully drained)", "mm", "Actions", 0.0),
        fld("cs", "Cover, stem", "mm", "Reinforcement", 40.0), fld("cb", "Cover, base", "mm", "Reinforcement", 50.0), fld("ps", "Stem bar φ", "mm", "Reinforcement", 16.0), fld("pb", "Base bar φ", "mm", "Reinforcement", 16.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0),
    ] }
}

struct Ev { pa: f64, pq: f64, pw: f64, hd: f64, m_dst: f64, v: f64, m_stb: f64 }

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, false) { Ok(x) => x, Err(e) => return fail(e) };
    let (hs, ts, tb, lt, bw, gam, phi, delta, q_allow, qs, hw, cs, cb, phs, phb, fck, fyk) = (x[0] / 1e3, x[1] / 1e3, x[2] / 1e3, x[3] / 1e3, x[4] / 1e3, x[5], x[6], x[7], x[8], x[9], x[10] / 1e3, x[11], x[12], x[13], x[14], x[15], x[16]);
    let mut errs = vec![];
    if lt + ts >= bw { errs.push("Base width must exceed toe length plus stem thickness (a heel is required)".to_string()); }
    if !(20.0..=45.0).contains(&phi) { errs.push("Backfill φ' must be between 20° and 45°".to_string()); }
    if delta > phi + 1e-9 { errs.push("Base friction angle δ must not exceed φ'".to_string()); }
    if hw > hs + tb { errs.push("Water height cannot exceed the wall height".to_string()); }
    if fck > 50.0 || fck < 12.0 { errs.push("fck must be within 12-50 MPa for this release".to_string()); }
    if !errs.is_empty() { return fail(errs); }
    let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5, s6, s7) = ("1. Design data", "2. Characteristic forces", "3. Stability: sliding and overturning", "4. Base pressure", "5. Stem design", "6. Toe design", "7. Heel design");
    let h = hs + tb; let hh = hs - 0.0;
    let _ = hh;
    t.step(s1, "Geometry", "", "", &format!("H_s = {} m;  t_s = {} m;  t_b = {} m;  L_t = {} m;  B = {} m;  total height H = H_s + t_b = {:.2} m;  heel length = {:.2} m", hs, ts, tb, lt, bw, h, bw - lt - ts), "", "Constant stem thickness, horizontal ground in front and behind, level backfill.");
    t.step(s1, "Soil, water and actions", "", "", &format!("γ = {} kN/m³;  φ' = {}°;  δ = {}°;  q_allow = {} kPa;  surcharge q = {} kPa;  water height h_w = {} m", gam, phi, delta, q_allow, qs, hw), "", if hw <= 0.0 { "Backfill is assumed FULLY DRAINED (no water pressure). This depends on the drainage detail being built and maintained: confirm on site." } else { "Hydrostatic pressure from water behind the wall is included. Seepage and soil buoyancy are not modelled." });
    t.step(s1, "Partial factors", "EN 1997-1 Annex A (recommended values); EN 1990 Table A1.2; EN 1992-1-1 cl. 2.4.2.4", "", &format!("DA1 Combination 1 (A1+M1): γ_G = 1.35 unfavourable, 1.0 favourable;  γ_Q = 1.5;  γ_φ = 1.0.   Combination 2 (A2+M2): γ_G = 1.0;  γ_Q = 1.3;  γ_φ = 1.25.   EQU: γ_G,dst = 1.1;  γ_G,stb = 0.9;  γ_Q = 1.5;  γ_φ = 1.25.   Structural: γ_G = {:.2};  γ_Q = {:.2};  γ_c = {:.2};  γ_s = {:.2}", ps.gamma_g, ps.gamma_q, ps.gamma_c, ps.gamma_s), "", &format!("Parameter set: {} [{}]. The geotechnical factors are the EN recommended values; the National Annex may choose a different Design Approach.", ps.label, ps.status));

    // weights (kN/m) and arms from toe
    let heel = bw - lt - ts;
    let w_stem = ts * hs * 25.0; let x_stem = lt + ts / 2.0;
    let w_base = bw * tb * 25.0; let x_base = bw / 2.0;
    let w_soil = heel * hs * gam; let x_soil = lt + ts + heel / 2.0;
    let wsum = w_stem + w_base + w_soil;
    let m_w = w_stem * x_stem + w_base * x_base + w_soil * x_soil;
    t.step(s2, "Self-weight of wall and soil on the heel (characteristic)", "EN 1991-1-1 Table A.1 (concrete 25 kN/m³)", "W_{stem} = t_s·H_s·25;   W_{base} = B·t_b·25;   W_{soil} = L_h·H_s·γ;   moments about the toe", &format!("W_stem = {} × {} × 25 = {:.1} kN/m at {:.3} m;  W_base = {} × {} × 25 = {:.1} kN/m at {:.3} m;  W_soil = {:.2} × {} × {} = {:.1} kN/m at {:.3} m", ts, hs, w_stem, x_stem, bw, tb, w_base, x_base, heel, hs, gam, w_soil, x_soil), &format!("ΣW = {:.1} kN/m;  ΣM_W = {:.1} kNm/m", wsum, m_w), "Soil in front of the toe and the vertical surcharge on the heel are ignored (conservative).");

    let ka_of = |ph: f64| { let s = ph.to_radians().sin(); (1.0 - s) / (1.0 + s) };
    let phi_d = |g: f64| (phi.to_radians().tan() / g).atan().to_degrees();
    let ev = |gd: f64, gs: f64, gq: f64, gphi: f64| -> Ev {
        let ka = ka_of(phi_d(gphi));
        let pa = gd * 0.5 * ka * gam * h * h; let pq = gq * ka * qs * h; let pw = gd * 0.5 * GW * hw * hw;
        let m_dst = pa * h / 3.0 + pq * h / 2.0 + pw * hw / 3.0;
        Ev { pa, pq, pw, hd: pa + pq + pw, m_dst, v: gs * wsum, m_stb: gs * m_w }
    };
    let ka_k = ka_of(phi);
    t.step(s2, "Rankine active earth pressure coefficient", "EN 1997-1 Annex C.1 / Rankine theory", "K_a = (1 − sin φ')/(1 + sin φ');   P_a = ½·K_a·γ·H^{2};   P_q = K_a·q·H;   P_w = ½·γ_w·h_w^{2}", &format!("K_a = (1 − sin {}°)/(1 + sin {}°) = {:.4};  P_a = 0.5 × {:.4} × {} × {:.2}^{{2}} = {:.2} kN/m;  P_q = {:.4} × {} × {:.2} = {:.2} kN/m;  P_w = 0.5 × 9.81 × {}^{{2}} = {:.2} kN/m", phi, phi, ka_k, ka_k, gam, h, 0.5 * ka_k * gam * h * h, ka_k, qs, h, ka_k * qs * h, hw, 0.5 * GW * hw * hw), &format!("K_{{a}} = {:.4}", ka_k), "Wall friction ignored (conservative). The earth pressure acts on a vertical plane through the end of the heel; the soil above the heel is treated as part of the wall.");

    // sliding
    let mut checks = vec![];
    for (name, gd, gq, gphi) in [("Combination 1 (A1+M1+R1)", 1.35, 1.5, 1.0), ("Combination 2 (A2+M2+R1)", 1.0, 1.3, 1.25)] {
        let e = ev(gd, 1.0, gq, gphi);
        let tand = delta.to_radians().tan() / gphi;
        let rd = e.v * tand;
        t.step(s3, &format!("Sliding, {}", name), "EN 1997-1 cl. 9.7.4, 6.5.3", "H_d = γ_G·P_a + γ_Q·P_q + γ_G·P_w;   R_d = V_d·tan δ_d,   tan δ_d = tan δ/γ_φ   (R;h = 1.0)", &format!("K_a(φ_d) = {:.4};  H_d = {:.2} + {:.2} + {:.2} = {:.2} kN/m;  V_d = {:.1} kN/m;  tan δ_d = {:.4};  R_d = {:.1} × {:.4}", ka_of(phi_d(gphi)), e.pa, e.pq, e.pw, e.hd, e.v, tand, e.v, tand), &format!("H_d = {:.1} kN/m  {} R_d = {:.1} kN/m", e.hd, if e.hd <= rd { "≤" } else { ">" }, rd), "Passive resistance in front of the toe and the base key are ignored.");
        checks.push(chk(&format!("Sliding, {}", if gphi == 1.0 { "DA1-1" } else { "DA1-2" }), "EN 1997-1 9.7.4", format!("{:.1} kN/m", e.hd), format!("{:.1} kN/m", rd), e.hd / rd, e.hd <= rd));
    }
    // overturning EQU
    let e = ev(1.1, 0.9, 1.5, 1.25);
    t.step(s3, "Overturning about the toe (EQU)", "EN 1997-1 cl. 2.4.7.2, 9.7.5; Table A.1", "M_{dst,d} = 1.1·(P_a·H/3 + P_w·h_w/3) + 1.5·P_q·H/2;   M_{stb,d} = 0.9·ΣM_W;   require M_{dst,d} ≤ M_{stb,d}", &format!("K_a(φ_d, γ_φ = 1.25) = {:.4};  M_dst,d = {:.1} kNm/m;  M_stb,d = 0.9 × {:.1} = {:.1} kNm/m", ka_of(phi_d(1.25)), e.m_dst, m_w, e.m_stb), &format!("M_{{dst,d}} = {:.1} kNm/m  {} M_{{stb,d}} = {:.1} kNm/m", e.m_dst, if e.m_dst <= e.m_stb { "≤" } else { ">" }, e.m_stb), "");
    checks.push(chk("Overturning (EQU)", "EN 1997-1 9.7.5", format!("{:.1} kNm/m", e.m_dst), format!("{:.1} kNm/m", e.m_stb), e.m_dst / e.m_stb, e.m_dst <= e.m_stb));

    // base pressure, generic: returns (p(x) closure params)
    let pressure = |ev: &Ev| -> Option<(f64, f64, f64, f64)> { // (xbar, p_toe, p_heel, kind) kind=0 linear,1 triangular
        let xbar = (ev.m_stb - ev.m_dst) / ev.v; if xbar <= 0.0 { return None; }
        if xbar >= bw / 3.0 { let e = bw / 2.0 - xbar; Some((xbar, ev.v / bw * (1.0 + 6.0 * e / bw), ev.v / bw * (1.0 - 6.0 * e / bw), 0.0)) }
        else { Some((xbar, 2.0 * ev.v / (3.0 * xbar), 0.0, 1.0)) }
    };
    let p_at = |pr: &(f64, f64, f64, f64), xx: f64| -> f64 { if pr.3 == 0.0 { pr.1 + (pr.2 - pr.1) * xx / bw } else { (pr.1 * (1.0 - xx / (3.0 * pr.0))).max(0.0) } };
    // characteristic bearing
    let ek = ev(1.0, 1.0, 1.0, 1.0);
    let prk = match pressure(&ek) { Some(p) => p, None => return fail(vec!["The resultant lies outside the base (overturning under characteristic loads). Increase the base width or heel length.".into()]) };
    let e_ecc = bw / 2.0 - prk.0;
    t.step(s4, "Base pressure under characteristic loads", "EN 1997-1 cl. 6.5.2 (allowable pressure approach)", "x̄ = (ΣM_W − M_dst)/ΣV;   e = B/2 − x̄;   if e ≤ B/6: q = (V/B)(1 ± 6e/B),  else triangular q_max = 2V/(3·x̄)", &format!("ΣV = {:.1} kN/m;  M_dst,k = {:.1} kNm/m;  x̄ = ({:.1} − {:.1})/{:.1} = {:.3} m;  e = {:.3} m;  B/6 = {:.3} m", ek.v, ek.m_dst, m_w, ek.m_dst, ek.v, prk.0, e_ecc, bw / 6.0), &format!("q_{{max}} = {:.1} kPa  {} q_{{allow}} = {} kPa;  q_{{min}} = {:.1} kPa", prk.1, if prk.1 <= q_allow { "≤" } else { ">" }, q_allow, prk.2), if e_ecc <= bw / 6.0 { "Resultant within the middle third." } else { "Resultant outside the middle third: partial contact (triangular pressure)." });
    checks.push(chk("Bearing pressure", "EN 1997-1 6.5.2", format!("{:.1} kPa", prk.1), format!("{:.0} kPa", q_allow), prk.1 / q_allow, prk.1 <= q_allow));

    // ULS structural (STR): 1.35 permanent, 1.5 variable, characteristic phi
    let (gp, gqv) = (ps.gamma_g, ps.gamma_q);
    let eu = ev(gp, gp, gqv, 1.0);
    let pru = match pressure(&eu) { Some(p) => p, None => return fail(vec!["The ULS resultant lies outside the base: increase the base width.".into()]) };
    t.step(s4, "Base pressure under ULS structural loads", "EN 1990 Eq 6.10", "All permanent actions × γ_G, surcharge × γ_Q; ground reaction from moment equilibrium about the toe", &format!("ΣV_u = {:.1} kN/m;  M_dst,u = {:.1} kNm/m;  M_stb,u = {:.1} kNm/m;  x̄ = {:.3} m", eu.v, eu.m_dst, eu.m_stb, pru.0), &format!("p_{{toe}} = {:.1} kPa;  p_{{heel}} = {:.1} kPa", p_at(&pru, 0.0), p_at(&pru, bw)), "Used for the flexure and shear of the toe and heel slabs.");
    // helper: reinforcement design for a member
    let fctm = 0.30 * fck.powf(2.0 / 3.0);
    let design = |t: &mut Trace, sec: &str, member: &str, m_knm: f64, v_kn: f64, thick_m: f64, cover: f64, bar: f64, checks: &mut Vec<Check>| -> Result<(f64, f64, f64), String> {
        let hmm = thick_m * 1000.0; let d = hmm - cover - bar / 2.0; let b = 1000.0;
        let (as_req, _, _) = flex_single(t, sec, b, d, fck, fcd, fyd, m_knm.max(0.0) * 1e6, &format!(" ({})", member))?;
        let as_min = (0.26 * fctm / fyk * b * d).max(0.0013 * b * d);
        let as_des = as_req.max(as_min); let a1 = PI * bar * bar / 4.0; let s_max = (3.0 * hmm).min(400.0);
        let s = ((1000.0 * a1 / as_des).min(s_max) / 25.0).floor() * 25.0;
        if s < 50.0 { return Err(format!("{}: required bar spacing below 50 mm - increase bar size or thickness", member)); }
        let as_prov = 1000.0 * a1 / s;
        let xx = as_prov * fyd / (0.8 * b * fcd); let mrd = as_prov * fyd * (d - 0.4 * xx);
        t.step(sec, &format!("Reinforcement ({})", member), "EN 1992-1-1 cl. 9.2.1.1, 9.3.1.1(3)", "A_{s,min} = max[0.26·(f_{ctm}/f_{yk})·b·d ; 0.0013·b·d];   s ≤ 1000·A_{bar}/A_{s,design};  s ≤ min(3h ; 400);   M_{Rd} = A_{s,prov}·f_{yd}·(d − 0.4x)", &format!("d = {:.1} mm;  A_s,min = {:.0};  A_s,req = {:.0};  A_bar = {:.1} mm^{{2}};  s = {:.0} mm;  A_s,prov = {:.0} mm^{{2}}/m;  x = {:.1} mm", d, as_min, as_req, a1, s, as_prov, xx), &format!("T{} @ {:.0} mm c/c;  M_{{Rd}} = {:.1} kNm/m", bar, s, mrd / 1e6), "");
        checks.push(chk(&format!("{}: flexure", member), "EN 1992-1-1 6.1", format!("{:.1} kNm/m", m_knm.max(0.0)), format!("{:.1} kNm/m", mrd / 1e6), m_knm.max(0.0) * 1e6 / mrd, m_knm.max(0.0) * 1e6 <= mrd && xx / d <= 0.45 + 1e-9));
        let (vrdc, _, _) = vrd_c(t, sec, b, d, fck, ps.gamma_c, as_prov, &format!(" ({})", member));
        checks.push(chk(&format!("{}: shear", member), "EN 1992-1-1 6.2.2(1)", format!("{:.1} kN/m", v_kn), format!("{:.1} kN/m", vrdc / 1e3), v_kn * 1e3 / vrdc, v_kn * 1e3 <= vrdc));
        Ok((as_prov, s, d))
    };
    // stem: active pressure on stem height hs, STR factors
    let hwst = (hw - tb).max(0.0);
    let m_stem = gp * (ka_k * gam * hs.powi(3) / 6.0 + GW * hwst.powi(3) / 6.0) + gqv * ka_k * qs * hs * hs / 2.0;
    let v_stem = gp * (0.5 * ka_k * gam * hs * hs + 0.5 * GW * hwst * hwst) + gqv * ka_k * qs * hs;
    t.step(s5, "Design moment and shear at the base of the stem", "Statics; EN 1990 Eq 6.10", "M_{Ed} = γ_G·(K_a·γ·H_s^{3}/6 + γ_w·h_{ws}^{3}/6) + γ_Q·K_a·q·H_s^{2}/2;   V_{Ed} = γ_G·(½·K_a·γ·H_s^{2} + ½·γ_w·h_{ws}^{2}) + γ_Q·K_a·q·H_s", &format!("M_Ed = {:.2} × ({:.4} × {} × {}^{{3}}/6) + {:.2} × {:.4} × {} × {}^{{2}}/2 (+ water);  V_Ed = {:.2} kN/m (stem base)", gp, ka_k, gam, hs, gqv, ka_k, qs, hs, v_stem), &format!("M_{{Ed}} = {:.1} kNm/m;  V_{{Ed}} = {:.1} kN/m", m_stem, v_stem), "Reinforcement on the retained (tension) face. Shear at the stem base is used without reduction at d (conservative).");
    let (as_stem, s_stem, _) = match design(&mut t, s5, "Stem", m_stem, v_stem, ts, cs, phs, &mut checks) { Ok(r) => r, Err(e) => return fail(vec![e]) };
    // numeric integration helpers (x in m from toe)
    let integ = |f: &dyn Fn(f64) -> f64, a: f64, b: f64| -> f64 { let n = 400; let dx = (b - a) / n as f64; (0..n).map(|i| f(a + (i as f64 + 0.5) * dx) * dx).sum::<f64>() };
    // toe: critical section at the stem face (x = lt); upward pressure minus toe self-weight
    let w_toe = gp * tb * 25.0;
    let m_toe = integ(&|xx| (p_at(&pru, xx) - w_toe) * (lt - xx), 0.0, lt);
    let d_toe = tb * 1000.0 - cb - phb / 2.0;
    let dm = d_toe / 1000.0;
    let v_toe = integ(&|xx| p_at(&pru, xx) - w_toe, 0.0, (lt - dm).max(0.0));
    t.step(s6, "Toe: design moment and shear", "Statics (cantilever from the stem face)", "M_{Ed} = ∫ (p(x) − γ_G·t_b·25)·(L_t − x) dx over the toe;   V_{Ed} = ∫ (p(x) − γ_G·t_b·25) dx from x = 0 to L_t − d", &format!("Ground reaction from the ULS base pressure (integrated numerically); toe self-weight {:.2} kPa", w_toe), &format!("M_{{Ed}} = {:.1} kNm/m;  V_{{Ed}} = {:.1} kN/m", m_toe, v_toe), "Tension at the underside. Soil above the toe is ignored (conservative).");
    let (as_toe, s_toe, _) = match design(&mut t, s6, "Toe", m_toe, v_toe.max(0.0), tb, cb, phb, &mut checks) { Ok(r) => r, Err(e) => return fail(vec![e]) };
    // heel: downward load (soil + slab at gamma_G, surcharge at gamma_Q) minus upward pressure
    let w_down = gp * (hs * gam + tb * 25.0) + gqv * qs;
    let xh = lt + ts;
    let m_heel = integ(&|xx| (w_down - p_at(&pru, xx)) * (xx - xh), xh, bw);
    let v_heel = integ(&|xx| w_down - p_at(&pru, xx), (xh + dm).min(bw), bw);
    t.step(s7, "Heel: design moment and shear", "Statics (cantilever from the stem face)", "w_{down} = γ_G·(H_s·γ + t_b·25) + γ_Q·q;   M_{Ed} = ∫ (w_{down} − p(x))·(x − x_s) dx over the heel;   V_{Ed} at x_s + d", &format!("w_down = {:.2} × ({} × {} + {} × 25) + {:.2} × {} = {:.1} kPa;  ground reaction integrated numerically", gp, hs, gam, tb, gqv, qs, w_down), &format!("M_{{Ed}} = {:.1} kNm/m;  V_{{Ed}} = {:.1} kN/m", m_heel, v_heel), "Tension at the top face. The upward ground reaction relieves the heel, so the case with the surcharge acting and zero reaction should be considered by the engineer.");
    let (as_heel, s_heel, _) = match design(&mut t, s7, "Heel", m_heel, v_heel.max(0.0), tb, cb, phb, &mut checks) { Ok(r) => r, Err(e) => return fail(vec![e]) };
    let mut obs: Vec<(&str, String)> = vec![];
    if hw <= 0.0 { obs.push(("AMBER", "Drained backfill assumed: the design is valid only if a drainage layer and weep holes or a drain behind the wall are built and kept working. Otherwise water pressure must be included.".to_string())); }
    if e_ecc > bw / 6.0 { obs.push(("AMBER", "The resultant under characteristic loads is outside the middle third of the base (partial contact).".to_string())); }
    obs.push(("INFO", "Global (slip-circle) stability, settlement, passive resistance, shear keys and seismic actions are not assessed. The allowable bearing pressure must come from a geotechnical report.".to_string()));
    obs.push(("INFO", "Geotechnical partial factors are the EN 1997-1 recommended values for Design Approach 1. The National Annex may differ.".to_string()));
    let summary = vec![kv("Wall height / base", format!("{:.2} m / {:.2} m wide", h, bw)), kv("K a (characteristic)", format!("{:.4}", ka_k)), kv("Max base pressure", format!("{:.1} kPa (allow {:.0})", prk.1, q_allow)), kv("Eccentricity e", format!("{:.3} m (B/6 = {:.3})", e_ecc, bw / 6.0)), kv("Stem reinforcement", format!("T{} @ {:.0} ({:.0} mm²/m)", phs, s_stem, as_stem)), kv("Toe reinforcement (bottom)", format!("T{} @ {:.0} ({:.0} mm²/m)", phb, s_toe, as_toe)), kv("Heel reinforcement (top)", format!("T{} @ {:.0} ({:.0} mm²/m)", phb, s_heel, as_heel)), kv("Drainage", if hw <= 0.0 { "Assumed fully drained".to_string() } else { format!("Water to {:.2} m", hw) })];
    finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: Rankine active pressure with surcharge and optional water, sliding (DA1-1, DA1-2), overturning (EQU), base pressure, stem, toe and heel flexure and shear with reinforcement design.".into(),
        "NOT implemented: global stability, settlement, passive resistance, sloping backfill, wall friction, seismic, counterfort and gravity walls, shear keys, crack control, anchorage and curtailment.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Retain { hs: hs * 1000.0, tb: tb * 1000.0, ts: ts * 1000.0, lt: lt * 1000.0, b: bw * 1000.0 })
}
