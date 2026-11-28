//! Solid timber column in compression with uniaxial or biaxial bending (rules and values as printed in the supplied Eurocode 5 source).
use crate::module::*;
use crate::params;
use crate::timber_data::*;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

pub fn def() -> ModDef {
    ModDef {
        id: "timbercol",
        name: "Timber column",
        title: "Solid timber column (compression and bending)",
        subtitle:
            "Eurocode 5 (values as printed in the supplied source, UK National Annex) · Rectangular section, buckling about both axes",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Design axial compression N d", "kN", "Actions - ULS", 10.0),
            fld("my_ed", "Design moment about y-y (major axis) M y,d", "kNm", "Actions - ULS", 0.35),
            fld("mz_ed", "Design moment about z-z (minor axis) M z,d", "kNm", "Actions - ULS", 0.0),
            fld("b", "Breadth b", "mm", "Geometry", 100.0),
            fld("h", "Depth h", "mm", "Geometry", 100.0),
            fld("lef_y", "Effective length for buckling about y-y", "mm", "Geometry", 3750.0),
            fld("lef_z", "Effective length for buckling about z-z", "mm", "Geometry", 3750.0),
            fld("lef_ltb", "Effective length for lateral-torsional buckling", "mm", "Geometry", 3750.0),
            fld_opt("class", "Strength class (EN 338)", "Material", 2.0, class_options()),
            fld_opt("service", "Service class", "Material", 2.0, service_options()),
            fld_opt("duration", "Load-duration class (shortest in the combination)", "Material", 3.0, duration_options()),
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
    let (n_ed, my, mz, b, h, ley, lez, lelt, cls_v, svc_v, dur_v) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10]);
    let mut errs = vec![];
    let cls = match class_of(cls_v) {
        Some(c) => c,
        None => {
            errs.push("Unknown strength class".to_string());
            &CLASSES[0]
        }
    };
    let (svc, dur) = (svc_v.round() as u8, dur_v.round() as u8);
    if !(1..=3).contains(&svc) {
        errs.push("Service class must be 1, 2 or 3".to_string());
    }
    if !(1..=5).contains(&dur) {
        errs.push("Load-duration class must be 1 to 5".to_string());
    }
    let e005 = match cls.e005 {
        Some(e) => e,
        None => {
            errs.push(format!(
                "E0,05 for class {} is printed inconsistently in the supplied table (D70 reads 6.8). Choose another class.",
                cls.name
            ));
            1.0
        }
    };
    if !errs.is_empty() {
        return fail(errs);
    }
    let km = kmod(svc, dur);
    let gm = GAMMA_M;
    let mut t = Trace::new();
    let (s1, s2, s3, s4) =
        ("1. Design data", "2. Design strengths and stresses", "3. Slenderness and buckling", "4. Combined compression and bending");
    t.step(s1, "Geometry, loads and material", "", "", &format!("N_d = {} kN;  M_y,d = {} kNm;  M_z,d = {} kNm;  b × h = {} × {} mm;  l_ef,y = {} mm;  l_ef,z = {} mm;  strength class {};  service class {};  load-duration class {}", n_ed, my, mz, b, h, ley, lez, cls.name, svc, dur), "", "Eurocode 5 gives no method for effective length: the values are the user's input.");
    t.step(
        s1,
        "Factors",
        "Supplied source: Tables 11.4, 11.5",
        "",
        &format!("γ_M = {:.2};  k_mod = {:.2};  β_c = 0.2 (solid timber);  k_m = 0.7 (rectangular section)", gm, km),
        "",
        "UK National Annex values as printed in the source. Check the Ugandan values.",
    );
    let (kh_y, kh_z) = (kh(h), kh(b));
    let fc0d = km * cls.fc0k / gm;
    let fmyd = kh_y * km * cls.fmk / gm;
    let fmzd = kh_z * km * cls.fmk / gm;
    t.step(
        s2,
        "Design strengths",
        "Supplied source: Eq 11.3, 11.2",
        "f_{c,0,d} = k_{mod}·f_{c,0,k}/γ_M;   f_{m,y,d} = k_{h,y}·k_{mod}·f_{m,k}/γ_M  (depth h);   f_{m,z,d} uses the depth b",
        &format!(
            "f_c,0,d = {} × {} / {} = {:.2};  k_h,y = {:.3}, f_m,y,d = {:.3} × {} × {} / {} = {:.2};  k_h,z = {:.3}, f_m,z,d = {:.2}",
            km, cls.fc0k, gm, fc0d, kh_y, kh_y, km, cls.fmk, gm, fmyd, kh_z, fmzd
        ),
        &format!("f_{{c,0,d}} = {:.2};  f_{{m,y,d}} = {:.2};  f_{{m,z,d}} = {:.2} N/mm^{{2}}", fc0d, fmyd, fmzd),
        "",
    );
    let area = b * h;
    let sc = n_ed * 1e3 / area;
    let smy = 6.0 * my * 1e6 / (b * h * h);
    let smz = 6.0 * mz * 1e6 / (h * b * b);
    t.step(
        s2,
        "Design stresses",
        "Supplied source: Eq 11.39, 11.6, 11.7",
        "σ_{c,0,d} = N_d/A;   σ_{m,y,d} = 6·M_{y,d}/(b·h^{2});   σ_{m,z,d} = 6·M_{z,d}/(h·b^{2})",
        &format!("σ_c,0,d = {}×10^{{3}} / {:.0} = {:.3};  σ_m,y,d = {:.3};  σ_m,z,d = {:.3}", n_ed, area, sc, smy, smz),
        &format!("σ_{{c,0,d}} = {:.3};  σ_{{m,y,d}} = {:.3};  σ_{{m,z,d}} = {:.3} N/mm^{{2}}", sc, smy, smz),
        "",
    );
    let (iy, iz) = (h / 12f64.sqrt(), b / 12f64.sqrt());
    let (ly, lz) = (ley / iy, lez / iz);
    let k_rel = (cls.fc0k / e005).sqrt() / PI;
    let (ry, rz) = (ly * k_rel, lz * k_rel);
    t.step(
        s3,
        "Slenderness",
        "Supplied source: Eq 11.35-11.37",
        "i_y = h/√12;  i_z = b/√12;  λ = l_{ef}/i;   λ_{rel} = (λ/π)·√(f_{c,0,k}/E_{0,05})",
        &format!(
            "i_y = {:.2} mm, λ_y = {:.1}, λ_rel,y = {:.3};  i_z = {:.2} mm, λ_z = {:.1}, λ_rel,z = {:.3}  (E_0,05 = {:.0} N/mm^{{2}})",
            iy, ly, ry, iz, lz, rz, e005
        ),
        &format!("λ_{{rel,y}} = {:.3};  λ_{{rel,z}} = {:.3}", ry, rz),
        "",
    );
    let kc_of = |r: f64| -> (f64, f64) {
        if r <= 0.3 {
            (1.0, 0.0)
        } else {
            let k = 0.5 * (1.0 + 0.2 * (r - 0.3) + r * r);
            (1.0 / (k + (k * k - r * r).sqrt()), k)
        }
    };
    let ((kcy, kyy), (kcz, kzz)) = (kc_of(ry), kc_of(rz));
    let slender = ry > 0.3 || rz > 0.3;
    t.step(s3, "Compression factors", "Supplied source: Eq 11.42-11.45", "k_y = 0.5·[1 + β_c·(λ_{rel,y} − 0.3) + λ_{rel,y}^{2}];   k_{c,y} = 1/[k_y + √(k_y^{2} − λ_{rel,y}^{2})]   (same for z);   k_c = 1 when λ_rel ≤ 0.3", &format!("k_y = {:.3}, k_c,y = {:.4};  k_z = {:.3}, k_c,z = {:.4}", kyy, kcy, kzz, kcz), &format!("k_{{c,y}} = {:.3};  k_{{c,z}} = {:.3}", kcy, kcz), if slender { "λ_rel > 0.3: the column is vulnerable to flexural buckling." } else { "Both λ_rel ≤ 0.3: no reduction for buckling." });
    let kmb = 0.7;
    let mut checks = vec![];
    let (u1, u2);
    if !slender {
        u1 = (sc / fc0d).powi(2) + smy / fmyd + kmb * smz / fmzd;
        u2 = (sc / fc0d).powi(2) + kmb * smy / fmyd + smz / fmzd;
        t.step(s4, "Combined check, λ_rel ≤ 0.3", "Supplied source: Eq 11.46, 11.47", "(σ_{c,0,d}/f_{c,0,d})^{2} + σ_{m,y,d}/f_{m,y,d} + k_m·σ_{m,z,d}/f_{m,z,d} ≤ 1;   (σ_{c,0,d}/f_{c,0,d})^{2} + k_m·σ_{m,y,d}/f_{m,y,d} + σ_{m,z,d}/f_{m,z,d} ≤ 1", &format!("Eq 11.46 = {:.3};  Eq 11.47 = {:.3}", u1, u2), &format!("{:.2} and {:.2}", u1, u2), "");
    } else {
        u1 = sc / (kcy * fc0d) + smy / fmyd + kmb * smz / fmzd;
        u2 = sc / (kcz * fc0d) + kmb * smy / fmyd + smz / fmzd;
        t.step(s4, "Combined check, λ_rel > 0.3", "Supplied source: Eq 11.48, 11.49", "σ_{c,0,d}/(k_{c,y}·f_{c,0,d}) + σ_{m,y,d}/f_{m,y,d} + k_m·σ_{m,z,d}/f_{m,z,d} ≤ 1;   σ_{c,0,d}/(k_{c,z}·f_{c,0,d}) + k_m·σ_{m,y,d}/f_{m,y,d} + σ_{m,z,d}/f_{m,z,d} ≤ 1", &format!("Eq 11.48 = {:.3} / {:.3} + {:.3} / {:.3} + 0.7 × {:.3} / {:.3} = {:.3};  Eq 11.49 = {:.3}", sc, kcy * fc0d, smy, fmyd, smz, fmzd, u1, u2), &format!("{:.2} and {:.2}", u1, u2), "");
    }
    checks.push(chk(
        if slender { "Compression and bending, Eq 11.48" } else { "Compression and bending, Eq 11.46" },
        "Supplied Eq 11.46-11.49",
        format!("{:.3}", u1),
        "1.000".into(),
        u1,
        u1 <= 1.0,
    ));
    checks.push(chk(
        if slender { "Compression and bending, Eq 11.49" } else { "Compression and bending, Eq 11.47" },
        "Supplied Eq 11.46-11.49",
        format!("{:.3}", u2),
        "1.000".into(),
        u2,
        u2 <= 1.0,
    ));
    if slender && my > 0.0 {
        let scrit = 0.78 * b * b * e005 / (h * lelt);
        let lam = (cls.fmk / scrit).sqrt();
        let kcr = kcrit(lam);
        let u3 = (smy / (kcr * fmyd)).powi(2) + sc / (kcz * fc0d);
        t.step(s4, "Lateral-torsional buckling check", "Supplied source: Eq 11.50, 11.19-11.24", "σ_{m,crit} = 0.78·b^{2}·E_{0,05}/(h·l_{ef});   λ_{rel,m} = √(f_{m,k}/σ_{m,crit});   [σ_{m,d}/(k_{crit}·f_{m,d})]^{2} + σ_{c,d}/(k_{c,z}·f_{c,0,d}) ≤ 1", &format!("σ_m,crit = {:.2} N/mm^{{2}};  λ_rel,m = {:.3};  k_crit = {:.3};  = ({:.3}/({:.3} × {:.2}))^{{2}} + {:.3}/({:.4} × {:.2}) = {:.3}", scrit, lam, kcr, smy, kcr, fmyd, sc, kcz, fc0d, u3), &format!("{:.2}", u3), "Equation 11.50 applies when λ_rel > 0.3 and a moment acts about the major axis. The σ_m,crit formula is stated for softwoods.");
        checks.push(chk("Lateral-torsional buckling, Eq 11.50", "Supplied Eq 11.50", format!("{:.3}", u3), "1.000".into(), u3, u3 <= 1.0));
    }
    let kmin = if slender { kcy.min(kcz) } else { 1.0 };
    let n_rd = kmin * fc0d * area / 1e3;
    let mut obs: Vec<(&str, String)> = vec![("AMBER", "k_mod, γ_M and β_c are the UK National Annex values as printed in the supplied source. Confirm the values that apply in Uganda.".to_string()), ("INFO", "Effective lengths are user inputs. Second-order effects of the bending moments, joints, fire and glued-laminated or engineered timber are not checked.".to_string())];
    if cls.name.starts_with('D') {
        obs.push(("AMBER", "The σ_m,crit formula (Eq 11.24) is stated for softwoods; a deciduous class was selected.".to_string()));
    }
    if ry.max(rz) > 3.0 {
        obs.push(("AMBER", "Very high relative slenderness: consider a stockier section.".to_string()));
    }
    let summary = vec![
        kv("Class / k mod", format!("{} / {:.2}", cls.name, km)),
        kv("λ rel,y / λ rel,z", format!("{:.2} / {:.2}", ry, rz)),
        kv("k c,y / k c,z", format!("{:.3} / {:.3}", kcy, kcz)),
        kv("Design strengths f c,0,d / f m,y,d", format!("{:.2} / {:.2} N/mm²", fc0d, fmyd)),
        kv("Axial capacity N Rd (concentric)", format!("{:.2} kN", n_rd)),
        kv("Governing combined check", format!("{:.2}", u1.max(u2))),
    ];
    let mut res = finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: slenderness, buckling factors, compression with bending about both axes (Eq 11.46-11.49), lateral-torsional check (Eq 11.50), axial capacity.".into(),
        "NOT implemented: effective-length determination, second-order moments, joints, fire, glued-laminated and engineered timber, circular or built-up sections.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::Rect { b, h });
    res.outputs = vec![out("n_ed", "Axial load N d", n_ed, "kN")];
    res
}
