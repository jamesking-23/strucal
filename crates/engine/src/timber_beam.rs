//! Simply supported solid timber beam / joist under a uniformly distributed load (Eurocode 5, rules and values as printed in the supplied
//! source: strength classes of EN 338 and the UK National Annex values for k_mod, gamma_M, k_def and deflection limits).
use crate::module::*;
use crate::params;
use crate::timber_data::*;
use crate::trace::Trace;
use serde_json::Value;

pub fn def() -> ModDef {
    ModDef { id: "timber", name: "Timber beam", title: "Solid timber beam / joist", subtitle: "Eurocode 5 (values as printed in the supplied source, UK National Annex) · Simply supported, uniformly distributed load, rectangular section", available: true, has_check_mode: false, fields: vec![
        fld("span", "Span L", "mm", "Geometry", 3400.0), fld("b", "Breadth b", "mm", "Geometry", 63.0), fld("h", "Depth h", "mm", "Geometry", 200.0), fld("lb", "Bearing length", "mm", "Geometry", 100.0), fld("ov", "Overhang beyond the bearing, a", "mm", "Geometry", 0.0),
        fld("h_ef", "Depth at end notch h ef (0 = no notch)", "mm", "End notch", 0.0), fld("notch_x", "Notch: distance x from support line to notch corner", "mm", "End notch", 0.0), fld("notch_i", "Notch inclination i (0 = square notch)", "-", "End notch", 0.0),
        fld_opt("notch_side", "Notch side", "End notch", 1.0, vec![(1.0, "Same side as the support (Fig 11.3a)"), (2.0, "Opposite side to the support (Fig 11.3b)")]),
        fld_opt("class", "Strength class (EN 338)", "Material", 4.0, class_options()), fld_opt("service", "Service class", "Material", 2.0, service_options()),
        fld_opt("duration", "Load-duration class (shortest in the combination)", "Material", 3.0, duration_options()), fld("ksys", "Load-sharing factor k sys (1.0 to 1.1)", "-", "Material", 1.1),
        fld("gk", "Permanent load g k", "kN/m", "Actions", 0.24), fld_opt("sw", "Beam self-weight", "Actions", 2.0, vec![(1.0, "Add from mean density"), (2.0, "Already included in g k")]),
        fld("qk", "Variable load q k", "kN/m", "Actions", 0.9), fld("psi2", "Quasi-permanent factor ψ2 (for creep)", "-", "Actions", 0.3),
        fld_opt("restraint", "Lateral restraint of the compression edge", "Stability", 1.0, vec![(1.0, "Continuously restrained (e.g. decking fixed to the joists)"), (2.0, "Restrained at the supports only")]),
        fld_opt("load_level", "Load applied at", "Stability", 1.0, vec![(1.0, "Compression (top) edge"), (2.0, "Centre of gravity"), (3.0, "Tension (bottom) edge")]),
        fld("defl_ratio", "Deflection limit: span / ratio (net final)", "-", "Serviceability", 250.0),
    ] }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let x = match require(v, &d0, false) { Ok(x) => x, Err(e) => return fail(e) };
    let (span, b, h, lb, ov, h_ef, nx, ni, nside, cls_v, svc_v, dur_v, ksys, gk, sw_opt, qk, psi2, restraint, level, ratio) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11], x[12], x[13], x[14], x[15], x[16], x[17], x[18], x[19]);
    let mut errs = vec![];
    let cls = match class_of(cls_v) { Some(c) => c, None => { errs.push("Unknown strength class".to_string()); &CLASSES[0] } };
    let (svc, dur) = (svc_v.round() as u8, dur_v.round() as u8);
    if !(1..=3).contains(&svc) { errs.push("Service class must be 1, 2 or 3".to_string()); }
    if !(1..=5).contains(&dur) { errs.push("Load-duration class must be 1 to 5".to_string()); }
    if !(1.0..=1.1).contains(&ksys) { errs.push("k_sys must be between 1.0 and 1.1".to_string()); }
    if psi2 > 1.0 { errs.push("ψ2 must not exceed 1".to_string()); }
    if h_ef > 0.0 && h_ef >= h { errs.push("Notch depth h_ef must be less than h (it is the remaining depth)".to_string()); }
    if h_ef > 0.0 && nside.round() as i32 == 1 && nx <= 0.0 { errs.push("Enter the notch distance x for a notch on the support side".to_string()); }
    if !errs.is_empty() { return fail(errs); }
    let km = kmod(svc, dur); let kdf = kdef(svc); let gm = GAMMA_M;
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5, s6, s7) = ("1. Design data", "2. Design strengths", "3. Actions", "4. Bending and lateral stability", "5. Shear", "6. Bearing perpendicular to the grain", "7. Deflection");
    t.step(s1, "Geometry and material", "", "", &format!("L = {} mm;  b × h = {} × {} mm;  bearing length = {} mm;  overhang a = {} mm;  strength class {};  service class {};  load-duration class {}", span, b, h, lb, ov, cls.name, svc, dur), "", "Solid timber, rectangular section, simply supported. Strength and stiffness values from the supplied Table 11.3 (EN 338).");
    t.step(s1, "Factors", "Supplied source: Tables 11.4, 11.5, 11.9; Eq 11.2", "", &format!("γ_M = {:.2};  k_mod = {:.2};  k_sys = {:.2};  k_def = {:.2};  γ_G = {:.2};  γ_Q = {:.2}", gm, km, ksys, kdf, ps.gamma_g, ps.gamma_q), "", "k_mod, γ_M, k_def, k_sys and the deflection limits are the UK National Annex values as printed in the source. Check the Ugandan values.");
    let khv = kh(h);
    let fmd = khv * ksys * km * cls.fmk / gm; let fvd = ksys * km * cls.fvk / gm; let fc90d = ksys * km * cls.fc90k / gm;
    t.step(s2, "Design strengths", "Supplied source: Eq 11.3, 11.2", "X_d = k_mod·k_sys·X_k/γ_M;   k_h = min[(150/h)^{0.2} ; 1.3] for h < 150 mm", &format!("k_h = {:.3};  f_m,d = {:.3} × {} × {} × {} / {} = {:.2};  f_v,d = {} × {} × {} / {} = {:.2};  f_c,90,d = {} × {} × {} / {} = {:.2}", khv, khv, ksys, km, cls.fmk, gm, fmd, ksys, km, cls.fvk, gm, fvd, ksys, km, cls.fc90k, gm, fc90d), &format!("f_{{m,d}} = {:.2} N/mm^{{2}};  f_{{v,d}} = {:.2} N/mm^{{2}};  f_{{c,90,d}} = {:.2} N/mm^{{2}}", fmd, fvd, fc90d), "");
    let g_sw = cls.rhomean * (b * h / 1e6) * 9.81 / 1000.0;
    let g_tot = gk + if sw_opt.round() as i32 == 1 { g_sw } else { 0.0 };
    t.step(s3, "Self-weight and permanent load", "Mean density from the supplied Table 11.3", "g_{sw} = ρ_{mean}·b·h·g", &format!("= {} × ({} × {} × 10^{{-6}}) × 9.81 / 1000 = {:.4} kN/m;  g_total = {} + {} = {:.4} kN/m", cls.rhomean, b, h, g_sw, gk, if sw_opt.round() as i32 == 1 { g_sw } else { 0.0 }, g_tot), &format!("g_{{k}} = {:.4} kN/m", g_tot), if sw_opt.round() as i32 == 1 { "Beam self-weight added." } else { "Self-weight taken as already included in g_k." });
    let l = span / 1000.0;
    let wd = ps.gamma_g * g_tot + ps.gamma_q * qk;
    let (ved, med) = (wd * l / 2.0, wd * l * l / 8.0);
    t.step(s3, "Design ULS load and action effects", "EN 1990 Eq 6.10", "w_d = γ_G·g_k + γ_Q·q_k;   V_d = w_d·L/2;   M_d = w_d·L^{2}/8", &format!("w_d = {:.2} × {:.4} + {:.2} × {} = {:.4} kN/m;  V_d = {:.4} × {:.2}/2;  M_d = {:.4} × {:.2}^{{2}}/8", ps.gamma_g, g_tot, ps.gamma_q, qk, wd, wd, l, wd, l), &format!("V_{{d}} = {:.3} kN;  M_{{d}} = {:.3} kNm", ved, med), "");
    // bending
    let sig_m = 6.0 * med * 1e6 / (b * h * h);
    t.step(s4, "Design bending stress", "Supplied source: Eq 11.6", "σ_{m,d} = M_d / W = 6·M_d / (b·h^{2})", &format!("= 6 × {:.3}×10^{{6}} / ({} × {}^{{2}})", med, b, h), &format!("σ_{{m,d}} = {:.3} N/mm^{{2}}", sig_m), "");
    let mut kcr = 1.0; let mut warn_lt = false;
    if restraint.round() as i32 == 1 {
        t.step(s4, "Lateral buckling", "Supplied source: Section 11.7.4 (Cl. 6.3.3)", "k_{crit} = 1.0 (compression edge restrained throughout the length)", "", "k_{crit} = 1.00", "The check is unnecessary when lateral displacement of the compression edge is prevented along the member and twisting at the supports is prevented.");
    } else {
        let e005 = match cls.e005 { Some(e) => e, None => return fail(vec![format!("E0,05 for class {} is printed inconsistently in the supplied table (D70 reads 6.8). Choose another class or restrain the compression edge.", cls.name)]) };
        let lef = 0.9 * span + if level.round() as i32 == 1 { 2.0 * h } else { 0.0 } - if level.round() as i32 == 3 { 0.5 * h } else { 0.0 };
        let scrit = 0.78 * b * b * e005 / (h * lef);
        let lam = (cls.fmk / scrit).sqrt(); kcr = kcrit(lam);
        t.step(s4, "Lateral buckling", "Supplied source: Table 11.11, Eq 11.18-11.24", "l_{ef} = 0.9·L (+ 2h for load on the compression edge, − 0.5h for load on the tension edge);   σ_{m,crit} = 0.78·b^{2}·E_{0,05}/(h·l_{ef});   λ_{rel,m} = √(f_{m,k}/σ_{m,crit});   k_{crit} = 1 (λ ≤ 0.75), 1.56 − 0.75λ (0.75 < λ ≤ 1.4), 1/λ^{2} (λ > 1.4)",
            &format!("l_ef = 0.9 × {} {} {} = {:.0} mm;  σ_m,crit = 0.78 × {}^{{2}} × {:.0} / ({} × {:.0}) = {:.2} N/mm^{{2}};  λ_rel,m = √({} / {:.2}) = {:.3}", span, if level.round() as i32 == 1 { "+ 2 ×" } else if level.round() as i32 == 3 { "− 0.5 ×" } else { "+ 0 ×" }, h, lef, b, e005, h, lef, scrit, cls.fmk, scrit, lam), &format!("k_{{crit}} = {:.3}", kcr), "The formula for σ_m,crit is stated for softwoods with solid rectangular sections.");
        warn_lt = cls.name.starts_with('D');
    }
    let cap_m = kcr * fmd;
    t.step(s4, "Bending check", "Supplied source: Eq 11.18 (11.4 for a single axis)", "σ_{m,d} ≤ k_{crit}·f_{m,d}", &format!("{:.3} ≤ {:.3} × {:.2} = {:.3} N/mm^{{2}}", sig_m, kcr, fmd, cap_m), &format!("Utilisation = {:.2}", sig_m / cap_m), "");
    let mut checks = vec![chk("Bending (with lateral buckling)", "Supplied Eq 11.4, 11.18", format!("{:.2} N/mm²", sig_m), format!("{:.2} N/mm²", cap_m), sig_m / cap_m, sig_m <= cap_m)];
    // shear
    let notch = h_ef > 0.0;
    let area = b * h;
    let (tau, _kvv, sh_cap) = if !notch {
        let tau = 1.5 * ved * 1e3 / area;
        t.step(s5, "Shear stress", "Supplied source: Eq 11.25, 11.26", "τ_d = 1.5·V_d / A;   τ_d ≤ f_{v,d}", &format!("= 1.5 × {:.3}×10^{{3}} / ({} × {}) = {:.3} N/mm^{{2}};  f_v,d = {:.2} N/mm^{{2}}", ved, b, h, tau, fvd), &format!("τ_{{d}} = {:.3} N/mm^{{2}}", tau), "");
        (tau, 1.0, fvd)
    } else {
        let alpha = h_ef / h;
        let kv = if nside.round() as i32 == 2 { 1.0 } else {
            let num = 5.0 * (1.0 + 1.1 * ni.powf(1.5) / h.sqrt());
            let den = h.sqrt() * ((alpha * (1.0 - alpha)).sqrt() + 0.8 * (nx / h) * (1.0 / alpha - alpha * alpha).sqrt());
            (num / den).min(1.0)
        };
        let tau = 1.5 * ved * 1e3 / (b * h_ef);
        t.step(s5, "Shear at the end notch", "Supplied source: Eq 11.28-11.31 (Example 11.2)", "α = h_{ef}/h;   k_v = min{ 1 ; k_n·(1 + 1.1·i^{1.5}/√h) / [√h·(√(α(1 − α)) + 0.8·(x/h)·√(1/α − α^{2}))] },  k_n = 5 (solid timber);   τ_d = 1.5·V_d/(b·h_{ef}) ≤ k_v·f_{v,d}",
            &format!("α = {} / {} = {:.3};  k_v = {:.3} ({});  τ_d = 1.5 × {:.3}×10^{{3}} / ({} × {}) = {:.3} N/mm^{{2}};  k_v·f_v,d = {:.3} × {:.2}", h_ef, h, alpha, kv, if nside.round() as i32 == 2 { "notch on the opposite side: k_v = 1" } else { "notch on the support side" }, ved, b, h_ef, tau, kv, fvd), &format!("τ_{{d}} = {:.3} N/mm^{{2}};  k_v·f_{{v,d}} = {:.3} N/mm^{{2}}", tau, kv * fvd), "");
        (tau, kv, kv * fvd)
    };
    checks.push(chk(if notch { "Shear at the end notch" } else { "Shear" }, "Supplied Eq 11.25, 11.28", format!("{:.3} N/mm²", tau), format!("{:.3} N/mm²", sh_cap), tau / sh_cap, tau <= sh_cap));
    // bearing
    let f90 = ved * 1e3;
    let sig90 = f90 / (b * lb);
    let kc90 = if ov <= h / 3.0 { ((2.38 - lb / 250.0) * (1.0 + h / (12.0 * lb))).min(4.0) } else { 1.0 };
    t.step(s6, "Bearing at the support", "Supplied source: Eq 11.32, 11.33", "σ_{c,90,d} = F_{90,d}/(b·l);   k_{c,90} = (2.38 − l/250)·(1 + h/(12·l)) ≤ 4.0  (overhang a ≤ h/3), otherwise 1.0;   σ_{c,90,d} ≤ k_{c,90}·f_{c,90,d}", &format!("F_90,d = {:.0} N;  σ = {:.0} / ({} × {}) = {:.3} N/mm^{{2}};  a = {} {} h/3 = {:.1} → k_c,90 = {:.3};  k_c,90·f_c,90,d = {:.3} × {:.2} = {:.2}", f90, f90, b, lb, sig90, ov, if ov <= h / 3.0 { "≤" } else { ">" }, h / 3.0, kc90, kc90, fc90d, kc90 * fc90d), &format!("σ_{{c,90,d}} = {:.3} N/mm^{{2}}", sig90), "Only the arrangement of Fig 11.4 is implemented; other member arrangements (Cl. 6.1.5(4)) use k_c,90 = 1.0.");
    checks.push(chk("Bearing perpendicular to the grain", "Supplied Eq 11.32", format!("{:.3} N/mm²", sig90), format!("{:.3} N/mm²", kc90 * fc90d), sig90 / (kc90 * fc90d), sig90 <= kc90 * fc90d));
    // deflection
    let ei = cls.e0mean * b * h.powi(3) / 12.0; let ea = cls.e0mean * area;
    let u = |w_knm: f64| -> f64 { let wn = w_knm * l * 1e3; 5.0 / 384.0 * wn * span.powi(3) / ei + 2.4 * wn * span / ea };
    let (ug, uq) = (u(g_tot), u(qk));
    let ufg = ug * (1.0 + kdf); let ufq = uq * (1.0 + psi2 * kdf); let ufin = ufg + ufq; let lim = span / ratio;
    t.step(s7, "Instantaneous and final deflection", "Supplied source: Eq 11.8-11.10, Table 6.9 form (bending + shear), Table 11.9", "u_{inst} = 5·W·L^{3}/(384·E_{0,mean}·I) + 2.4·W·L/(E_{0,mean}·A)  (G_{mean} = E_{0,mean}/16);   u_{fin,G} = u_{inst,G}·(1 + k_{def});   u_{fin,Q} = u_{inst,Q}·(1 + ψ_2·k_{def});   u_{fin} = u_{fin,G} + u_{fin,Q} ≤ L/ratio",
        &format!("E_0,mean = {:.0} N/mm^{{2}};  u_inst,G = {:.3} mm;  u_inst,Q = {:.3} mm;  u_fin,G = {:.3} × (1 + {}) = {:.3};  u_fin,Q = {:.3} × (1 + {} × {}) = {:.3}", cls.e0mean, ug, uq, ug, kdf, ufg, uq, psi2, kdf, ufq), &format!("u_{{fin}} = {:.2} mm  {} L/{} = {:.2} mm", ufin, if ufin <= lim { "≤" } else { ">" }, ratio, lim), "Serviceability partial factors are 1.0 (characteristic loads). The limit is a project/National Annex value.");
    checks.push(chk("Final deflection", "Supplied Eq 11.8-11.10", format!("{:.2} mm", ufin), format!("{:.2} mm", lim), ufin / lim, ufin <= lim));
    let mut obs: Vec<(&str, String)> = vec![("AMBER", "k_mod, γ_M, k_def, k_sys and the deflection limit are the UK National Annex values as printed in the supplied source. Confirm the values that apply in Uganda.".to_string()), ("INFO", "Vibration of residential floors, notches on the tension side, holes and members other than simply supported are not checked.".to_string())];
    if warn_lt { obs.push(("AMBER", "The σ_m,crit formula (Eq 11.24) is stated for softwoods; a deciduous class was selected.".to_string())); }
    if h < 150.0 { obs.push(("INFO", format!("Depth below 150 mm: the strength increase factor k_h = {:.3} was applied to the bending strength.", khv))); }
    if sw_opt.round() as i32 == 2 { obs.push(("INFO", "Beam self-weight is not added automatically: make sure it is included in g_k.".to_string())); }
    let summary = vec![kv("Class / service / duration", format!("{} / {} / k_mod {:.2}", cls.name, svc, km)), kv("Design load w d", format!("{:.3} kN/m", wd)), kv("M d / V d", format!("{:.3} kNm / {:.3} kN", med, ved)), kv("Design strengths f m,d / f v,d", format!("{:.2} / {:.2} N/mm²", fmd, fvd)), kv("Bending stress σ m,d", format!("{:.2} N/mm² (k crit {:.2})", sig_m, kcr)), kv("Shear stress τ d", format!("{:.3} N/mm²", tau)), kv("Bearing stress σ c,90,d", format!("{:.3} N/mm² (k c,90 {:.2})", sig90, kc90)), kv("Final deflection", format!("{:.2} mm (limit {:.2})", ufin, lim))];
    let mut res = finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: bending, lateral buckling (compression edge restrained or free), shear with optional end notch, bearing perpendicular to the grain, final deflection with creep.".into(),
        "NOT implemented: floor vibration, glued-laminated and engineered timber, joints, notches on the tension side, holes, columns (see the timber column module), fire design, biaxial bending, point loads.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Rect { b, h });
    res.outputs = vec![out("reaction", "Support reaction (ULS)", ved, "kN"), out("m_ed", "Design moment M d", med, "kNm"), out("v_ed", "Design shear V d", ved, "kN")];
    res
}
