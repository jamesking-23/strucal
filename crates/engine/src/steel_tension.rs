//! Steel tension member (EN 1993-1-1:2005 cl. 6.2.3): plastic resistance of the gross section, ultimate resistance of the
//! net section at fastener holes, and the ductility condition of cl. 6.2.3(3). Plate, flat bar or any section whose net
//! area is entered directly. Angles connected through one leg (EN 1993-1-8 cl. 3.10.3) are outside this module.
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;

/// EN 1993-1-1 cl. 6.1(1) Note 2B recommended value for the net-section resistance.
const GAMMA_M2: f64 = 1.25;

pub fn def() -> ModDef {
    ModDef {
        id: "steeltension",
        name: "Steel tension member",
        title: "Steel tension member (gross and net section)",
        subtitle: "EN 1993-1-1:2005 · Axial tension, gross-section yielding and net-section fracture, cl. 6.2.3",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Design tension N Ed (ULS)", "kN", "Actions - ULS", 380.0),
            fld("b", "Plate width b", "mm", "Section (flat plate)", 160.0),
            fld("t", "Plate thickness t", "mm", "Section (flat plate)", 12.0),
            fld("n_holes", "Holes across the critical section", "holes", "Holes", 2.0),
            fld("d0", "Hole diameter d 0", "mm", "Holes", 22.0),
            fld("len", "Member length L", "mm", "Geometry", 3000.0),
            fld("fy", "Yield strength f y", "MPa", "Materials", 275.0),
            fld("fu", "Ultimate strength f u", "MPa", "Materials", 430.0),
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
    let (n_ed, b, t_pl, n_holes, dh, len, fy, fu) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]);
    let mut errs = vec![];
    if n_holes.fract() != 0.0 || n_holes > 20.0 {
        errs.push("Holes across the critical section must be a whole number from 0 to 20".to_string());
    }
    if n_holes * dh >= b {
        errs.push("The holes take up the whole plate width: reduce the number or diameter of holes".to_string());
    }
    if !(235.0..=460.0).contains(&fy) {
        errs.push("fy must be between 235 and 460 MPa (S235-S460)".to_string());
    }
    if fu <= fy {
        errs.push("fu must be greater than fy".to_string());
    }
    if t_pl > 40.0 {
        errs.push("Plate thickness above 40 mm is outside this module (strengths of EN 1993-1-1 Table 3.1 for t ≤ 40 mm)".to_string());
    }
    if !errs.is_empty() {
        return fail(errs);
    }
    let mut t = Trace::new();
    let (s1, s2, s3) = ("1. Design data", "2. Section areas", "3. Tension resistance");
    t.step(
        s1,
        "Member, load and material",
        "",
        "",
        &format!(
            "N_{{Ed}} = {} kN;  plate {} × {} mm;  {} hole(s) of d_0 = {} mm across the critical section;  L = {} mm;  f_y = {} MPa;  f_u = {} MPa",
            n_ed, b, t_pl, n_holes, dh, len, fy, fu
        ),
        "",
        "Holes in line across the section (no staggered paths). Concentric tension; the connection itself is checked separately.",
    );
    t.step(
        s1,
        "Partial factors",
        "EN 1993-1-1 cl. 6.1(1)",
        "",
        &format!("γ_{{M0}} = {:.2};  γ_{{M2}} = {:.2}", ps.gamma_m0, GAMMA_M2),
        "",
        &format!("Parameter set: {} [{}]. γ_M2 = 1.25 is the recommended value of EN 1993-1-1 Note 2B.", ps.label, ps.status),
    );
    let a = b * t_pl;
    let a_net = (b - n_holes * dh) * t_pl;
    t.step(
        s2,
        "Gross and net areas",
        "EN 1993-1-1 cl. 6.2.2.1, 6.2.2.2",
        "A = b·t;   A_{net} = (b − n·d_0)·t",
        &format!("A = {} × {} ;  A_net = ({} − {} × {}) × {}", b, t_pl, b, n_holes, dh, t_pl),
        &format!("A = {:.0} mm^{{2}};  A_{{net}} = {:.0} mm^{{2}}  (A_net/A = {:.3})", a, a_net, a_net / a),
        "",
    );
    let npl = a * fy / ps.gamma_m0;
    let nu = 0.9 * a_net * fu / GAMMA_M2;
    let ntrd = npl.min(nu);
    t.step(
        s3,
        "Design plastic resistance of the gross section",
        "EN 1993-1-1 cl. 6.2.3(2a), Eq 6.6",
        "N_{pl,Rd} = A·f_{y}/γ_{M0}",
        &format!("= {:.0} × {} / {:.2}", a, fy, ps.gamma_m0),
        &format!("N_{{pl,Rd}} = {:.1} kN", npl / 1e3),
        "",
    );
    t.step(
        s3,
        "Design ultimate resistance of the net section",
        "EN 1993-1-1 cl. 6.2.3(2b), Eq 6.7",
        "N_{u,Rd} = 0.9·A_{net}·f_{u}/γ_{M2}",
        &format!("= 0.9 × {:.0} × {} / {:.2}", a_net, fu, GAMMA_M2),
        &format!("N_{{u,Rd}} = {:.1} kN", nu / 1e3),
        "",
    );
    t.step(
        s3,
        "Design tension resistance",
        "EN 1993-1-1 cl. 6.2.3(2), Eq 6.5",
        "N_{t,Rd} = min(N_{pl,Rd} ; N_{u,Rd});   N_{Ed}/N_{t,Rd} ≤ 1.0",
        &format!("= min({:.1} ; {:.1}) kN;  {} / {:.1}", npl / 1e3, nu / 1e3, n_ed, ntrd / 1e3),
        &format!("N_{{t,Rd}} = {:.1} kN;  η = {:.3}", ntrd / 1e3, n_ed * 1e3 / ntrd),
        if nu < npl { "Net-section fracture governs." } else { "Gross-section yielding governs." },
    );
    let duct_lhs = a_net * fu * 0.9 / GAMMA_M2;
    let duct_rhs = a * fy / ps.gamma_m0;
    t.step(
        s3,
        "Ductile behaviour (capacity design only)",
        "EN 1993-1-1 cl. 6.2.3(3)",
        "Where ductile behaviour is required: N_{u,Rd} ≥ N_{pl,Rd}",
        &format!("{:.1} kN {} {:.1} kN", duct_lhs / 1e3, if duct_lhs >= duct_rhs { "≥" } else { "<" }, duct_rhs / 1e3),
        if duct_lhs >= duct_rhs { "Ductile: gross section yields first" } else { "Not ductile: net section fractures first" },
        "Reported for information; required only where ductile behaviour is demanded (e.g. seismic or capacity design).",
    );
    let checks = vec![
        chk(
            "Gross-section yielding",
            "EN 1993-1-1 6.2.3 Eq 6.6",
            format!("{:.1} kN", n_ed),
            format!("{:.1} kN", npl / 1e3),
            n_ed * 1e3 / npl,
            n_ed * 1e3 <= npl,
        ),
        chk(
            "Net-section fracture",
            "EN 1993-1-1 6.2.3 Eq 6.7",
            format!("{:.1} kN", n_ed),
            format!("{:.1} kN", nu / 1e3),
            n_ed * 1e3 / nu,
            n_ed * 1e3 <= nu,
        ),
    ];
    let summary = vec![
        kv("Gross area A", format!("{:.0} mm²", a)),
        kv("Net area A net", format!("{:.0} mm²", a_net)),
        kv("N pl,Rd (gross yielding)", format!("{:.1} kN", npl / 1e3)),
        kv("N u,Rd (net fracture)", format!("{:.1} kN", nu / 1e3)),
        kv("N t,Rd", format!("{:.1} kN", ntrd / 1e3)),
        kv("Ductile (N u,Rd ≥ N pl,Rd)", if duct_lhs >= duct_rhs { "yes".into() } else { "no".into() }),
    ];
    let mut extra = vec![];
    if duct_lhs < duct_rhs {
        extra.push((
            "INFO",
            "The net section fractures before the gross section yields; not acceptable where ductile behaviour is required (cl. 6.2.3(3))."
                .to_string(),
        ));
    }
    let mut r = finish(
        t,
        checks,
        &ps.status,
        summary,
        vec![],
        extra,
        vec![
            "Implemented: gross-section yielding (Eq 6.6), net-section fracture (Eq 6.7), ductility condition 6.2.3(3) for information.".into(),
            "NOT implemented: staggered holes (6.2.2.2(4)), angles connected through one leg (EN 1993-1-8 3.10.3), block tearing, connection design, fatigue.".into(),
            "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into(),
        ],
        Drawing::Plate { b, t: t_pl, n_holes: n_holes as u32, d0: dh },
    );
    r.outputs = vec![out("n_ed", "Design tension N Ed", n_ed, "kN"), out("ntrd", "Tension resistance N t,Rd", ntrd / 1e3, "kN")];
    r
}
