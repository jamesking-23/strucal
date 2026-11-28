//! Fillet weld group under a concentric force (EN 1993-1-8:2005 cl. 4.5.3): directional method and simplified method.
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;

pub fn def() -> ModDef {
    ModDef { id: "weld", name: "Fillet weld", title: "Fillet weld group verification", subtitle: "EN 1993-1-8:2005 · Concentric force on equal-leg fillet welds, directional and simplified methods", available: true, has_check_mode: false, fields: vec![
        fld("lw", "Total effective weld length L w (sum of all welds)", "mm", "Weld", 300.0), fld("a", "Throat thickness a", "mm", "Weld", 5.0), fld("tmin", "Thinner connected plate t", "mm", "Weld", 10.0),
        fld("f_perp", "Force perpendicular to the weld axis F ⊥", "kN", "Actions", 40.0), fld("f_par", "Force parallel to the weld axis F ∥", "kN", "Actions", 60.0),
        fld("fy", "Weaker part: yield f y", "MPa", "Material", 275.0), fld("fu", "Weaker part: ultimate f u", "MPa", "Material", 430.0),
    ] }
}

pub fn beta_w(fy: f64) -> f64 { if fy <= 235.0 { 0.8 } else if fy <= 275.0 { 0.85 } else if fy <= 355.0 { 0.9 } else { 1.0 } }

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) { Some(p) => p, None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]) };
    let fperp = num(v, "f_perp").unwrap_or(0.0); let fpar = num(v, "f_par").unwrap_or(0.0);
    if fperp < 0.0 || fpar < 0.0 || (fperp == 0.0 && fpar == 0.0) { return fail(vec!["Enter at least one force greater than zero (magnitudes, kN)".into()]); }
    let mut v2 = v.clone(); v2["f_perp"] = 1.0.into(); v2["f_par"] = 1.0.into();
    let x = match require(&v2, &d0, false) { Ok(x) => x, Err(e) => return fail(e) };
    let (lw, a, tmin, fy, fu) = (x[0], x[1], x[2], x[5], x[6]);
    let mut errs = vec![];
    if !(235.0..=460.0).contains(&fy) { errs.push("fy must be between 235 and 460 MPa (S235-S460)".to_string()); }
    if !(360.0..=570.0).contains(&fu) { errs.push("fu must be between 360 and 570 MPa".to_string()); }
    if !errs.is_empty() { return fail(errs); }
    let bw = beta_w(fy); let gm2 = 1.25;
    let mut t = Trace::new();
    let (s1, s2, s3) = ("1. Design data", "2. Directional method", "3. Simplified method");
    t.step(s1, "Weld and loading", "", "", &format!("L_w = {} mm;  a = {} mm;  F_⊥ = {} kN;  F_∥ = {} kN;  f_y = {} MPa;  f_u = {} MPa", lw, a, fperp, fpar, fy, fu), "", "Equal-leg fillet welds loaded through the weld group centroid (no eccentricity). Forces are magnitudes in the plane of the connected plates.");
    t.step(s1, "Correlation factor and partial factor", "EN 1993-1-8 Table 4.1, Table 2.1", "β_w by steel grade: S235 0.80;  S275 0.85;  S355 0.90;  S420/S460 1.00;   γ_{M2} = 1.25", &format!("f_y = {} MPa → β_w = {}", fy, bw), &format!("β_{{w}} = {:.2};  γ_{{M2}} = {:.2}", bw, gm2), &format!("Parameter set: {} [{}]", ps.label, ps.status));
    let area = a * lw;
    let sig = fperp * 1e3 / (2f64.sqrt() * area); let tau_perp = sig; let tau_par = fpar * 1e3 / area;
    let eq = (sig * sig + 3.0 * (tau_perp * tau_perp + tau_par * tau_par)).sqrt();
    let lim1 = fu / (bw * gm2); let lim2 = 0.9 * fu / gm2;
    t.step(s2, "Stresses on the throat section", "EN 1993-1-8 cl. 4.5.3.2(3), Fig. 4.5", "σ_⊥ = τ_⊥ = F_⊥/(√2·a·L_w);   τ_∥ = F_∥/(a·L_w)", &format!("A_w = {} × {} = {:.0} mm^{{2}};  σ_⊥ = τ_⊥ = {} × 10^{{3}} / (√2 × {:.0}) = {:.1} MPa;  τ_∥ = {} × 10^{{3}} / {:.0} = {:.1} MPa", a, lw, area, fperp, area, sig, fpar, area, tau_par), &format!("σ_{{⊥}} = {:.1} MPa;  τ_{{∥}} = {:.1} MPa", sig, tau_par), "The perpendicular force is assumed to act at 45° on equal-leg welds.");
    t.step(s2, "Directional method criteria", "EN 1993-1-8 cl. 4.5.3.2(4), Eq 4.1", "√[σ_⊥^{2} + 3·(τ_⊥^{2} + τ_∥^{2})] ≤ f_u/(β_w·γ_{M2});   σ_⊥ ≤ 0.9·f_u/γ_{M2}", &format!("= √[{:.1}^{{2}} + 3 × ({:.1}^{{2}} + {:.1}^{{2}})] = {:.1} MPa;  limit = {} / ({} × {:.2}) = {:.1} MPa;  σ_⊥ = {:.1} vs 0.9 × {} / {:.2} = {:.1} MPa", sig, tau_perp, tau_par, eq, fu, bw, gm2, lim1, sig, fu, gm2, lim2), &format!("{:.1} MPa  {} {:.1} MPa", eq, if eq <= lim1 { "≤" } else { ">" }, lim1), "");
    let fres = (fperp * fperp + fpar * fpar).sqrt();
    let fvwd = fu / 3f64.sqrt() / (bw * gm2);
    let fw_rd = fvwd * a; let fw_ed = fres * 1e3 / lw;
    t.step(s3, "Design shear strength and resistance per unit length", "EN 1993-1-8 cl. 4.5.3.3(2), Eq 4.4, 4.3", "f_{vw,d} = (f_u/√3)/(β_w·γ_{M2});   F_{w,Rd} = f_{vw,d}·a;   F_{w,Ed} = F/L_w (resultant)", &format!("f_vw,d = ({}/√3)/({} × {:.2}) = {:.1} MPa;  F_w,Rd = {:.1} × {} = {:.0} N/mm;  F = √({}^{{2}} + {}^{{2}}) = {:.1} kN;  F_w,Ed = {:.1}×10^{{3}} / {} = {:.0} N/mm", fu, bw, gm2, fvwd, fvwd, a, fw_rd, fperp, fpar, fres, fres, lw, fw_ed), &format!("F_{{w,Ed}} = {:.0} N/mm  {} F_{{w,Rd}} = {:.0} N/mm", fw_ed, if fw_ed <= fw_rd { "≤" } else { ">" }, fw_rd), "The simplified method is valid for any direction of the force.");
    let lmin = 30.0f64.max(6.0 * a);
    let lj_ok = lw <= 150.0 * a;
    let mut checks = vec![];
    checks.push(chk("Directional method (combined)", "EN 1993-1-8 4.5.3.2(4)", format!("{:.1} MPa", eq), format!("{:.1} MPa", lim1), eq / lim1, eq <= lim1));
    checks.push(chk("Normal stress on throat", "EN 1993-1-8 4.5.3.2(4)", format!("{:.1} MPa", sig), format!("{:.1} MPa", lim2), sig / lim2, sig <= lim2));
    checks.push(chk("Simplified method", "EN 1993-1-8 4.5.3.3", format!("{:.0} N/mm", fw_ed), format!("{:.0} N/mm", fw_rd), fw_ed / fw_rd, fw_ed <= fw_rd));
    checks.push(chk("Minimum effective length", "EN 1993-1-8 4.5.1(1)", format!("{:.0} mm min", lmin), format!("{:.0} mm", lw), lmin / lw, lw >= lmin));
    let mut obs = vec![("INFO", "Weld group eccentricity, weld sequences, and stresses in the connected parts (e.g. punching of a flange) are not checked.".to_string())];
    if a > 0.7 * tmin { obs.push(("AMBER", format!("Throat a = {} mm exceeds 0.7 × t_min = {:.1} mm: the weld may be larger than the plate can develop.", a, 0.7 * tmin))); }
    if !lj_ok { obs.push(("AMBER", "Weld longer than 150a: the long-joint reduction of EN 1993-1-8 cl. 4.11 is not applied here; reduce the design resistance.".to_string())); }
    let summary = vec![kv("Throat / length", format!("a = {} mm, L w = {} mm", a, lw)), kv("Resultant force", format!("{:.1} kN", fres)), kv("F w,Ed / F w,Rd", format!("{:.0} / {:.0} N/mm", fw_ed, fw_rd)), kv("Directional stress / limit", format!("{:.1} / {:.1} MPa", eq, lim1)), kv("β w", format!("{:.2}", bw))];
    finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: directional method and simplified method for an equal-leg fillet weld group loaded concentrically, minimum length.".into(),
        "NOT implemented: eccentric weld groups, butt and partial-penetration welds, long-joint reduction, fatigue, weld between thin plates and hollow sections.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Weld { lw, a })
}
