//! Pin-ended hot-rolled I/H steel column in axial compression (EN 1993-1-1:2005 cl. 6.2.4, 6.3.1): flexural buckling about both axes.
use crate::module::*;
use crate::params;
use crate::steel_beam::props;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

const E: f64 = 210_000.0;

pub fn def() -> ModDef {
    ModDef {
        id: "steelcol",
        name: "Steel column",
        title: "Steel I/H column verification (axial compression)",
        subtitle: "EN 1993-1-1:2005 · Hot-rolled doubly symmetric section, flexural buckling about both axes",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Axial compression N Ed (ULS)", "kN", "Actions - ULS", 800.0),
            fld("len", "Column length L (system length)", "mm", "Geometry", 4000.0),
            fld("ky", "Effective length factor, y-y (strong axis)", "-", "Geometry", 1.0),
            fld("kz", "Effective length factor, z-z (weak axis)", "-", "Geometry", 1.0),
            fld("h", "Section depth h", "mm", "Section (HEB 200 shown)", 200.0),
            fld("b", "Flange width b", "mm", "Section (HEB 200 shown)", 200.0),
            fld("tw", "Web thickness t w", "mm", "Section (HEB 200 shown)", 9.0),
            fld("tf", "Flange thickness t f", "mm", "Section (HEB 200 shown)", 15.0),
            fld("r", "Root radius r", "mm", "Section (HEB 200 shown)", 18.0),
            fld("fy", "Yield strength f y", "MPa", "Materials", 275.0),
        ],
    }
}

/// Imperfection factor and curve name per EN 1993-1-1 Table 6.2 (rolled I sections).
pub fn curve(h: f64, b: f64, tf: f64, axis_y: bool) -> (&'static str, f64) {
    let alpha = |c: &str| match c {
        "a0" => 0.13,
        "a" => 0.21,
        "b" => 0.34,
        "c" => 0.49,
        _ => 0.76,
    };
    let c = if tf > 100.0 {
        "d"
    } else if h / b > 1.2 {
        if tf <= 40.0 {
            if axis_y {
                "a"
            } else {
                "b"
            }
        } else if axis_y {
            "b"
        } else {
            "c"
        }
    } else if axis_y {
        "b"
    } else {
        "c"
    };
    (c, alpha(c))
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
    let (n_ed, len, ky, kz, h, b, tw, tf, r, fy) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9]);
    let mut errs = vec![];
    if 2.0 * tf >= h {
        errs.push("2·tf must be less than h".to_string());
    }
    if tw >= b {
        errs.push("tw must be less than b".to_string());
    }
    if r > (b - tw) / 2.0 || r > (h - 2.0 * tf) / 2.0 {
        errs.push("Root radius is too large for the section".to_string());
    }
    if !(235.0..=460.0).contains(&fy) {
        errs.push("fy must be between 235 and 460 MPa (S235-S460)".to_string());
    }
    if ky < 0.5 || kz < 0.5 || ky > 2.5 || kz > 2.5 {
        errs.push("Effective length factors must be between 0.5 and 2.5".to_string());
    }
    if !errs.is_empty() {
        return fail(errs);
    }
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = (
        "1. Design data",
        "2. Section properties",
        "3. Section classification",
        "4. Cross-section resistance",
        "5. Flexural buckling resistance",
    );
    t.step(s1, "Geometry, load and material", "", "", &format!("N_{{Ed}} = {} kN;  L = {} mm;  k_y = {};  k_z = {};  h = {} mm;  b = {} mm;  t_w = {} mm;  t_f = {} mm;  r = {} mm;  f_y = {} MPa", n_ed, len, ky, kz, h, b, tw, tf, r, fy), "", "Doubly symmetric rolled I/H section, axially loaded, no transverse loads or end moments. Effective length factors are the user's inputs.");
    t.step(
        s1,
        "Partial factors",
        "EN 1993-1-1 cl. 6.1(1)",
        "",
        &format!("γ_{{M0}} = {:.2};  γ_{{M1}} = {:.2};  E = 210000 MPa", ps.gamma_m0, ps.gamma_m1),
        "",
        &format!("Parameter set: {} [{}]", ps.label, ps.status),
    );
    let p = props(h, b, tw, tf, r);
    let hw = h - 2.0 * tf;
    t.step(
        s2,
        "Area and second moments of area",
        "Section geometry (rectangles + root fillets)",
        "A = 2·b·t_f + h_w·t_w + (4 − π)·r^{2};   I_y, I_z with root fillets",
        &format!("h_w = {} − 2 × {} = {:.1} mm", h, tf, hw),
        &format!("A = {:.0} mm^{{2}};  I_{{y}} = {:.3}×10^{{6}} mm^{{4}};  I_{{z}} = {:.3}×10^{{6}} mm^{{4}}", p.a, p.iy / 1e6, p.iz / 1e6),
        "Compare against the manufacturer's table.",
    );
    let eps = (235.0 / fy).sqrt();
    let cf = (b - tw - 2.0 * r) / 2.0;
    let cw = hw - 2.0 * r;
    let cl = |ratio: f64, l1: f64, l2: f64, l3: f64| {
        if ratio <= l1 * eps {
            1
        } else if ratio <= l2 * eps {
            2
        } else if ratio <= l3 * eps {
            3
        } else {
            4
        }
    };
    let cls_f = cl(cf / tf, 9.0, 10.0, 14.0);
    let cls_w = cl(cw / tw, 33.0, 38.0, 42.0);
    let cls = cls_f.max(cls_w);
    t.step(
        s3,
        "Slenderness of flange outstand and web in compression",
        "EN 1993-1-1 Table 5.2",
        "ε = √(235/f_y);   flange: c_f/t_f ≤ 9ε / 10ε / 14ε;   web (compression): c_w/t_w ≤ 33ε / 38ε / 42ε",
        &format!("ε = {:.3};  c_f/t_f = {:.1}/{} = {:.2};  c_w/t_w = {:.1}/{} = {:.2}", eps, cf, tf, cf / tf, cw, tw, cw / tw),
        &format!("Flange class {};  web class {};  section class {}", cls_f, cls_w, cls),
        "",
    );
    if cls == 4 {
        return fail(vec![format!(
            "Class 4 section (flange class {}, web class {}): effective area is not implemented. Choose a more compact section.",
            cls_f, cls_w
        )]);
    }
    let ncrd = p.a * fy / ps.gamma_m0;
    t.step(
        s4,
        "Design compression resistance of the cross-section",
        "EN 1993-1-1 cl. 6.2.4(1), Eq 6.10",
        "N_{c,Rd} = A·f_y/γ_{M0}   (class 1, 2 and 3)",
        &format!("= {:.0} × {} / {:.2}", p.a, fy, ps.gamma_m0),
        &format!("N_{{c,Rd}} = {:.0} kN", ncrd / 1e3),
        "",
    );
    let mut checks = vec![chk(
        "Cross-section compression",
        "EN 1993-1-1 6.2.4",
        format!("{} kN", n_ed),
        format!("{:.0} kN", ncrd / 1e3),
        n_ed * 1e3 / ncrd,
        n_ed * 1e3 <= ncrd,
    )];
    let mut rows = vec![];
    for (axis, i_ax, k, is_y) in [("y-y", p.iy, ky, true), ("z-z", p.iz, kz, false)] {
        let lcr = k * len;
        let ncr = PI * PI * E * i_ax / (lcr * lcr);
        let lam = (p.a * fy / ncr).sqrt();
        let (cn, al) = curve(h, b, tf, is_y);
        let phi = 0.5 * (1.0 + al * (lam - 0.2) + lam * lam);
        let chi_raw = 1.0 / (phi + (phi * phi - lam * lam).sqrt());
        let skip = lam <= 0.2 || n_ed * 1e3 / ncr <= 0.04;
        let chi = if skip { 1.0 } else { chi_raw.min(1.0) };
        let nb = chi * p.a * fy / ps.gamma_m1;
        t.step(
            s5,
            &format!("Elastic critical force and slenderness, {} axis", axis),
            "EN 1993-1-1 cl. 6.3.1.2, 6.3.1.3",
            "L_{cr} = k·L;   N_{cr} = π^{2}·E·I/L_{cr}^{2};   λ̄ = √(A·f_y/N_{cr})",
            &format!(
                "L_cr = {} × {} = {:.0} mm;  N_cr = π² × 210000 × {:.3}×10^{{6}} / {:.0}^{{2}} = {:.0} kN",
                k,
                len,
                lcr,
                i_ax / 1e6,
                lcr,
                ncr / 1e3
            ),
            &format!("λ̄_{{{}}} = {:.3}", axis, lam),
            "",
        );
        t.step(
            s5,
            &format!("Reduction factor, {} axis", axis),
            "EN 1993-1-1 cl. 6.3.1.2, Eq 6.49, Tables 6.1 and 6.2",
            "Φ = 0.5·[1 + α·(λ̄ − 0.2) + λ̄^{2}];   χ = 1/[Φ + √(Φ^{2} − λ̄^{2})] ≤ 1",
            &format!(
                "h/b = {:.2}, t_f = {} mm → buckling curve {} (α = {});  Φ = 0.5 × [1 + {} × ({:.3} − 0.2) + {:.3}^{{2}}] = {:.3}",
                h / b,
                tf,
                cn,
                al,
                al,
                lam,
                lam,
                phi
            ),
            &format!(
                "χ_{{{}}} = {:.3}{}",
                axis,
                chi_raw.min(1.0),
                if skip { "  (buckling check not needed: λ̄ ≤ 0.2 or N_Ed/N_cr ≤ 0.04; χ = 1.0 used)" } else { "" }
            ),
            "",
        );
        t.step(
            s5,
            &format!("Design buckling resistance, {} axis", axis),
            "EN 1993-1-1 cl. 6.3.1.1, Eq 6.47",
            "N_{b,Rd} = χ·A·f_y/γ_{M1}",
            &format!("= {:.3} × {:.0} × {} / {:.2}", chi, p.a, fy, ps.gamma_m1),
            &format!("N_{{b,Rd,{}}} = {:.0} kN", axis, nb / 1e3),
            "",
        );
        checks.push(chk(
            &format!("Flexural buckling {}", axis),
            "EN 1993-1-1 6.3.1.1",
            format!("{} kN", n_ed),
            format!("{:.0} kN", nb / 1e3),
            n_ed * 1e3 / nb,
            n_ed * 1e3 <= nb,
        ));
        rows.push((axis, lam, chi, nb, ncr));
    }
    let nb_min = rows.iter().map(|r| r.3).fold(f64::MAX, f64::min);
    let obs = vec![("INFO", "Torsional and flexural-torsional buckling are not checked (they rarely govern for doubly symmetric rolled I/H sections). Bending moments, transverse loads and combined axial-bending checks (cl. 6.3.3) are not included.".to_string()), ("INFO", "Section properties are computed from the entered dimensions and should be compared with the manufacturer's table.".to_string())];
    let summary = vec![
        kv("Section class", format!("{} (flange {}, web {})", cls, cls_f, cls_w)),
        kv("A / I y / I z", format!("{:.0} mm² / {:.2} / {:.2} ×10⁶ mm⁴", p.a, p.iy / 1e6, p.iz / 1e6)),
        kv("N c,Rd", format!("{:.0} kN", ncrd / 1e3)),
        kv("λ̄ y / λ̄ z", format!("{:.3} / {:.3}", rows[0].1, rows[1].1)),
        kv("χ y / χ z", format!("{:.3} / {:.3}", rows[0].2, rows[1].2)),
        kv("N b,Rd (governing)", format!("{:.0} kN", nb_min / 1e3)),
        kv("N Ed / N b,Rd", format!("{:.2}", n_ed * 1e3 / nb_min)),
    ];
    finish(t, checks, &ps.status, summary, vec![], obs, vec![
        "Implemented: classification (classes 1-3), cross-section compression resistance, flexural buckling about both axes for rolled I/H sections.".into(),
        "NOT implemented: class 4 sections, welded sections, combined axial and bending, torsional-flexural buckling, section selection from tables, fire.".into(),
        "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into()],
        Drawing::ISection { h, b, tw, tf })
}
