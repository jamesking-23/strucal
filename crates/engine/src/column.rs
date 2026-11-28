//! Braced rectangular RC column: axial load + uniaxial bending (EN 1992-1-1:2004).
//! Interaction by strain compatibility (rectangular stress block); second order by nominal curvature (5.8.8).
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

const ES: f64 = 200_000.0;

pub fn def() -> ModDef {
    ModDef { id: "column", name: "RC column", title: "Reinforced concrete column", subtitle: "EN 1992-1-1:2004 · Braced rectangular column, axial load with uniaxial bending, nominal curvature second order", available: true, has_check_mode: true, fields: vec![
        fld("b", "Width b (out of bending plane)", "mm", "Geometry", 300.0), fld("h", "Depth h (in bending plane)", "mm", "Geometry", 400.0), fld("len", "Column clear height l", "mm", "Geometry", 3500.0), fld("beta", "Effective length factor l0/l", "-", "Geometry", 1.0),
        fld("cover", "Cover c", "mm", "Geometry", 35.0), fld("link_dia", "Link φ", "mm", "Geometry", 8.0), fld("bar_dia", "Longitudinal bar φ", "mm", "Geometry", 20.0),
        fld("fck", "fck", "MPa", "Materials", 30.0), fld("fyk", "fyk", "MPa", "Materials", 500.0), fld("phi_ef", "Effective creep ratio φef", "-", "Materials", 1.0),
        fld("n_ed", "Axial load N Ed (ULS)", "kN", "Actions - ULS", 1500.0), fld("m02", "Larger end moment M02", "kNm", "Actions - ULS", 80.0), fld("m01", "Smaller end moment M01 (signed, e.g. -40 for double curvature)", "kNm", "Actions - ULS", 40.0),
        fld_chk("n_face", "Bars per face (top and bottom row) (drawing)", "bars", "Provided reinforcement", 3.0), fld_chk("link_spacing", "Link spacing s (drawing)", "mm", "Provided reinforcement", 250.0),
    ] }
}

pub(crate) struct S { pub b: f64, pub h: f64, pub d2: f64, pub a: f64, pub fcd: f64, pub fyd: f64 }
pub(crate) fn eps(s: &S, x: f64, y: f64) -> f64 { if x <= s.h { 0.0035 * (x - y) / x } else { 0.002 * (x - y) / (x - 3.0 * s.h / 7.0) } }
pub(crate) fn sig(e: f64, fyd: f64) -> f64 { (ES * e).clamp(-fyd, fyd) }
/// Axial force (N, compression +) and moment about the centroid (Nmm) for neutral axis depth x.
pub(crate) fn nm(s: &S, x: f64) -> (f64, f64) {
    let blk = (0.8 * x).min(s.h); let fc = s.b * blk * s.fcd; let yc = blk / 2.0;
    let (ysc, yst) = (s.d2, s.h - s.d2);
    let mut ssc = sig(eps(s, x, ysc), s.fyd); if ysc < blk { ssc -= s.fcd; }
    let mut sst = sig(eps(s, x, yst), s.fyd); if yst < blk { sst -= s.fcd; }
    let (fsc, fst) = (s.a * ssc, s.a * sst);
    (fc + fsc + fst, fc * (s.h / 2.0 - yc) + fsc * (s.h / 2.0 - ysc) + fst * (s.h / 2.0 - yst))
}
pub(crate) fn n_rd0(s: &S) -> f64 { nm(s, 10.0 * s.h).0 }
pub(crate) fn mrd(s: &S, n: f64) -> Option<(f64, f64)> {
    if n > n_rd0(s) { return None; }
    let (mut lo, mut hi) = (1e-3, 10.0 * s.h);
    for _ in 0..200 { let mid = 0.5 * (lo + hi); if nm(s, mid).0 < n { lo = mid; } else { hi = mid; } }
    let x = 0.5 * (lo + hi); Some((nm(s, x).1, x))
}

struct Ev { nface: u32, a_each: f64, a_tot: f64, omega: f64, kr: f64, kphi: f64, inv_r: f64, e2: f64, m2: f64, m_tot: f64, m_rd: Option<(f64, f64)>, d: f64, beta_phi: f64 }

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let check = v.get("mode").and_then(|m| m.as_str()) == Some("check");
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    // m01 and phi_ef may be zero/negative, so read them separately
    let m01 = match num(v, "m01") { Some(x) => x, None => return fail(vec!["Smaller end moment M01 is required".into()]) };
    let phi_ef = num(v, "phi_ef").unwrap_or(0.0).max(0.0);
    let mut v2 = v.clone(); v2["m01"] = 1.0.into(); v2["phi_ef"] = 1.0.into();
    let x = match require(&v2, &d0, check) { Ok(x) => x, Err(e) => return fail(e) };
    let (b, h, len, beta, cover, ldia, phi, fck, fyk, n_ed, m02, nface_in, s_in) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[10], x[11], x[13], x[14]);
    if !(12.0..=50.0).contains(&fck) { return fail(vec!["fck must be within 12-50 MPa for this release".into()]); }
    if check && (nface_in.fract() != 0.0 || !(2.0..=14.0).contains(&nface_in)) { return fail(vec!["Bars per face must be a whole number from 2 to 14".into()]); }
    if m01.abs() > m02 { return fail(vec!["|M01| must not exceed M02 (M02 is the larger end moment)".into()]); }
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = ("1. Design data", "2. Slenderness", "3. Design moment", "4. Resistance: axial load and bending", "5. Detailing");
    let ac = b * h; let fcd = ps.alpha_cc * fck / ps.gamma_c; let fyd = fyk / ps.gamma_s;
    let d2 = cover + ldia + phi / 2.0; let d = h - d2;
    if d <= d2 { return fail(vec!["Section too small for the cover and bar sizes".into()]); }
    t.step(s1, "Geometry and materials", "", "", &format!("b = {} mm;  h = {} mm (bending plane);  clear height l = {} mm;  c = {} mm;  link φ = {} mm;  bar φ = {} mm;  f_{{ck}} = {} MPa;  f_{{yk}} = {} MPa", b, h, len, cover, ldia, phi, fck, fyk), "", "Braced column, bending about the axis perpendicular to h. Symmetric reinforcement in the two faces parallel to b.");
    t.step(s1, "Actions (ULS)", "", "", &format!("N_{{Ed}} = {} kN;  M_{{02}} = {} kNm;  M_{{01}} = {} kNm", n_ed, m02, m01), "", "First-order end moments from the frame analysis, supplied by the user.");
    t.step(s1, "Partial factors", "EN 1992-1-1 cl. 2.4.2.4, 3.1.6", "f_{cd} = α_{cc}·f_{ck}/γ_c;   f_{yd} = f_{yk}/γ_s", &format!("γ_c = {:.2}; γ_s = {:.2}; α_cc = {:.2};  f_cd = {:.2} × {} / {:.2};  f_yd = {} / {:.2}", ps.gamma_c, ps.gamma_s, ps.alpha_cc, ps.alpha_cc, fck, ps.gamma_c, fyk, ps.gamma_s), &format!("f_{{cd}} = {:.2} MPa;  f_{{yd}} = {:.2} MPa", fcd, fyd), &format!("Parameter set: {} [{}]", ps.label, ps.status));
    t.step(s1, "Reinforcement geometry", "EN 1992-1-1 cl. 4.4.1", "d_2 = c + φ_{link} + φ/2;   d = h − d_2", &format!("d_2 = {} + {} + {}/2 = {:.1} mm;  d = {} − {:.1}", cover, ldia, phi, d2, h, d2), &format!("d_{{2}} = {:.1} mm;  d = {:.1} mm", d2, d), "");

    // slenderness
    let l0 = beta * len; let i_r = h / 12f64.sqrt(); let lam = l0 / i_r;
    let n_rel = n_ed * 1e3 / (ac * fcd);
    let rm = m01 / m02;
    let a_f = 1.0 / (1.0 + 0.2 * phi_ef); let b_f = 1.1; let c_f = 1.7 - rm;
    let lam_lim = 20.0 * a_f * b_f * c_f / n_rel.sqrt();
    t.step(s2, "Slenderness ratio", "EN 1992-1-1 cl. 5.8.3.2(1)", "l_0 = β·l;   i = h/√12;   λ = l_0/i", &format!("l_0 = {} × {} = {:.0} mm;  i = {}/√12 = {:.1} mm", beta, len, l0, h, i_r), &format!("λ = {:.1}", lam), "");
    t.step(s2, "Limiting slenderness", "EN 1992-1-1 cl. 5.8.3.1(1), Eq 5.13N", "n = N_{Ed}/(A_c·f_{cd});  A = 1/(1 + 0.2φ_{ef});  B = 1.1 (ω unknown);  C = 1.7 − r_m,  r_m = M_{01}/M_{02};  λ_{lim} = 20·A·B·C/√n", &format!("n = {}×10^{{3}}/({:.0} × {:.2}) = {:.3};  A = {:.3};  B = 1.1;  C = 1.7 − ({:.2}) = {:.2};  λ_lim = 20 × {:.3} × 1.1 × {:.2} / √{:.3}", n_ed, ac, fcd, n_rel, a_f, rm, c_f, a_f, c_f, n_rel), &format!("λ_{{lim}} = {:.1};  {}", lam_lim, if lam <= lam_lim { "λ ≤ λ_lim: second-order effects may be ignored" } else { "λ > λ_lim: second-order effects must be included" }), "");
    let slender = lam > lam_lim;

    // design moment
    let e0 = (h / 30.0).max(20.0);
    let m0e = (0.6 * m02 + 0.4 * m01).max(0.4 * m02);
    let m_imp = n_ed * e0 / 1e3;
    t.step(s3, "First-order design moment", "EN 1992-1-1 cl. 5.8.8.2(2), 6.1(4)", "M_{0e} = max(0.6·M_{02} + 0.4·M_{01} ; 0.4·M_{02});   e_0 = max(h/30 ; 20 mm);   M_{0Ed} = max(M_{0e} ; N_{Ed}·e_0)", &format!("M_0e = max({:.1} ; {:.1}) = {:.1} kNm;  e_0 = max({:.1} ; 20) = {:.1} mm;  N·e_0 = {} × {:.1}/1000 = {:.1} kNm", 0.6 * m02 + 0.4 * m01, 0.4 * m02, m0e, h / 30.0, e0, n_ed, e0, m_imp), &format!("M_{{0Ed}} = {:.1} kNm", m0e.max(m_imp)), "Minimum eccentricity per cl. 6.1(4).");
    let m0 = m0e.max(m_imp);
    let beta_phi = (0.35 + fck / 200.0 - lam / 150.0).max(0.0);

    let area1 = PI * phi * phi / 4.0;
    let evaluate = |nface: u32| -> Ev {
        let a_each = nface as f64 * area1; let a_tot = 2.0 * a_each;
        let s = S { b, h, d2, a: a_each, fcd, fyd };
        let omega = a_tot * fyd / (ac * fcd);
        let (mut kr, mut kphi, mut inv_r, mut e2, mut m2) = (1.0, 1.0, 0.0, 0.0, 0.0);
        if slender {
            let nu = 1.0 + omega; let nbal = 0.4;
            kr = ((nu - n_rel) / (nu - nbal)).clamp(0.0, 1.0);
            kphi = (1.0 + beta_phi * phi_ef).max(1.0);
            inv_r = kr * kphi * (fyd / ES) / (0.45 * d);
            e2 = inv_r * l0 * l0 / 10.0;
            m2 = n_ed * e2 / 1e3;
        }
        let m_tot = if slender { m02.max(m0 + m2).max(m_imp) } else { m0 };
        Ev { nface, a_each, a_tot, omega, kr, kphi, inv_r, e2, m2, m_tot, m_rd: mrd(&s, n_ed * 1e3).map(|(m, x)| (m / 1e6, x)), d, beta_phi }
    };
    let avail = b - 2.0 * (cover + ldia);
    let max_face = { let mut n = 2u32; while n < 14 && (avail - (n + 1) as f64 * phi) / (n as f64) >= phi.max(25.0) { n += 1; } n };
    let ev = if check { evaluate(nface_in.round() as u32) } else {
        let mut chosen = evaluate(2);
        for n in 2..=max_face { let e = evaluate(n); let ok = e.m_rd.map(|(m, _)| e.m_tot <= m).unwrap_or(false); chosen = e; if ok { break; } }
        chosen
    };
    if ev.nface < 2 { return fail(vec!["At least 2 bars per face (4 bars in total) are required".into()]); }
    if slender {
        t.step(s3, "Second-order moment, nominal curvature method", "EN 1992-1-1 cl. 5.8.8.2, 5.8.8.3, Eq 5.34-5.37", "ω = A_s·f_{yd}/(A_c·f_{cd});  n_u = 1 + ω;  n_{bal} = 0.4;  K_r = (n_u − n)/(n_u − n_{bal}) ≤ 1;  β = 0.35 + f_{ck}/200 − λ/150;  K_φ = 1 + β·φ_{ef} ≥ 1;   1/r = K_r·K_φ·(f_{yd}/E_s)/(0.45·d);   e_2 = (1/r)·l_0^{2}/10;   M_2 = N_{Ed}·e_2",
            &format!("A_s = {:.0} mm^{{2}};  ω = {:.3};  K_r = {:.3};  β = {:.3};  K_φ = {:.3};  1/r = {:.3}×10^{{-5}} /mm;  e_2 = {:.1} mm;  M_2 = {} × {:.1}/1000", ev.a_tot, ev.omega, ev.kr, ev.beta_phi, ev.kphi, ev.inv_r * 1e5, ev.e2, n_ed, ev.e2), &format!("M_{{2}} = {:.1} kNm", ev.m2), "Reinforcement trial: the section was iterated on the number of bars.");
        t.step(s3, "Total design moment", "EN 1992-1-1 cl. 5.8.8.2(2), Eq 5.31", "M_{Ed} = max(M_{02} ; M_{0Ed} + M_2 ; N_{Ed}·e_0)", &format!("= max({:.1} ; {:.1} + {:.1} ; {:.1})", m02, m0, ev.m2, m_imp), &format!("M_{{Ed}} = {:.1} kNm", ev.m_tot), "");
    } else {
        t.step(s3, "Total design moment", "EN 1992-1-1 cl. 5.8.3.1", "M_{Ed} = M_{0Ed}  (second-order effects ignored)", "", &format!("M_{{Ed}} = {:.1} kNm", ev.m_tot), "");
    }
    // resistance
    let a_prov = ev.a_tot;
    t.step(s4, if check { "Provided longitudinal reinforcement (from drawing)" } else { "Longitudinal reinforcement" }, "", "A_{s1} = A_{s2} = n_{face}·π·φ^{2}/4;   A_{s,tot} = 2·A_{s1}", &format!("= {} × π × {}^{{2}}/4 = {:.0} mm^{{2}} per face", ev.nface, phi, ev.a_each), &format!("A_{{s,tot}} = {:.0} mm^{{2}}  ({}T{})", a_prov, 2 * ev.nface, phi), "Reinforcement in the two faces parallel to b only. Side bars are not counted in the resistance.");
    let s = S { b, h, d2, a: ev.a_each, fcd, fyd };
    let nrd0 = n_rd0(&s) / 1e3;
    t.step(s4, "Axial resistance (zero eccentricity)", "EN 1992-1-1 cl. 6.1", "N_{Rd,0} = f_{cd}·b·h + A_{s,tot}·σ_s  (strain 0.002)", &format!("= {:.2} × {} × {} + {:.0} × {:.0}", fcd, b, h, a_prov, ES * 0.002_f64.min(fyd / ES)), &format!("N_{{Rd,0}} = {:.0} kN", nrd0), "");
    let mut checks = vec![];
    match ev.m_rd {
        Some((m, xx)) => {
            let (nn, _mm) = nm(&s, xx);
            let ea = eps(&s, xx, d2); let et = eps(&s, xx, h - d2);
            t.step(s4, "Moment resistance at N_Ed (strain compatibility)", "EN 1992-1-1 cl. 3.1.7, 6.1, 3.2.7", "Equilibrium N_{Rd}(x) = F_c + F_{s2} + F_{s1} = N_{Ed};  F_c = λx·b·f_{cd};  ε_{cu} = 0.0035;  σ_s = E_s·ε_s ≤ f_{yd};   M_{Rd} = ΣF·(h/2 − y)", &format!("Solved x = {:.1} mm (λx = {:.1} mm);  ε_s2 = {:.5} (compression face steel);  ε_s1 = {:.5} (far face steel);  N_Rd = {:.1} kN", xx, (0.8 * xx).min(h), ea, et, nn / 1e3), &format!("M_{{Rd}} = {:.1} kNm", m), "Rectangular stress block (λ = 0.8, η = 1.0). Displaced concrete deducted for steel inside the block.");
            checks.push(chk("Axial + bending interaction", "EN 1992-1-1 6.1, 5.8.8", format!("{:.1} kNm", ev.m_tot), format!("{:.1} kNm", m), ev.m_tot / m, ev.m_tot <= m));
        }
        None => { checks.push(chk("Axial + bending interaction", "EN 1992-1-1 6.1", format!("{} kN", n_ed), format!("{:.0} kN max", nrd0), 9.99, false)); }
    }
    checks.push(chk("Axial capacity", "EN 1992-1-1 6.1", format!("{} kN", n_ed), format!("{:.0} kN", nrd0), n_ed / nrd0, n_ed <= nrd0));
    // detailing
    let as_min = (0.10 * n_ed * 1e3 / fyd).max(0.002 * ac); let as_max = 0.04 * ac;
    t.step(s5, "Minimum and maximum longitudinal reinforcement", "EN 1992-1-1 cl. 9.5.2(2), 9.5.2(3)", "A_{s,min} = max(0.10·N_{Ed}/f_{yd} ; 0.002·A_c);   A_{s,max} = 0.04·A_c", &format!("= max(0.10 × {}×10^{{3}}/{:.2} ; 0.002 × {:.0}) = max({:.0} ; {:.0});  A_s,max = 0.04 × {:.0}", n_ed, fyd, ac, 0.10 * n_ed * 1e3 / fyd, 0.002 * ac, ac), &format!("A_{{s,min}} = {:.0} mm^{{2}};  A_{{s,max}} = {:.0} mm^{{2}}", as_min, as_max), "A_s,max applies outside lap zones.");
    checks.push(chk("Minimum reinforcement", "EN 1992-1-1 9.5.2(2)", format!("{:.0} mm²", as_min), format!("{:.0} mm²", a_prov), as_min / a_prov, a_prov >= as_min));
    checks.push(chk("Maximum reinforcement", "EN 1992-1-1 9.5.2(3)", format!("{:.0} mm²", a_prov), format!("{:.0} mm²", as_max), a_prov / as_max, a_prov <= as_max));
    let n_f = ev.nface as f64;
    let gap = (avail - n_f * phi) / (n_f - 1.0); let min_gap = phi.max(25.0);
    t.step(s5, "Clear spacing of bars", "EN 1992-1-1 cl. 8.2(2)", "s_{clear} = [b − 2(c + φ_{link}) − n·φ]/(n − 1) ≥ max(φ ; 20 + 5)", &format!("= [{} − 2 × ({} + {}) − {} × {}]/({} − 1)", b, cover, ldia, ev.nface, phi, ev.nface), &format!("s_{{clear}} = {:.1} mm  {} {:.0} mm", gap, if gap >= min_gap { "≥" } else { "<" }, min_gap), "");
    checks.push(chk("Bar clear spacing", "EN 1992-1-1 8.2(2)", format!("{:.0} mm min", min_gap), format!("{:.0} mm clear", gap), min_gap / gap.max(0.1), gap >= min_gap));
    let link_min = (phi / 4.0).max(6.0);
    let s_max = (20.0 * phi).min(b.min(h)).min(400.0);
    let s_max_r = (s_max / 25.0).floor() * 25.0;
    let link_s = if check { s_in } else { s_max_r };
    t.step(s5, "Links: size and spacing", "EN 1992-1-1 cl. 9.5.3(1), 9.5.3(3)", "φ_{link} ≥ max(6 mm ; φ/4);   s_{cl,max} = min(20·φ ; min(b, h) ; 400 mm)", &format!("φ_link,min = max(6 ; {}/4) = {:.1} mm;  s_max = min({:.0} ; {:.0} ; 400) = {:.0} mm", phi, link_min, 20.0 * phi, b.min(h), s_max), &if check { format!("Provided T{} @ {:.0} mm", ldia, link_s) } else { format!("Use T{} links @ {:.0} mm c/c", ldia, link_s) }, "Spacing reduced to 0.6× within the height h above and below beams/slabs (cl. 9.5.3(4)) and at laps: detail on drawings. Tie requirement for bars beyond 150 mm of a restrained corner bar (cl. 9.5.3(6)) is not checked.");
    checks.push(chk("Link diameter", "EN 1992-1-1 9.5.3(1)", format!("{:.1} mm min", link_min), format!("{} mm", ldia), link_min / ldia, ldia >= link_min));
    checks.push(chk("Link spacing", "EN 1992-1-1 9.5.3(3)", format!("{:.0} mm", link_s), format!("{:.0} mm max", s_max), link_s / s_max, link_s <= s_max + 1e-9));

    let mut obs: Vec<(&str, String)> = vec![("INFO", "Only bending in the plane of h is checked. Biaxial bending and buckling in the other plane are not assessed: verify slenderness about the weak axis separately.".to_string())];
    if slender { obs.push(("INFO", format!("Slender column (λ = {:.0} > λ_lim = {:.0}): second-order effects included by the nominal curvature method.", lam, lam_lim))); }
    if lam > 140.0 { obs.push(("RED", "λ exceeds 140: outside the intended range of the simplified method. Use a second-order analysis.".to_string())); }
    else if lam > 100.0 { obs.push(("AMBER", "λ above 100: consider a more rigorous second-order analysis.".to_string())); }
    if phi_ef == 0.0 && slender { obs.push(("AMBER", "Creep ratio φef is zero: creep effects neglected in the second-order moment.".to_string())); }
    let summary = vec![kv("Slenderness λ / λlim", format!("{:.1} / {:.1}", lam, lam_lim)), kv("Design moment M Ed", format!("{:.1} kNm{}", ev.m_tot, if slender { format!("  (M2 = {:.1})", ev.m2) } else { String::new() })), kv("Moment resistance M Rd", ev.m_rd.map(|(m, _)| format!("{:.1} kNm", m)).unwrap_or("n/a".into())), kv("Axial capacity N Rd,0", format!("{:.0} kN", nrd0)), kv("Longitudinal steel", format!("{}T{}  ({:.0} mm²)", 2 * ev.nface, phi, a_prov)), kv("Links", format!("T{} @ {:.0} c/c", ldia, link_s)), kv("Steel ratio", format!("{:.2} %", 100.0 * a_prov / ac))];
    let _ = ev.d;
    let mut res =     finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: slenderness, nominal curvature second order, N-M interaction by strain compatibility, min/max steel, bar and link detailing.".into(),
        "NOT implemented: biaxial bending, weak-axis slenderness, columns in unbraced frames, side bars in the resistance, shear, fire resistance, confinement detailing.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Column { b, h, cover, link_dia: ldia, bar_dia: phi, n_face: ev.nface });
    res.outputs = vec![out("n_ed", "Axial load N Ed (ULS)", n_ed, "kN"), out("reaction", "Base reaction (ULS)", n_ed, "kN"), out("m_tot", "Design moment incl. second order", ev.m_tot, "kNm")];
    res
}
