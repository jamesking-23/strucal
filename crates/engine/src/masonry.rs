//! Unreinforced masonry wall under vertical load, single leaf, per metre run (EN 1996-1-1:2005 cl. 3.6.1, 5.5.1, 6.1.2
//! and Annex G): characteristic compressive strength, effective height and thickness, slenderness limit, capacity
//! reduction for eccentricity at the top/bottom (Eq 6.4) and at mid-height including creep (Eq 6.6-6.9, G.1-G.4).
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;

pub fn def() -> ModDef {
    ModDef {
        id: "masonry",
        name: "Masonry wall",
        title: "Unreinforced masonry wall (vertical load)",
        subtitle: "EN 1996-1-1:2005 · Single-leaf loadbearing wall, eccentric vertical load, capacity reduction Φ at top and mid-height",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Design vertical load at the top N Ed (ULS)", "kN/m", "Actions - ULS", 180.0),
            fld("e_load", "Eccentricity of the load at the top e", "mm", "Actions - ULS", 15.0),
            fld("t", "Wall thickness t", "mm", "Geometry", 215.0),
            fld("h", "Clear storey height h", "mm", "Geometry", 2800.0),
            fld_opt(
                "rho_n",
                "Restraint (reduction factor ρn, cl. 5.5.1.2)",
                "Geometry",
                0.75,
                vec![
                    (0.75, "ρ2 = 0.75 · top and bottom restrained by concrete floors"),
                    (1.0, "ρ2 = 1.0 · restrained by timber floors or unrestrained edge"),
                ],
            ),
            fld("fb", "Normalised unit compressive strength f b", "MPa", "Materials", 10.0),
            fld_opt(
                "fm",
                "Mortar compressive strength f m (general purpose)",
                "Materials",
                6.0,
                vec![(4.0, "M4 (f m = 4 MPa)"), (6.0, "M6 (f m = 6 MPa)"), (10.0, "M10 (f m = 10 MPa)"), (12.0, "M12 (f m = 12 MPa)")],
            ),
            fld_opt(
                "group",
                "Masonry unit group and type",
                "Materials",
                1.0,
                vec![
                    (1.0, "Group 1 clay / calcium silicate / aggregate concrete (K = 0.55)"),
                    (2.0, "Group 2 clay (K = 0.45)"),
                    (3.0, "Group 2 aggregate concrete (K = 0.52)"),
                    (4.0, "Autoclaved aerated concrete (K = 0.55)"),
                ],
            ),
            fld("gamma_m", "Partial factor γ M (National Annex)", "-", "Materials", 2.5),
            fld("phi_inf", "Final creep coefficient φ ∞ (Table 3.8)", "-", "Materials", 1.0),
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
    let (n_ed, e_load, t_w, h, rho_n, fb, fm, grp, gm, phi_inf) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9]);
    let mut errs = vec![];
    if !(75.0..=600.0).contains(&t_w) {
        errs.push("Wall thickness must be between 75 and 600 mm".to_string());
    }
    if !(2.0..=75.0).contains(&fb) {
        errs.push("f b must be between 2 and 75 MPa".to_string());
    }
    if !(1.5..=3.5).contains(&gm) {
        errs.push("γ M must be between 1.5 and 3.5 (EN 1996-1-1 Table 2.3 Note ranges)".to_string());
    }
    if phi_inf > 3.0 {
        errs.push("Final creep coefficient must not exceed 3.0".to_string());
    }
    if e_load >= 0.45 * t_w {
        errs.push("Load eccentricity must be less than 0.45·t".to_string());
    }
    let (k_val, k_name) = match grp.round() as i64 {
        1 => (0.55, "Group 1"),
        2 => (0.45, "Group 2 clay"),
        3 => (0.52, "Group 2 aggregate concrete"),
        4 => (0.55, "Autoclaved aerated concrete"),
        _ => {
            errs.push("Unknown masonry unit group".to_string());
            (0.55, "")
        }
    };
    if !errs.is_empty() {
        return fail(errs);
    }
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = (
        "1. Design data",
        "2. Masonry strength",
        "3. Effective height, thickness and slenderness",
        "4. Top and bottom of the wall",
        "5. Mid-height of the wall",
    );
    t.step(s1, "Wall, load and materials", "", "", &format!("N_{{Ed}} = {} kN/m at eccentricity e = {} mm;  t = {} mm;  h = {} mm;  ρ_n = {};  f_b = {} MPa;  f_m = {} MPa ({});  γ_M = {};  φ_∞ = {}", n_ed, e_load, t_w, h, rho_n, fb, fm, k_name, gm, phi_inf), "", "Single-leaf wall of general-purpose mortar, braced at top and bottom, no openings, self-weight included in N_Ed. Wind and concentrated loads are not included.");
    let fm_use = fm.min(20.0).min(2.0 * fb);
    let fk = k_val * fb.powf(0.7) * fm_use.powf(0.3);
    let fd = fk / gm;
    t.step(
        s2,
        "Characteristic and design compressive strength",
        "EN 1996-1-1 cl. 3.6.1.2(1), Eq 3.1, Table 3.3; cl. 2.4.3",
        "f_{k} = K·f_{b}^{0.7}·f_{m}^{0.3},  f_{m} ≤ min(20 ; 2f_{b});   f_{d} = f_{k}/γ_{M}",
        &format!("f_m = {:.1} MPa;  f_k = {} × {}^{{0.7}} × {:.1}^{{0.3}};  f_d = {:.3} / {}", fm_use, k_val, fb, fm_use, fk, gm),
        &format!("f_{{k}} = {:.3} MPa;  f_{{d}} = {:.3} MPa", fk, fd),
        &format!("K and exponents as recommended for general-purpose mortar. γ_M is a National Annex value. Concrete parameter set ({}) not used.", ps.label),
    );
    let hef = rho_n * h;
    let tef = t_w;
    let slender = hef / tef;
    t.step(
        s3,
        "Effective height, thickness and slenderness ratio",
        "EN 1996-1-1 cl. 5.5.1.2 Eq 5.2, 5.5.1.3, 5.5.1.4",
        "h_{ef} = ρ_{n}·h;   t_{ef} = t (single leaf);   h_{ef}/t_{ef} ≤ 27",
        &format!("h_ef = {} × {} = {:.0} mm;  t_ef = {} mm", rho_n, h, hef, tef),
        &format!("h_{{ef}}/t_{{ef}} = {:.2}", slender),
        "",
    );
    let einit = hef / 450.0;
    let ei = (e_load + einit).max(0.05 * t_w);
    let phi_i = 1.0 - 2.0 * ei / t_w;
    let nrd_i = phi_i * t_w * fd;
    t.step(
        s4,
        "Eccentricity and capacity reduction at the top",
        "EN 1996-1-1 cl. 5.5.1.1(4), 6.1.2.2(1), Eq 6.4, 6.5",
        "e_{init} = h_{ef}/450;   e_{i} = M_{id}/N_{id} + e_{he} + e_{init} ≥ 0.05t;   Φ_{i} = 1 − 2e_{i}/t;   N_{Rd} = Φ·t·f_{d}",
        &format!("e_init = {:.0}/450 = {:.2} mm;  e_i = max({} + {:.2} ; 0.05 × {}) = {:.2} mm;  Φ_i = 1 − 2 × {:.2}/{}", hef, einit, e_load, einit, t_w, ei, ei, t_w),
        &format!("Φ_{{i}} = {:.3};  N_{{Rd,i}} = {:.1} kN/m", phi_i, nrd_i),
        "The eccentricity of the load at the top is the user's input (bearing of floors, Annex C or simplified rules). The bottom is taken as no more onerous than the top.",
    );
    let em = (0.6 * e_load + einit).max(0.0);
    let lam_c = 15.0;
    let ek = if slender <= lam_c { 0.0 } else { 0.002 * phi_inf * slender * (t_w * em).sqrt() };
    let emk = (em + ek).max(0.05 * t_w);
    let e_mod = 1000.0 * fk;
    let lam = slender * (fk / e_mod).sqrt();
    let a1 = 1.0 - 2.0 * emk / t_w;
    let u = (lam - 0.063) / (0.73 - 1.17 * emk / t_w);
    let phi_m = (a1 * (-u * u / 2.0).exp()).max(0.0);
    let nrd_m = phi_m * t_w * fd;
    t.step(
        s5,
        "Eccentricity at mid-height including creep",
        "EN 1996-1-1 cl. 6.1.2.2(2), Eq 6.6, 6.7, 6.9",
        "e_{m} = M_{md}/N_{md} + e_{init};   e_{k} = 0.002·φ_{∞}·(h_{ef}/t_{ef})·√(t·e_{m})  (e_k = 0 for h_ef/t_ef ≤ 15);   e_{mk} = e_{m} + e_{k} ≥ 0.05t",
        &format!("M_md/N_md = 0.6 × {} = {:.2} mm (linear moment, top eccentric, bottom pinned);  e_m = {:.2} mm;  e_k = {:.2} mm", e_load, 0.6 * e_load, em, ek),
        &format!("e_{{mk}} = {:.2} mm", emk),
        "The mid-height moment is taken as 0.6 of the top moment, the usual value for a wall pinned at the base.",
    );
    t.step(
        s5,
        "Capacity reduction factor at mid-height",
        "EN 1996-1-1 Annex G, Eq G.1-G.4 (K_E = 1000)",
        "A_{1} = 1 − 2e_{mk}/t;   λ = (h_{ef}/t_{ef})·√(f_{k}/E),  E = 1000·f_{k};   u = (λ − 0.063)/(0.73 − 1.17e_{mk}/t);   Φ_{m} = A_{1}·e^{−u²/2}",
        &format!("A_1 = {:.3};  λ = {:.2} × √(1/1000) = {:.3};  u = {:.3}", a1, slender, lam, u),
        &format!("Φ_{{m}} = {:.3};  N_{{Rd,m}} = {:.1} kN/m", phi_m, nrd_m),
        "",
    );
    let checks = vec![
        chk("Slenderness h ef / t ef", "EN 1996-1-1 5.5.1.4", format!("{:.2}", slender), "27.0".into(), slender / 27.0, slender <= 27.0),
        chk(
            "Vertical load at top (Φ i)",
            "EN 1996-1-1 6.1.2.1 Eq 6.2",
            format!("{:.1} kN/m", n_ed),
            format!("{:.1} kN/m", nrd_i),
            if nrd_i > 0.0 { n_ed / nrd_i } else { 9.99 },
            n_ed <= nrd_i,
        ),
        chk(
            "Vertical load at mid-height (Φ m)",
            "EN 1996-1-1 6.1.2.1 · Annex G",
            format!("{:.1} kN/m", n_ed),
            format!("{:.1} kN/m", nrd_m),
            if nrd_m > 0.0 { n_ed / nrd_m } else { 9.99 },
            n_ed <= nrd_m,
        ),
    ];
    let summary = vec![
        kv("f k / f d", format!("{:.3} / {:.3} MPa", fk, fd)),
        kv("h ef / t ef", format!("{:.0} / {:.0} mm ({:.2})", hef, tef, slender)),
        kv("e i / e mk", format!("{:.2} / {:.2} mm", ei, emk)),
        kv("Φ i / Φ m", format!("{:.3} / {:.3}", phi_i, phi_m)),
        kv("N Rd (governing)", format!("{:.1} kN/m", nrd_i.min(nrd_m))),
    ];
    let mut r = finish(
        t,
        checks,
        &ps.status,
        summary,
        vec![],
        vec![("AMBER", format!("γ_M = {} and φ_∞ = {} are National Annex / material values entered by the user: confirm them for the project.", gm, phi_inf))],
        vec![
            "Implemented: f k (Eq 3.1), h ef with ρ2, slenderness limit 27, Φ at top (Eq 6.4) and at mid-height with creep (Annex G).".into(),
            "NOT implemented: walls restrained on three or four sides (ρ3, ρ4), cavity and double-leaf walls, concentrated loads (6.1.3), shear walls (6.2), lateral load (6.3), small-area reduction, chases and recesses.".into(),
            "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into(),
        ],
        Drawing::Masonry { t: t_w, h, e: e_load },
    );
    r.outputs = vec![out("n_ed", "Vertical load N Ed", n_ed, "kN/m"), out("reaction", "Base reaction (vertical)", n_ed, "kN/m")];
    r
}
