//! Braced RC wall, 1 m length strip: axial load + out-of-plane bending (EN 1992-1-1:2004 cl. 5.8, 9.6).
use crate::column::{mrd, n_rd0, nm, S};
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

const ES: f64 = 200_000.0;

pub fn def() -> ModDef {
    ModDef { id: "wall", name: "RC wall", title: "Reinforced concrete wall (per metre length)", subtitle: "EN 1992-1-1:2004 · Braced wall, vertical load with out-of-plane bending, nominal curvature second order", available: true, has_check_mode: true, fields: vec![
        fld("h", "Wall thickness h", "mm", "Geometry", 200.0), fld("len", "Clear height l", "mm", "Geometry", 3000.0), fld("beta", "Effective length factor l0/l", "-", "Geometry", 1.0),
        fld("cover", "Cover c (to horizontal bars)", "mm", "Geometry", 30.0), fld("bar_dia", "Vertical bar φ", "mm", "Geometry", 12.0), fld("hbar_dia", "Horizontal bar φ", "mm", "Geometry", 10.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0), fld("phi_ef", "Effective creep ratio φef", "-", "Materials", 1.0),
        fld("n_ed", "Vertical load N Ed (ULS)", "kN/m", "Actions - ULS", 600.0), fld("m02", "Larger end moment M02 (out of plane)", "kNm/m", "Actions - ULS", 12.0), fld("m01", "Smaller end moment M01 (signed)", "kNm/m", "Actions - ULS", 6.0),
        fld_chk("spacing", "Vertical bar spacing, each face (drawing)", "mm", "Provided reinforcement", 200.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let check = v.get("mode").and_then(|m| m.as_str()) == Some("check");
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let m01 = match num(v, "m01") { Some(x) => x, None => return fail(vec!["Smaller end moment M01 is required".into()]) };
    let phi_ef = num(v, "phi_ef").unwrap_or(0.0).max(0.0);
    let mut v2 = v.clone(); v2["m01"] = 1.0.into(); v2["phi_ef"] = 1.0.into();
    let x = match require(&v2, &d0, check) { Ok(x) => x, Err(e) => return fail(e) };
    let (h, len, beta, cover, phi, hdia, fck, fyk, n_ed, m02, s_in) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[9], x[10], x[12]);
    if !(12.0..=50.0).contains(&fck) { return fail(vec!["fck must be within 12-50 MPa for this release".into()]); }
    if m01.abs() > m02 { return fail(vec!["|M01| must not exceed M02 (M02 is the larger end moment)".into()]); }
    let b = 1000.0; let ac = b * h;
    let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    let d2 = cover + hdia + phi / 2.0; let d = h - d2;
    if d <= d2 { return fail(vec!["Wall too thin for the cover and bar sizes".into()]); }
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = ("1. Design data", "2. Slenderness", "3. Design moment", "4. Resistance: axial load and bending", "5. Detailing");
    t.step(s1, "Geometry and materials", "", "", &format!("h = {} mm;  clear height l = {} mm;  c = {} mm (to horizontal bars);  vertical φ = {} mm;  horizontal φ = {} mm;  f_{{ck}} = {} MPa;  f_{{yk}} = {} MPa", h, len, cover, phi, hdia, fck, fyk), "", "Design strip of 1.0 m length (b = 1000 mm). Bending about the wall's long axis (out of plane).");
    t.step(s1, "Actions (ULS, per metre length)", "", "", &format!("N_{{Ed}} = {} kN/m;  M_{{02}} = {} kNm/m;  M_{{01}} = {} kNm/m", n_ed, m02, m01), "", "First-order end moments, including eccentricity of the supported floors, supplied by the user.");
    t.step(s1, "Partial factors and strengths", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6", "f_{cd} = α_{cc}·f_{ck}/γ_c;   f_{yd} = f_{yk}/γ_s", &format!("γ_c = {:.2}; γ_s = {:.2}; α_cc = {:.2};  f_cd = {:.2} × {} / {:.2};  f_yd = {} / {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc, ps.alpha_cc, fck, ps.gamma_c, fyk, ps.gamma_s), &format!("f_{{cd}} = {:.2} MPa;  f_{{yd}} = {:.2} MPa", fcd, fyd), &format!("Parameter set: {} [{}]", ps.label, ps.status));
    t.step(s1, "Reinforcement geometry", "EN 1992-1-1 cl. 4.4.1", "d_2 = c + φ_h + φ/2;   d = h − d_2", &format!("d_2 = {} + {} + {}/2 = {:.1} mm;  d = {} − {:.1}", cover, hdia, phi, d2, h, d2), &format!("d_{{2}} = {:.1} mm;  d = {:.1} mm", d2, d), "");

    let l0 = beta * len; let i_r = h / 12f64.sqrt(); let lam = l0 / i_r;
    let n_rel = n_ed * 1e3 / (ac * fcd); let rm = m01 / m02;
    let a_f = 1.0 / (1.0 + 0.2 * phi_ef); let c_f = 1.7 - rm; let lam_lim = 20.0 * a_f * 1.1 * c_f / n_rel.sqrt();
    t.step(s2, "Slenderness ratio", "EN 1992-1-1 cl. 5.8.3.2", "l_0 = β·l;   i = h/√12;   λ = l_0/i", &format!("l_0 = {} × {} = {:.0} mm;  i = {}/√12 = {:.1} mm", beta, len, l0, h, i_r), &format!("λ = {:.1}", lam), "Effective length factor β is the user's input and must reflect the wall's actual restraint.");
    t.step(s2, "Limiting slenderness", "EN 1992-1-1 cl. 5.8.3.1, Eq 5.13N", "n = N_{Ed}/(A_c·f_{cd});  A = 1/(1 + 0.2φ_{ef});  B = 1.1;  C = 1.7 − r_m;  λ_{lim} = 20·A·B·C/√n", &format!("n = {}×10^{{3}}/({:.0} × {:.2}) = {:.3};  A = {:.3};  C = {:.2};  λ_lim = 20 × {:.3} × 1.1 × {:.2}/√{:.3}", n_ed, ac, fcd, n_rel, a_f, c_f, a_f, c_f, n_rel), &format!("λ_{{lim}} = {:.1};  {}", lam_lim, if lam <= lam_lim { "λ ≤ λ_lim: second-order effects may be ignored" } else { "λ > λ_lim: second-order effects must be included" }), "");
    let slender = lam > lam_lim;
    let e0 = (h / 30.0).max(20.0); let m0e = (0.6 * m02 + 0.4 * m01).max(0.4 * m02); let m_imp = n_ed * e0 / 1e3; let m0 = m0e.max(m_imp);
    t.step(s3, "First-order design moment", "EN 1992-1-1 cl. 5.8.8.2(2), 6.1(4)", "M_{0e} = max(0.6·M_{02} + 0.4·M_{01} ; 0.4·M_{02});   e_0 = max(h/30 ; 20 mm);   M_{0Ed} = max(M_{0e} ; N_{Ed}·e_0)", &format!("M_0e = {:.2} kNm/m;  e_0 = {:.1} mm;  N·e_0 = {} × {:.1}/1000 = {:.2} kNm/m", m0e, e0, n_ed, e0, m_imp), &format!("M_{{0Ed}} = {:.2} kNm/m", m0), "");
    let beta_phi = (0.35 + fck / 200.0 - lam / 150.0).max(0.0);
    let area1 = PI * phi * phi / 4.0;
    let s_max = (3.0 * h).min(400.0); let s_max_r = (s_max / 25.0).floor() * 25.0;
    let min_gap = phi.max(25.0);

    // evaluate a spacing: returns (a_each, a_tot, kr, kphi, inv_r, e2, m2, m_tot, m_rd)
    let eval = |sp: f64| {
        let a_each = 1000.0 * area1 / sp; let a_tot = 2.0 * a_each; let s = S { b, h, d2, a: a_each, fcd, fyd };
        let omega = a_tot * fyd / (ac * fcd);
        let (mut kr, mut kphi, mut inv_r, mut e2, mut m2) = (1.0, 1.0, 0.0, 0.0, 0.0);
        if slender { kr = (((1.0 + omega) - n_rel) / ((1.0 + omega) - 0.4)).clamp(0.0, 1.0); kphi = (1.0 + beta_phi * phi_ef).max(1.0); inv_r = kr * kphi * (fyd / ES) / (0.45 * d); e2 = inv_r * l0 * l0 / 10.0; m2 = n_ed * e2 / 1e3; }
        let m_tot = if slender { m02.max(m0 + m2).max(m_imp) } else { m0 };
        (a_each, a_tot, omega, kr, kphi, inv_r, e2, m2, m_tot, mrd(&s, n_ed * 1e3).map(|(m, xx)| (m / 1e6, xx)), s)
    };
    let a_min_tot = 0.002 * ac;
    let spacing = if check { s_in } else {
        let mut sp = s_max_r; let mut found = sp;
        while sp >= (phi + 25.0).max(75.0) { let e = eval(sp); found = sp; if e.1 >= a_min_tot && e.9.map(|(m, _)| e.8 <= m).unwrap_or(false) { break; } sp -= 25.0; }
        found
    };
    if spacing <= 0.0 { return fail(vec!["Bar spacing must be positive".into()]); }
    let (a_each, a_tot, omega, kr, kphi, inv_r, e2, m2, m_tot, m_rd, s) = eval(spacing);
    if slender {
        t.step(s3, "Second-order moment, nominal curvature method", "EN 1992-1-1 cl. 5.8.8.2, 5.8.8.3", "ω = A_s·f_{yd}/(A_c·f_{cd});  K_r = (1 + ω − n)/(1 + ω − 0.4) ≤ 1;  β = 0.35 + f_{ck}/200 − λ/150;  K_φ = 1 + β·φ_{ef};  1/r = K_r·K_φ·(f_{yd}/E_s)/(0.45·d);  e_2 = (1/r)·l_0^{2}/10;  M_2 = N_{Ed}·e_2", &format!("A_s = {:.0} mm^{{2}}/m;  ω = {:.3};  K_r = {:.3};  β = {:.3};  K_φ = {:.3};  1/r = {:.3}×10^{{-5}} /mm;  e_2 = {:.1} mm;  M_2 = {} × {:.1}/1000", a_tot, omega, kr, beta_phi, kphi, inv_r * 1e5, e2, n_ed, e2), &format!("M_{{2}} = {:.2} kNm/m", m2), "Evaluated for the final bar spacing.");
        t.step(s3, "Total design moment", "EN 1992-1-1 cl. 5.8.8.2(2), Eq 5.31", "M_{Ed} = max(M_{02} ; M_{0Ed} + M_2 ; N_{Ed}·e_0)", &format!("= max({:.2} ; {:.2} + {:.2} ; {:.2})", m02, m0, m2, m_imp), &format!("M_{{Ed}} = {:.2} kNm/m", m_tot), "");
    } else { t.step(s3, "Total design moment", "EN 1992-1-1 cl. 5.8.3.1", "M_{Ed} = M_{0Ed}  (second-order effects ignored)", "", &format!("M_{{Ed}} = {:.2} kNm/m", m_tot), ""); }
    t.step(s4, if check { "Provided vertical reinforcement (from drawing)" } else { "Vertical reinforcement" }, "", "A_{s,face} = 1000·π·φ^{2}/(4·s);   A_{s,tot} = 2·A_{s,face}", &format!("= 1000 × π × {}^{{2}}/(4 × {:.0}) = {:.0} mm^{{2}}/m per face", phi, spacing, a_each), &format!("A_{{s,tot}} = {:.0} mm^{{2}}/m  (T{} @ {:.0} each face)", a_tot, phi, spacing), "Equal reinforcement in both faces.");
    let nrd0 = n_rd0(&s) / 1e3;
    t.step(s4, "Axial resistance (zero eccentricity)", "EN 1992-1-1 cl. 6.1", "N_{Rd,0} = f_{cd}·b·h + A_{s,tot}·σ_s  (strain 0.002)", &format!("= {:.2} × 1000 × {} + {:.0} × σ_s", fcd, h, a_tot), &format!("N_{{Rd,0}} = {:.0} kN/m", nrd0), "");
    let mut checks = vec![];
    match m_rd {
        Some((m, xx)) => {
            let (nn, _) = nm(&s, xx);
            t.step(s4, "Moment resistance at N_Ed (strain compatibility)", "EN 1992-1-1 cl. 3.1.7, 6.1, 3.2.7", "N_{Rd}(x) = F_c + F_{s2} + F_{s1} = N_{Ed};   ε_{cu} = 0.0035;   σ_s = E_s·ε_s ≤ f_{yd};   M_{Rd} = ΣF·(h/2 − y)", &format!("Solved x = {:.1} mm;  N_Rd = {:.1} kN/m", xx, nn / 1e3), &format!("M_{{Rd}} = {:.2} kNm/m", m), "Rectangular stress block (λ = 0.8, η = 1.0).");
            checks.push(chk("Axial + bending interaction", "EN 1992-1-1 6.1, 5.8.8", format!("{:.2} kNm/m", m_tot), format!("{:.2} kNm/m", m), m_tot / m, m_tot <= m));
        }
        None => checks.push(chk("Axial + bending interaction", "EN 1992-1-1 6.1", format!("{} kN/m", n_ed), format!("{:.0} kN/m max", nrd0), 9.99, false)),
    }
    checks.push(chk("Axial capacity", "EN 1992-1-1 6.1", format!("{} kN/m", n_ed), format!("{:.0} kN/m", nrd0), n_ed / nrd0, n_ed <= nrd0));
    let a_max = 0.04 * ac;
    t.step(s5, "Vertical reinforcement limits", "EN 1992-1-1 cl. 9.6.2(1), 9.6.2(2)", "A_{s,v,min} = 0.002·A_c;   A_{s,v,max} = 0.04·A_c  (each limit on the total of both faces)", &format!("= 0.002 × {:.0};  = 0.04 × {:.0}", ac, ac), &format!("A_{{s,v,min}} = {:.0} mm^{{2}}/m;  A_{{s,v,max}} = {:.0} mm^{{2}}/m", a_min_tot, a_max), "Where N_Ed ≥ ... the minimum may be reduced only by agreement; not applied here.");
    checks.push(chk("Minimum vertical reinforcement", "EN 1992-1-1 9.6.2(1)", format!("{:.0} mm²/m", a_min_tot), format!("{:.0} mm²/m", a_tot), a_min_tot / a_tot, a_tot >= a_min_tot));
    checks.push(chk("Maximum vertical reinforcement", "EN 1992-1-1 9.6.2(2)", format!("{:.0} mm²/m", a_tot), format!("{:.0} mm²/m", a_max), a_tot / a_max, a_tot <= a_max));
    t.step(s5, "Vertical bar spacing", "EN 1992-1-1 cl. 9.6.2(3), 8.2(2)", "s ≤ min(3h ; 400 mm);   s_{clear} = s − φ ≥ max(φ ; 25 mm)", &format!("s_max = min({:.0} ; 400) = {:.0} mm;  s_clear = {:.0} − {} = {:.0} mm;  limit = {:.0} mm", 3.0 * h, s_max, spacing, phi, spacing - phi, min_gap), &format!("s = {:.0} mm  {} {:.0} mm", spacing, if spacing <= s_max + 1e-9 { "≤" } else { ">" }, s_max), "");
    checks.push(chk("Vertical bar spacing (max)", "EN 1992-1-1 9.6.2(3)", format!("{:.0} mm", spacing), format!("{:.0} mm max", s_max), spacing / s_max, spacing <= s_max + 1e-9));
    checks.push(chk("Vertical bar spacing (clear)", "EN 1992-1-1 8.2(2)", format!("{:.0} mm min", min_gap), format!("{:.0} mm clear", spacing - phi), min_gap / (spacing - phi).max(0.1), spacing - phi >= min_gap));
    let a_h_min = (0.25 * a_tot).max(0.001 * ac);
    let a_h_bar = 2.0 * PI * hdia * hdia / 4.0;
    let sh = (((1000.0 * a_h_bar / a_h_min).min(400.0)) / 25.0).floor() * 25.0;
    t.step(s5, "Horizontal reinforcement", "EN 1992-1-1 cl. 9.6.3(1), 9.6.3(2)", "A_{s,h,min} = max(0.25·A_{s,v} ; 0.001·A_c);   s_h ≤ 400 mm", &format!("= max(0.25 × {:.0} ; 0.001 × {:.0}) = {:.0} mm^{{2}}/m;  two faces: 2 × π × {}^{{2}}/4 = {:.1} mm^{{2}} per bar pair", a_tot, ac, a_h_min, hdia, a_h_bar), &format!("Use T{} @ {:.0} mm c/c, each face", hdia, sh), "Not part of the PASS/FAIL set; detail on drawings.");
    let mut obs: Vec<(&str, String)> = vec![("INFO", "In-plane shear, in-plane bending and concentrated loads on the wall are not assessed. Only the out-of-plane strip is checked.".to_string())];
    if a_tot > 0.02 * ac { obs.push(("AMBER", "Vertical steel exceeds 2% of Ac: transverse links to restrain the compression bars are required (cl. 9.6.2(4)); not designed here.".to_string())); }
    if slender { obs.push(("INFO", format!("Slender wall (λ = {:.0} > λ_lim = {:.0}): second-order effects included by the nominal curvature method.", lam, lam_lim))); }
    if lam > 140.0 { obs.push(("RED", "λ exceeds 140: outside the intended range of the simplified method.".to_string())); }
    let summary = vec![kv("Slenderness λ / λlim", format!("{:.1} / {:.1}", lam, lam_lim)), kv("Design moment M Ed", format!("{:.2} kNm/m{}", m_tot, if slender { format!("  (M2 = {:.2})", m2) } else { String::new() })), kv("Moment resistance M Rd", m_rd.map(|(m, _)| format!("{:.2} kNm/m", m)).unwrap_or("n/a".into())), kv("Axial capacity N Rd,0", format!("{:.0} kN/m", nrd0)), kv("Vertical steel", format!("T{} @ {:.0} each face  ({:.0} mm²/m)", phi, spacing, a_tot)), kv("Horizontal steel", format!("T{} @ {:.0} each face", hdia, sh)), kv("Steel ratio", format!("{:.2} %", 100.0 * a_tot / ac))];
    let mut res =     finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: slenderness, nominal curvature second order, N-M interaction by strain compatibility, vertical and horizontal reinforcement limits.".into(),
        "NOT implemented: in-plane shear and bending, shear walls, plain concrete walls (cl. 12.6), openings, concentrated loads, fire resistance, confinement of boundary elements.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Wall { h, cover, bar_dia: phi, spacing });
    res.outputs = vec![out("n_ed", "Vertical load N Ed (ULS)", n_ed, "kN/m"), out("reaction", "Base reaction (ULS)", n_ed, "kN/m")];
    res
}
