//! Bolted lap/splice connection of plates in tension, bolts in shear and bearing (EN 1993-1-8:2005 Table 3.4, 3.8; EN 1993-1-1 6.2.3).
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

/// Tensile stress area from the ISO 262 coarse pitch: A_s = π/4·(d − 0.9382·P)².
pub fn stress_area(d: f64) -> Option<f64> {
    let p = match d as i64 {
        12 => 1.75,
        16 => 2.0,
        20 => 2.5,
        24 => 3.0,
        27 => 3.0,
        30 => 3.5,
        _ => return None,
    };
    Some(PI / 4.0 * (d - 0.9382 * p).powi(2))
}
fn grade(g: f64) -> Option<(f64, f64, f64, &'static str)> {
    // (fub, fyb, alpha_v in thread, name)
    let t = [
        (4.6, 400.0, 240.0, 0.6, "4.6"),
        (4.8, 400.0, 320.0, 0.5, "4.8"),
        (5.6, 500.0, 300.0, 0.6, "5.6"),
        (5.8, 500.0, 400.0, 0.5, "5.8"),
        (6.8, 600.0, 480.0, 0.5, "6.8"),
        (8.8, 800.0, 640.0, 0.6, "8.8"),
        (10.9, 1000.0, 900.0, 0.5, "10.9"),
    ];
    t.iter().find(|r| (r.0 - g).abs() < 0.01).map(|r| (r.1, r.2, r.3, r.4))
}

pub fn def() -> ModDef {
    ModDef {
        id: "bolted",
        name: "Bolted connection",
        title: "Bolted plate connection in tension (shear and bearing)",
        subtitle: "EN 1993-1-8:2005 · Concentric tension, bolts in shear, non-preloaded (Category A), plate net-section and spacing checks",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Design tensile force N Ed (ULS)", "kN", "Actions", 250.0),
            fld("d", "Bolt diameter (12, 16, 20, 24, 27 or 30)", "mm", "Bolts", 20.0),
            fld("grade", "Bolt grade (4.6 4.8 5.6 5.8 6.8 8.8 10.9)", "-", "Bolts", 8.8),
            fld("planes", "Shear planes per bolt (1 or 2)", "-", "Bolts", 1.0),
            fld("thread", "Shear plane through threads (1) or shank (2)", "-", "Bolts", 1.0),
            fld("n1", "Bolts in a row along the load n 1", "-", "Layout", 3.0),
            fld("n2", "Rows across the width n 2", "-", "Layout", 2.0),
            fld("e1", "End distance e 1", "mm", "Layout", 40.0),
            fld("p1", "Pitch along the load p 1", "mm", "Layout", 70.0),
            fld("e2", "Edge distance e 2", "mm", "Layout", 30.0),
            fld("p2", "Gauge across the load p 2", "mm", "Layout", 60.0),
            fld("w", "Plate width b", "mm", "Plate", 150.0),
            fld("t", "Plate thickness t (thinner outer plate for bearing)", "mm", "Plate", 10.0),
            fld("fy", "Plate yield f y", "MPa", "Plate", 275.0),
            fld("fu", "Plate ultimate f u", "MPa", "Plate", 430.0),
        ],
    }
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) {
        Some(p) => p,
        None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]),
    };
    let x = match require(v, &d0, false) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let (n_ed, d, g, planes, thread, n1, n2, e1, p1, e2, p2, w, tp, fy, fu) =
        (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11], x[12], x[13], x[14]);
    let mut errs = vec![];
    let a_s = match stress_area(d) {
        Some(a) => a,
        None => {
            errs.push("Bolt diameter must be 12, 16, 20, 24, 27 or 30 mm".to_string());
            0.0
        }
    };
    let gr = match grade(g) {
        Some(r) => r,
        None => {
            errs.push("Bolt grade must be one of 4.6, 4.8, 5.6, 5.8, 6.8, 8.8, 10.9".to_string());
            (0.0, 0.0, 0.0, "")
        }
    };
    if !(planes == 1.0 || planes == 2.0) {
        errs.push("Shear planes must be 1 or 2".to_string());
    }
    if !(thread == 1.0 || thread == 2.0) {
        errs.push("Thread flag must be 1 (threads in shear plane) or 2 (shank)".to_string());
    }
    if n1.fract() != 0.0 || n2.fract() != 0.0 || !(1.0..=12.0).contains(&n1) || !(1.0..=6.0).contains(&n2) {
        errs.push("Bolt counts must be whole numbers (n1 up to 12, n2 up to 6)".to_string());
    }
    if !errs.is_empty() {
        return fail(errs);
    }
    let (n1u, n2u) = (n1 as usize, n2 as usize);
    let nb = (n1u * n2u) as f64;
    let (fub, fyb, av_thr, gname) = gr;
    let d0h = d + if d <= 14.0 {
        1.0
    } else if d <= 24.0 {
        2.0
    } else {
        3.0
    };
    let gm2 = 1.25;
    let gm0 = ps.gamma_m0;
    let mut t = Trace::new();
    let (s1, s2, s3, s4) = ("1. Design data", "2. Bolt resistances", "3. Plate resistances", "4. Spacing and edge distances");
    t.step(s1, "Connection data", "", "", &format!("N_{{Ed}} = {} kN;  M{} grade {} (f_{{ub}} = {} MPa, f_{{yb}} = {} MPa);  {} shear plane(s);  {} bolts ({} × {});  e_1 = {};  p_1 = {};  e_2 = {};  p_2 = {};  plate {} × {} mm, f_y = {} MPa, f_u = {} MPa", n_ed, d, gname, fub, fyb, planes, n1u * n2u, n1u, n2u, e1, p1, e2, p2, w, tp, fy, fu), "", "Plates in tension with bolts in a regular grid; Category A bearing-type connection, normal round holes; load assumed shared equally by all bolts.");
    t.step(
        s1,
        "Partial factors",
        "EN 1993-1-8 Table 2.1; EN 1993-1-1 cl. 6.1",
        "",
        &format!("γ_{{M2}} = {:.2};  γ_{{M0}} = {:.2}", gm2, gm0),
        "",
        &format!("Parameter set: {} [{}]. γ_M values are the recommended values.", ps.label, ps.status),
    );
    t.step(
        s1,
        "Hole diameter and stress area",
        "EN 1090-2 cl. 6.6 (nominal clearance); ISO 262",
        "d_0 = d + clearance (1 mm up to M14, 2 mm M16-M24, 3 mm from M27);   A_s = π/4·(d − 0.9382·P)^{2};   A = π·d^{2}/4",
        &format!("d_0 = {} + {} = {} mm;  A_s = {:.1} mm^{{2}};  A = {:.1} mm^{{2}}", d, d0h - d, d0h, a_s, PI * d * d / 4.0),
        &format!("d_{{0}} = {} mm;  A_s = {:.1} mm^{{2}}", d0h, a_s),
        "",
    );
    // shear
    let (av, a_use, plane) = if thread == 1.0 { (av_thr, a_s, "threaded portion") } else { (0.6, PI * d * d / 4.0, "unthreaded shank") };
    let fv = planes * av * fub * a_use / gm2;
    t.step(s2, "Shear resistance per bolt", "EN 1993-1-8 Table 3.4", "F_{v,Rd} = n_s·α_v·f_{ub}·A/γ_{M2};   α_v = 0.6 (4.6, 5.6, 8.8) or 0.5 (4.8, 5.8, 6.8, 10.9) with threads in the shear plane; 0.6 for the unthreaded shank", &format!("= {} × {} × {} × {:.1} / {:.2}  (shear plane through {})", planes, av, fub, a_use, gm2, plane), &format!("F_{{v,Rd}} = {:.1} kN per bolt", fv / 1e3), "");
    let lj = (n1u as f64 - 1.0) * p1;
    let beta_lf = if lj > 15.0 * d { (1.0 - (lj - 15.0 * d) / (200.0 * d)).clamp(0.75, 1.0) } else { 1.0 };
    t.step(
        s2,
        "Long joint reduction",
        "EN 1993-1-8 cl. 3.8",
        "L_j = (n_1 − 1)·p_1;   β_{Lf} = 1 − (L_j − 15d)/(200d),  0.75 ≤ β_{Lf} ≤ 1.0  (L_j > 15d)",
        &format!("L_j = {:.0} mm;  15d = {:.0} mm", lj, 15.0 * d),
        &format!("β_{{Lf}} = {:.3}", beta_lf),
        "",
    );
    let fv_grp = nb * fv * beta_lf;
    // bearing: end bolts + edge distance for all bolts (conservative)
    let ad = e1 / (3.0 * d0h);
    let ab = ad.min(fub / fu).min(1.0);
    let k1 = (2.8 * e2 / d0h - 1.7).min(1.4 * p2 / d0h - 1.7).min(2.5);
    let fb = k1 * ab * fu * d * tp / gm2;
    t.step(s2, "Bearing resistance per bolt (end bolt, edge row)", "EN 1993-1-8 Table 3.4", "α_d = e_1/(3·d_0);   α_b = min(α_d ; f_{ub}/f_u ; 1.0);   k_1 = min(2.8·e_2/d_0 − 1.7 ; 1.4·p_2/d_0 − 1.7 ; 2.5);   F_{b,Rd} = k_1·α_b·f_u·d·t/γ_{M2}", &format!("α_d = {}/(3 × {}) = {:.3};  α_b = min({:.3} ; {}/{} ; 1.0) = {:.3};  k_1 = min({:.3} ; {:.3} ; 2.5) = {:.3};  F_b,Rd = {:.3} × {:.3} × {} × {} × {} / {:.2}", e1, d0h, ad, ad, fub, fu, ab, 2.8 * e2 / d0h - 1.7, 1.4 * p2 / d0h - 1.7, k1, k1, ab, fu, d, tp, gm2), &format!("F_{{b,Rd}} = {:.1} kN per bolt", fb / 1e3), "The end-bolt α_d and the edge-row k_1 are applied to every bolt (conservative). For double shear the thickness of the thinner outer plate governs.");
    let fb_grp = nb * fb;
    // plate
    let a_gross = w * tp;
    let npl = a_gross * fy / gm0;
    let a_net = (w - n2 * d0h) * tp;
    let nu = 0.9 * a_net * fu / gm2;
    t.step(s3, "Plate tension resistance", "EN 1993-1-1 cl. 6.2.3(2), Eq 6.6, 6.7", "N_{pl,Rd} = A·f_y/γ_{M0};   N_{u,Rd} = 0.9·A_{net}·f_u/γ_{M2},   A_{net} = (b − n_2·d_0)·t", &format!("A = {} × {} = {:.0} mm^{{2}};  N_pl,Rd = {:.0} × {} / {:.2};  A_net = ({} − {} × {}) × {} = {:.0} mm^{{2}};  N_u,Rd = 0.9 × {:.0} × {} / {:.2}", w, tp, a_gross, a_gross, fy, gm0, w, n2u, d0h, tp, a_net, a_net, fu, gm2), &format!("N_{{pl,Rd}} = {:.1} kN;  N_{{u,Rd}} = {:.1} kN", npl / 1e3, nu / 1e3), "Net section taken across one row of n_2 holes. Staggered holes and block tearing are not checked.");
    // spacings
    let (e1m, e2m, p1m, p2m) = (1.2 * d0h, 1.2 * d0h, 2.2 * d0h, 2.4 * d0h);
    let (emax, p1max) = (4.0 * tp + 40.0, (14.0 * tp).min(200.0));
    t.step(
        s4,
        "Minimum and maximum distances",
        "EN 1993-1-8 Table 3.3",
        "e_1, e_2 ≥ 1.2·d_0;   p_1 ≥ 2.2·d_0;   p_2 ≥ 2.4·d_0;   e_1, e_2 ≤ 4t + 40;   p_1 ≤ min(14t ; 200)",
        &format!(
            "e_1 ≥ {:.1} (have {});  e_2 ≥ {:.1} (have {});  p_1 ≥ {:.1} (have {});  p_2 ≥ {:.1} (have {});  e ≤ {:.0};  p_1 ≤ {:.0}",
            e1m, e1, e2m, e2, p1m, p1, p2m, p2, emax, p1max
        ),
        "",
        "Maximum distances apply to members exposed to weather or corrosion.",
    );
    let ok_min = e1 >= e1m - 1e-9 && e2 >= e2m - 1e-9 && (n1u < 2 || p1 >= p1m - 1e-9) && (n2u < 2 || p2 >= p2m - 1e-9);
    let ok_max = e1 <= emax + 1e-9 && e2 <= emax + 1e-9 && (n1u < 2 || p1 <= p1max + 1e-9);
    let fits = (n2 - 1.0) * p2 + 2.0 * e2 <= w + 1e-9;
    let mut checks = vec![];
    checks.push(chk(
        "Bolt shear (group)",
        "EN 1993-1-8 Table 3.4",
        format!("{} kN", n_ed),
        format!("{:.1} kN", fv_grp / 1e3),
        n_ed * 1e3 / fv_grp,
        n_ed * 1e3 <= fv_grp,
    ));
    checks.push(chk(
        "Bolt bearing (group)",
        "EN 1993-1-8 Table 3.4",
        format!("{} kN", n_ed),
        format!("{:.1} kN", fb_grp / 1e3),
        n_ed * 1e3 / fb_grp,
        n_ed * 1e3 <= fb_grp,
    ));
    checks.push(chk(
        "Plate gross section yielding",
        "EN 1993-1-1 6.2.3(2)a",
        format!("{} kN", n_ed),
        format!("{:.1} kN", npl / 1e3),
        n_ed * 1e3 / npl,
        n_ed * 1e3 <= npl,
    ));
    checks.push(chk(
        "Plate net section",
        "EN 1993-1-1 6.2.3(2)b",
        format!("{} kN", n_ed),
        format!("{:.1} kN", nu / 1e3),
        n_ed * 1e3 / nu,
        n_ed * 1e3 <= nu,
    ));
    checks.push(chk(
        "Minimum edge and spacing",
        "EN 1993-1-8 Table 3.3",
        "min distances".into(),
        if ok_min { "satisfied".into() } else { "violated".into() },
        if ok_min { 0.5 } else { 1.5 },
        ok_min,
    ));
    checks.push(chk(
        "Maximum edge and spacing",
        "EN 1993-1-8 Table 3.3",
        "max distances".into(),
        if ok_max { "satisfied".into() } else { "exceeded".into() },
        if ok_max { 0.5 } else { 1.5 },
        ok_max,
    ));
    checks.push(chk(
        "Bolts fit within plate width",
        "Geometry",
        format!("{:.0} mm", (n2 - 1.0) * p2 + 2.0 * e2),
        format!("{:.0} mm", w),
        ((n2 - 1.0) * p2 + 2.0 * e2) / w,
        fits,
    ));
    let obs = vec![("AMBER", "Block tearing (EN 1993-1-8 cl. 3.10.2), staggered holes, eccentric loading of the bolt group and prying are not checked.".to_string()), ("INFO", "Category A (bearing) connection: preloading, slip resistance and fatigue are not assessed. Bolt shear resistance assumes the load is shared equally by all bolts.".to_string())];
    let summary = vec![
        kv("Bolts", format!("{} × M{} grade {}", n1u * n2u, d, gname)),
        kv("F v,Rd per bolt", format!("{:.1} kN", fv / 1e3)),
        kv("F b,Rd per bolt", format!("{:.1} kN", fb / 1e3)),
        kv("Group shear / bearing", format!("{:.1} / {:.1} kN", fv_grp / 1e3, fb_grp / 1e3)),
        kv("Plate N pl,Rd / N u,Rd", format!("{:.1} / {:.1} kN", npl / 1e3, nu / 1e3)),
        kv("Joint length L j", format!("{:.0} mm (β Lf = {:.3})", lj, beta_lf)),
    ];
    finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: bolt shear (with long-joint reduction), bearing, plate gross and net section, minimum and maximum distances.".into(),
        "NOT implemented: bolts in tension and combined shear-tension, preloaded (Category B/C) connections, block tearing, eccentric bolt groups, prying, connected angle and cleat details, fatigue.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Bolts { w, n1: n1u as u32, n2: n2u as u32, e1, p1, e2, p2, d })
}
