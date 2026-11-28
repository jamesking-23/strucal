//! Steel beam-column: hot-rolled doubly symmetric I/H member under axial compression and major-axis bending with a linear
//! moment distribution (EN 1993-1-1:2005 cl. 6.2.9, 6.3.1, 6.3.2, 6.3.3 and Annex B, method 2). The member is
//! susceptible to torsional deformation (open section, lateral-torsional buckling not prevented unless λLT ≤ 0.4).
use crate::module::*;
use crate::params;
use crate::steel_beam::props;
use crate::steel_column::curve;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

const E: f64 = 210_000.0;
const G: f64 = 81_000.0;

pub fn def() -> ModDef {
    ModDef {
        id: "beamcol",
        name: "Steel beam-column",
        title: "Steel beam-column (axial compression with bending)",
        subtitle: "EN 1993-1-1:2005 · Axial force and major-axis bending, interaction Eq 6.61/6.62, Annex B (method 2)",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Axial compression N Ed (ULS)", "kN", "Actions - ULS", 600.0),
            fld("my_ed", "Maximum end moment M y,Ed (ULS)", "kNm", "Actions - ULS", 80.0),
            fld_opt(
                "psi",
                "End-moment ratio ψ = M smaller / M larger",
                "Actions - ULS",
                0.0,
                vec![
                    (1.0, "ψ = 1.0 (uniform moment)"),
                    (0.5, "ψ = 0.5"),
                    (0.0, "ψ = 0 (one end pinned)"),
                    (-0.5, "ψ = −0.5"),
                    (-1.0, "ψ = −1.0 (double curvature)"),
                ],
            ),
            fld("len", "Member length L", "mm", "Geometry", 4000.0),
            fld("ky", "Effective length factor, y-y", "-", "Geometry", 1.0),
            fld("kz", "Effective length factor, z-z", "-", "Geometry", 1.0),
            fld("klt", "Effective length factor, lateral-torsional", "-", "Geometry", 1.0),
            fld("h", "Section depth h", "mm", "Section (HEB 240 shown)", 240.0),
            fld("b", "Flange width b", "mm", "Section (HEB 240 shown)", 240.0),
            fld("tw", "Web thickness t w", "mm", "Section (HEB 240 shown)", 10.0),
            fld("tf", "Flange thickness t f", "mm", "Section (HEB 240 shown)", 17.0),
            fld("r", "Root radius r", "mm", "Section (HEB 240 shown)", 21.0),
            fld("fy", "Yield strength f y", "MPa", "Materials", 355.0),
        ],
    }
}

/// Buckling reduction factor, EN 1993-1-1 Eq 6.49 (flexural) — returns (Φ, χ).
fn chi_flex(lam: f64, alpha: f64) -> (f64, f64) {
    let phi = 0.5 * (1.0 + alpha * (lam - 0.2) + lam * lam);
    let chi = (1.0 / (phi + (phi * phi - lam * lam).max(0.0).sqrt())).min(1.0);
    (phi, if lam <= 0.2 { 1.0 } else { chi })
}

pub fn run(v: &Value) -> ModResult {
    let d0 = def();
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) {
        Some(p) => p,
        None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]),
    };
    // ψ may be zero or negative: read it separately from the positive inputs.
    let psi = match num(v, "psi") {
        Some(p) if p.is_finite() && (-1.0..=1.0).contains(&p) => p,
        _ => return fail(vec!["End-moment ratio ψ must be between −1 and 1".into()]),
    };
    let mut probe = v.clone();
    if let Some(o) = probe.as_object_mut() {
        o.insert("psi".into(), serde_json::json!(1.0));
    }
    let x = match require(&probe, &d0, false) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let (n_ed, my_ed, len, ky, kz, klt, h, b, tw, tf, r, fy) = (x[0], x[1], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11], x[12]);
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
    for (k, n) in [(ky, "y-y"), (kz, "z-z"), (klt, "lateral-torsional")] {
        if !(0.5..=2.5).contains(&k) {
            errs.push(format!("Effective length factor {} must be between 0.5 and 2.5", n));
        }
    }
    if !errs.is_empty() {
        return fail(errs);
    }

    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5, s6) = (
        "1. Design data",
        "2. Section properties and class",
        "3. Cross-section resistance",
        "4. Flexural buckling",
        "5. Lateral-torsional buckling",
        "6. Member interaction (Annex B)",
    );
    t.step(s1, "Member, actions and material", "", "", &format!("N_{{Ed}} = {} kN;  M_{{y,Ed}} = {} kNm (larger end);  ψ = {};  L = {} mm;  k_y = {};  k_z = {};  k_LT = {};  h = {} mm;  b = {} mm;  t_w = {} mm;  t_f = {} mm;  r = {} mm;  f_y = {} MPa", n_ed, my_ed, psi, len, ky, kz, klt, h, b, tw, tf, r, fy), "", "Linear major-axis moment between M and ψ·M, no transverse load, no minor-axis moment. Ends restrained against torsion; load applied through the shear centre.");
    t.step(
        s1,
        "Partial factors",
        "EN 1993-1-1 cl. 6.1(1)",
        "",
        &format!("γ_{{M0}} = {:.2};  γ_{{M1}} = {:.2};  E = 210000 MPa;  G = 81000 MPa", ps.gamma_m0, ps.gamma_m1),
        "",
        &format!("Parameter set: {} [{}]", ps.label, ps.status),
    );

    let p = props(h, b, tw, tf, r);
    let hw = h - 2.0 * tf;
    t.step(
        s2,
        "Section properties",
        "Section geometry (rectangles + root fillets)",
        "A;  I_y;  I_z;  W_{el,y};  W_{pl,y};  I_t;  I_w",
        &format!("h_w = {:.1} mm", hw),
        &format!(
            "A = {:.0} mm^{{2}};  I_y = {:.2}×10^{{6}};  I_z = {:.2}×10^{{6}} mm^{{4}};  W_el,y = {:.1}×10^{{3}};  W_pl,y = {:.1}×10^{{3}} mm^{{3}};  I_t = {:.1}×10^{{3}} mm^{{4}};  I_w = {:.3}×10^{{9}} mm^{{6}}",
            p.a, p.iy / 1e6, p.iz / 1e6, p.wel / 1e3, p.wpl / 1e3, p.it / 1e3, p.iw / 1e9
        ),
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
        s2,
        "Cross-section classification",
        "EN 1993-1-1 Table 5.2",
        "ε = √(235/f_y);  flange outstand c_f/t_f ≤ 9ε / 10ε / 14ε;  web conservatively in pure compression c_w/t_w ≤ 33ε / 38ε / 42ε",
        &format!("ε = {:.3};  c_f/t_f = {:.2};  c_w/t_w = {:.2}", eps, cf / tf, cw / tw),
        &format!("Flange class {};  web class {};  section class {}", cls_f, cls_w, cls),
        "Classifying the web for pure compression is conservative for combined axial force and bending.",
    );
    if cls == 4 {
        return fail(vec![format!(
            "Class 4 section (flange class {}, web class {}): effective properties are not implemented. Choose a more compact section.",
            cls_f, cls_w
        )]);
    }
    let plastic = cls <= 2;
    let wy = if plastic { p.wpl } else { p.wel };
    let nrk = p.a * fy;
    let myrk = wy * fy;

    // ---------------- cross-section ----------------
    let npl = nrk / ps.gamma_m0;
    let mrd = myrk / ps.gamma_m0;
    let n_rat = n_ed * 1e3 / npl;
    let (cs_util, cs_cap_txt) = if plastic {
        let a_w = ((p.a - 2.0 * b * tf) / p.a).min(0.5);
        let mn = (mrd * (1.0 - n_rat) / (1.0 - 0.5 * a_w)).min(mrd).max(0.0);
        t.step(
            s3,
            "Bending and axial force (plastic, class 1 or 2)",
            "EN 1993-1-1 cl. 6.2.9.1(5), Eq 6.36",
            "n = N_{Ed}/N_{pl,Rd};   a = (A − 2b·t_f)/A ≤ 0.5;   M_{N,y,Rd} = M_{pl,y,Rd}·(1 − n)/(1 − 0.5a) ≤ M_{pl,y,Rd}",
            &format!(
                "N_pl,Rd = {:.0} × {} / {:.2} = {:.0} kN;  M_pl,y,Rd = {:.1} kNm;  n = {:.3};  a = {:.3}",
                p.a,
                fy,
                ps.gamma_m0,
                npl / 1e3,
                mrd / 1e6,
                n_rat,
                a_w
            ),
            &format!("M_{{N,y,Rd}} = {:.1} kNm", mn / 1e6),
            "",
        );
        let u = if mn > 0.0 { my_ed * 1e6 / mn } else { 9.99 };
        (u.max(n_rat), format!("{:.1} kNm", mn / 1e6))
    } else {
        let sigma = (n_ed * 1e3 / p.a + my_ed * 1e6 / p.wel) * ps.gamma_m0 / fy;
        t.step(
            s3,
            "Bending and axial force (elastic, class 3)",
            "EN 1993-1-1 cl. 6.2.9.2, Eq 6.42",
            "σ_{x,Ed} = N_{Ed}/A + M_{y,Ed}/W_{el,y} ≤ f_y/γ_{M0}",
            &format!("= {:.1}×10^{{3}}/{:.0} + {:.1}×10^{{6}}/{:.0}", n_ed, p.a, my_ed, p.wel),
            &format!("σ_{{x,Ed}}·γ_{{M0}}/f_y = {:.3}", sigma),
            "",
        );
        (sigma, format!("f_y/γ_M0 = {:.0} MPa", fy / ps.gamma_m0))
    };

    // ---------------- flexural buckling ----------------
    let mut chis = vec![];
    for (axis, i_ax, k, is_y) in [("y-y", p.iy, ky, true), ("z-z", p.iz, kz, false)] {
        let lcr = k * len;
        let ncr = PI * PI * E * i_ax / (lcr * lcr);
        let lam = (nrk / ncr).sqrt();
        let (cn, al) = curve(h, b, tf, is_y);
        let (phi, chi) = chi_flex(lam, al);
        t.step(
            s4,
            &format!("Flexural buckling about {}", axis),
            "EN 1993-1-1 cl. 6.3.1.2, 6.3.1.3, Eq 6.49, Table 6.2",
            "N_{cr} = π^{2}EI/L_{cr}^{2};   λ = √(A·f_y/N_{cr});   Φ = 0.5[1 + α(λ − 0.2) + λ^{2}];   χ = 1/[Φ + √(Φ^{2} − λ^{2})] ≤ 1",
            &format!("L_cr = {:.0} mm;  N_cr = {:.0} kN;  λ = {:.3};  curve {} (α = {});  Φ = {:.3}", lcr, ncr / 1e3, lam, cn, al, phi),
            &format!("χ_{} = {:.3}", if is_y { "y" } else { "z" }, chi),
            if lam <= 0.2 { "λ ≤ 0.2: buckling effects may be ignored (cl. 6.3.1.2(4)); χ = 1.0." } else { "" },
        );
        chis.push((lam, chi));
    }
    let ((lam_y, chi_y), (lam_z, chi_z)) = (chis[0], chis[1]);

    // ---------------- lateral-torsional buckling ----------------
    let llt = klt * len;
    let c1 = (1.88 - 1.40 * psi + 0.52 * psi * psi).min(2.70);
    let k1 = PI * PI * E * p.iz / (llt * llt);
    let mcr = c1 * k1 * (p.iw / p.iz + llt * llt * G * p.it / (PI * PI * E * p.iz)).sqrt();
    let lam_lt = (myrk / mcr).sqrt();
    let (cn_lt, al_lt) = if h / b <= 2.0 { ("b", 0.34) } else { ("c", 0.49) };
    let phi_lt = 0.5 * (1.0 + al_lt * (lam_lt - 0.4) + 0.75 * lam_lt * lam_lt);
    let chi_lt_raw = (1.0 / (phi_lt + (phi_lt * phi_lt - 0.75 * lam_lt * lam_lt).max(0.0).sqrt())).min(1.0).min(1.0 / (lam_lt * lam_lt));
    let chi_lt = if lam_lt <= 0.4 { 1.0 } else { chi_lt_raw };
    t.step(
        s5,
        "Elastic critical moment",
        "Doubly symmetric section, end moments (load at the shear centre)",
        "M_{cr} = C_{1}·π^{2}EI_{z}/L_{LT}^{2}·√[I_{w}/I_{z} + L_{LT}^{2}GI_{t}/(π^{2}EI_{z})];   C_{1} = 1.88 − 1.40ψ + 0.52ψ^{2} ≤ 2.70",
        &format!("L_LT = {:.0} mm;  C_1 = {:.3};  π²EI_z/L² = {:.1} kN", llt, c1, k1 / 1e3),
        &format!("M_{{cr}} = {:.1} kNm", mcr / 1e6),
        "The C1 expression is a widely used approximation (not in the Eurocode text). Verify, or confirm Mcr independently.",
    );
    t.step(
        s5,
        "Reduction factor, rolled sections",
        "EN 1993-1-1 cl. 6.3.2.3, Eq 6.57, Table 6.5",
        "λ_{LT} = √(W_{y}·f_y/M_{cr});   Φ_{LT} = 0.5[1 + α_{LT}(λ_{LT} − 0.4) + 0.75λ_{LT}^{2}];   χ_{LT} = 1/[Φ_{LT} + √(Φ_{LT}^{2} − 0.75λ_{LT}^{2})] ≤ 1, ≤ 1/λ_{LT}^{2}",
        &format!("W_y = {:.1}×10^{{3}} mm^{{3}} ({});  λ_LT = {:.3};  h/b = {:.2} → curve {} (α_LT = {});  Φ_LT = {:.3}", wy / 1e3, if plastic { "W_pl" } else { "W_el" }, lam_lt, h / b, cn_lt, al_lt, phi_lt),
        &format!("χ_{{LT}} = {:.3}", chi_lt),
        if lam_lt <= 0.4 { "λ_LT ≤ 0.4: lateral-torsional buckling may be ignored; χ_LT = 1.0." } else { "The modification factor f (Eq 6.58) is conservatively not applied." },
    );

    // ---------------- interaction ----------------
    let cm = (0.6 + 0.4 * psi).max(0.4);
    let ny = n_ed * 1e3 / (chi_y * nrk / ps.gamma_m1);
    let nz = n_ed * 1e3 / (chi_z * nrk / ps.gamma_m1);
    let (kyy, kzy) = if plastic {
        let kyy = (cm * (1.0 + (lam_y - 0.2) * ny)).min(cm * (1.0 + 0.8 * ny));
        let kzy = if lam_z < 0.4 {
            (0.6 + lam_z).min(1.0 - 0.1 * lam_z * nz / (cm - 0.25))
        } else {
            (1.0 - 0.1 * lam_z * nz / (cm - 0.25)).max(1.0 - 0.1 * nz / (cm - 0.25))
        };
        (kyy, kzy)
    } else {
        let kyy = (cm * (1.0 + 0.6 * lam_y * ny)).min(cm * (1.0 + 0.6 * ny));
        let kzy = (1.0 - 0.05 * lam_z * nz / (cm - 0.25)).max(1.0 - 0.05 * nz / (cm - 0.25));
        (kyy, kzy)
    };
    t.step(
        s6,
        "Equivalent moment factor and interaction factors",
        &format!("EN 1993-1-1 Annex B, Table B.3 and Table B.2 (class {})", if plastic { "1/2" } else { "3" }),
        if plastic {
            "C_{my} = C_{mLT} = 0.6 + 0.4ψ ≥ 0.4;  n_y = N_{Ed}/(χ_{y}N_{Rk}/γ_{M1});  k_{yy} = C_{my}[1 + (λ_{y} − 0.2)n_{y}] ≤ C_{my}(1 + 0.8n_{y});  k_{zy} = 1 − 0.1λ_{z}n_{z}/(C_{mLT} − 0.25) ≥ 1 − 0.1n_{z}/(C_{mLT} − 0.25)  (λ_z ≥ 0.4)"
        } else {
            "C_{my} = C_{mLT} = 0.6 + 0.4ψ ≥ 0.4;  k_{yy} = C_{my}(1 + 0.6λ_{y}n_{y}) ≤ C_{my}(1 + 0.6n_{y});  k_{zy} = 1 − 0.05λ_{z}n_{z}/(C_{mLT} − 0.25) ≥ 1 − 0.05n_{z}/(C_{mLT} − 0.25)"
        },
        &format!("C_m = {:.3};  n_y = {:.3};  n_z = {:.3};  λ_y = {:.3};  λ_z = {:.3}", cm, ny, nz, lam_y, lam_z),
        &format!("k_{{yy}} = {:.3};  k_{{zy}} = {:.3}", kyy, kzy),
        "",
    );
    let mb = chi_lt * myrk / ps.gamma_m1;
    let eq61 = ny + kyy * my_ed * 1e6 / mb;
    let eq62 = nz + kzy * my_ed * 1e6 / mb;
    t.step(
        s6,
        "Interaction about the major axis",
        "EN 1993-1-1 cl. 6.3.3(4), Eq 6.61",
        "N_{Ed}/(χ_{y}N_{Rk}/γ_{M1}) + k_{yy}·M_{y,Ed}/(χ_{LT}M_{y,Rk}/γ_{M1}) ≤ 1",
        &format!("= {:.3} + {:.3} × {} / {:.1}", ny, kyy, my_ed, mb / 1e6),
        &format!("{:.3}", eq61),
        "",
    );
    t.step(
        s6,
        "Interaction about the minor axis",
        "EN 1993-1-1 cl. 6.3.3(4), Eq 6.62",
        "N_{Ed}/(χ_{z}N_{Rk}/γ_{M1}) + k_{zy}·M_{y,Ed}/(χ_{LT}M_{y,Rk}/γ_{M1}) ≤ 1",
        &format!("= {:.3} + {:.3} × {} / {:.1}", nz, kzy, my_ed, mb / 1e6),
        &format!("{:.3}", eq62),
        "",
    );
    let checks = vec![
        chk(
            "Cross-section N + M",
            if plastic { "EN 1993-1-1 6.2.9.1" } else { "EN 1993-1-1 6.2.9.2" },
            format!("{} kNm · {} kN", my_ed, n_ed),
            cs_cap_txt,
            cs_util,
            cs_util <= 1.0,
        ),
        chk(
            "Flexural buckling y-y",
            "EN 1993-1-1 6.3.1",
            format!("{} kN", n_ed),
            format!("{:.0} kN", chi_y * nrk / ps.gamma_m1 / 1e3),
            ny,
            ny <= 1.0,
        ),
        chk(
            "Flexural buckling z-z",
            "EN 1993-1-1 6.3.1",
            format!("{} kN", n_ed),
            format!("{:.0} kN", chi_z * nrk / ps.gamma_m1 / 1e3),
            nz,
            nz <= 1.0,
        ),
        chk(
            "Lateral-torsional buckling",
            "EN 1993-1-1 6.3.2",
            format!("{} kNm", my_ed),
            format!("{:.1} kNm", mb / 1e6),
            my_ed * 1e6 / mb,
            my_ed * 1e6 <= mb,
        ),
        chk("Interaction, Eq 6.61", "EN 1993-1-1 6.3.3 · Annex B", format!("{:.3}", eq61), "1.000".into(), eq61, eq61 <= 1.0),
        chk("Interaction, Eq 6.62", "EN 1993-1-1 6.3.3 · Annex B", format!("{:.3}", eq62), "1.000".into(), eq62, eq62 <= 1.0),
    ];
    let summary = vec![
        kv("Section class", format!("{} (flange {}, web {})", cls, cls_f, cls_w)),
        kv("N pl,Rd / M y,Rd", format!("{:.0} kN / {:.1} kNm", npl / 1e3, mrd / 1e6)),
        kv("χ y / χ z / χ LT", format!("{:.3} / {:.3} / {:.3}", chi_y, chi_z, chi_lt)),
        kv("M cr (C1)", format!("{:.1} kNm ({:.3})", mcr / 1e6, c1)),
        kv("C my = C mLT", format!("{:.3}", cm)),
        kv("k yy / k zy", format!("{:.3} / {:.3}", kyy, kzy)),
    ];
    let mut r = finish(
        t,
        checks,
        &ps.status,
        summary,
        vec![],
        vec![("INFO", "Section properties are computed from the entered dimensions and should be compared with the manufacturer's table.".to_string())],
        vec![
            "Implemented: classification (web conservatively in compression), cross-section N+M (6.2.9), flexural buckling about both axes, LTB with Mcr from C1, interaction Eq 6.61/6.62 with Annex B factors.".into(),
            "NOT implemented: minor-axis bending, transverse loads between restraints, shear interaction, class 4 sections, torsionally restrained members (Table B.1), the LTB factor f.".into(),
            "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into(),
        ],
        Drawing::ISection { h, b, tw, tf },
    );
    r.outputs = vec![out("n_ed", "Axial force N Ed", n_ed, "kN"), out("reaction", "Base reaction (axial)", n_ed, "kN")];
    r
}
