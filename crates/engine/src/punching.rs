//! Punching shear at a slab-column connection (EN 1992-1-1:2004 cl. 6.4): basic control perimeter at 2d, maximum
//! resistance at the column face, resistance without punching reinforcement, and with vertical links where provided
//! (Eq 6.52), plus the outer perimeter where reinforcement is no longer required (Eq 6.54). Rectangular column,
//! slab of uniform depth, β from the recommended values of Figure 6.21N.
use crate::module::*;
use crate::params;
use crate::trace::Trace;
use serde_json::Value;
use std::f64::consts::PI;

pub fn def() -> ModDef {
    ModDef {
        id: "punching",
        name: "Punching shear",
        title: "Punching shear at a slab-column connection",
        subtitle: "EN 1992-1-1:2004 · Flat slab, rectangular column, control perimeters u0, u1 and u out,ef, cl. 6.4",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("v_ed", "Column reaction V Ed (ULS)", "kN", "Actions - ULS", 650.0),
            fld_opt(
                "position",
                "Column position (β, Figure 6.21N)",
                "Actions - ULS",
                1.0,
                vec![(1.0, "Internal column (β = 1.15)"), (2.0, "Edge column (β = 1.4)"), (3.0, "Corner column (β = 1.5)")],
            ),
            fld("cx", "Column dimension c 1 (perpendicular to the edge)", "mm", "Geometry", 400.0),
            fld("cy", "Column dimension c 2 (parallel to the edge)", "mm", "Geometry", 400.0),
            fld("h", "Slab thickness h", "mm", "Geometry", 250.0),
            fld("cover", "Nominal cover to the top bars", "mm", "Geometry", 30.0),
            fld("bar_y", "Top bar φ, direction y (outer layer)", "mm", "Tension reinforcement", 16.0),
            fld("s_y", "Top bar spacing, direction y", "mm", "Tension reinforcement", 150.0),
            fld("bar_z", "Top bar φ, direction z (inner layer)", "mm", "Tension reinforcement", 16.0),
            fld("s_z", "Top bar spacing, direction z", "mm", "Tension reinforcement", 150.0),
            fld("asw", "Punching links per perimeter A sw (0 = none)", "mm²", "Punching reinforcement", 1000.0),
            fld("sr", "Radial spacing of link perimeters s r", "mm", "Punching reinforcement", 125.0),
            fld("fck", "fck", "MPa", "Materials", 30.0),
            fld("fyk", "fyk (bars and links)", "MPa", "Materials", 500.0),
        ],
    }
}

/// Basic control perimeter at 2d and loaded-area perimeter for the column position (EN 1992-1-1 Fig. 6.13, 6.15, 6.4.5(3)).
fn perimeters(pos: u8, c1: f64, c2: f64, d: f64) -> (f64, f64, &'static str, &'static str) {
    match pos {
        2 => ((c2 + 3.0 * d).min(c2 + 2.0 * c1), 2.0 * c1 + c2 + 2.0 * PI * d, "u_0 = c_2 + 3d ≤ c_2 + 2c_1", "u_1 = 2c_1 + c_2 + π·(2d)"),
        3 => ((3.0 * d).min(c1 + c2), c1 + c2 + PI * d, "u_0 = 3d ≤ c_1 + c_2", "u_1 = c_1 + c_2 + (π/2)·(2d)"),
        _ => (2.0 * (c1 + c2), 2.0 * (c1 + c2) + 4.0 * PI * d, "u_0 = 2(c_1 + c_2)", "u_1 = 2(c_1 + c_2) + 2π·(2d)"),
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
    let (v_ed, pos_v, c1, c2, h, cover, bar_y, s_y, bar_z, s_z, asw, sr, fck, fyk) =
        (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8], x[9], x[10], x[11], x[12], x[13]);
    let pos = pos_v.round() as u8;
    let mut errs = vec![];
    if !(1..=3).contains(&pos) {
        errs.push("Column position must be internal, edge or corner".to_string());
    }
    let dy = h - cover - bar_y / 2.0;
    let dz = h - cover - bar_y - bar_z / 2.0;
    if dz <= 50.0 {
        errs.push("Effective depth below 50 mm: increase the slab thickness or reduce cover and bar sizes".to_string());
    }
    if !(12.0..=50.0).contains(&fck) {
        errs.push("fck must be between 12 and 50 MPa (C12/15 to C50/60)".to_string());
    }
    if !(400.0..=600.0).contains(&fyk) {
        errs.push("fyk must be between 400 and 600 MPa".to_string());
    }
    if s_y < bar_y || s_z < bar_z {
        errs.push("Bar spacing must exceed the bar diameter".to_string());
    }
    if c1 / c2 > 3.0 || c2 / c1 > 3.0 {
        errs.push(
            "Column aspect ratio above 3: EN 1992-1-1 6.4.2(3) requires a reduced perimeter (wall-like supports) not implemented here"
                .to_string(),
        );
    }
    if !errs.is_empty() {
        return fail(errs);
    }
    let beta = match pos {
        2 => 1.4,
        3 => 1.5,
        _ => 1.15,
    };
    let pos_name = match pos {
        2 => "edge",
        3 => "corner",
        _ => "internal",
    };
    let fcd = ps.alpha_cc * fck / ps.gamma_c;
    let fyd = fyk / ps.gamma_s;
    let mut t = Trace::new();
    let (s1, s2, s3, s4, s5) = (
        "1. Design data",
        "2. Effective depth and reinforcement ratio",
        "3. Column face (u0)",
        "4. Basic control perimeter (u1)",
        "5. Punching reinforcement",
    );
    t.step(s1, "Connection, load and material", "", "", &format!("V_{{Ed}} = {} kN;  {} column {} × {} mm;  h = {} mm;  cover = {} mm;  top bars T{} @ {} (y), T{} @ {} (z);  A_sw = {} mm^{{2}} per perimeter at s_r = {} mm;  f_ck = {} MPa;  f_yk = {} MPa", v_ed, pos_name, c1, c2, h, cover, bar_y, s_y, bar_z, s_z, asw, sr, fck, fyk), "", "Slab of uniform depth; column not near an opening; loaded-area dimensions are the column dimensions.");
    t.step(
        s1,
        "Partial factors and load eccentricity factor",
        "EN 1992-1-1 cl. 2.4.2.4, 3.1.6, 6.4.3(6), Figure 6.21N",
        "f_{cd} = α_{cc}·f_{ck}/γ_c;   f_{yd} = f_{yk}/γ_s;   β (recommended values, braced structure, spans differing ≤ 25 %)",
        &format!("γ_c = {:.2};  γ_s = {:.2};  α_cc = {:.2};  f_cd = {:.2} MPa;  f_yd = {:.1} MPa", ps.gamma_c, ps.gamma_s, ps.alpha_cc, fcd, fyd),
        &format!("β = {} ({} column)", beta, pos_name),
        &format!("Parameter set: {} [{}]. β from Figure 6.21N applies only where the lateral stability does not depend on frame action of slabs and columns.", ps.label, ps.status),
    );
    let d = (dy + dz) / 2.0;
    let rho_y = (PI * bar_y * bar_y / 4.0 / s_y) / dy;
    let rho_z = (PI * bar_z * bar_z / 4.0 / s_z) / dz;
    let rho = (rho_y * rho_z).sqrt().min(0.02);
    t.step(
        s2,
        "Effective depth and tension reinforcement ratio",
        "EN 1992-1-1 cl. 6.4.2(1), Eq 6.32; 6.4.4(1)",
        "d_{y} = h − c − φ_{y}/2;  d_{z} = h − c − φ_{y} − φ_{z}/2;  d_{eff} = (d_y + d_z)/2;  ρ_{l} = √(ρ_{ly}·ρ_{lz}) ≤ 0.02",
        &format!("d_y = {:.1} mm;  d_z = {:.1} mm;  ρ_ly = {:.5};  ρ_lz = {:.5}", dy, dz, rho_y, rho_z),
        &format!("d = {:.1} mm;  ρ_{{l}} = {:.5}", d, rho),
        "ρ calculated per unit width from the bar size and spacing; the bars should extend at least 3d beyond the column face.",
    );
    let (u0, u1, f0, f1) = perimeters(pos, c1, c2, d);
    let nu = 0.6 * (1.0 - fck / 250.0);
    let vmax = 0.5 * nu * fcd;
    let ved0 = beta * v_ed * 1e3 / (u0 * d);
    t.step(
        s3,
        "Maximum punching resistance at the column face",
        "EN 1992-1-1 cl. 6.4.5(3), Eq 6.53",
        &format!("{};   v_{{Ed,0}} = β·V_{{Ed}}/(u_0·d);   v_{{Rd,max}} = 0.5·ν·f_{{cd}},  ν = 0.6(1 − f_{{ck}}/250)", f0),
        &format!("u_0 = {:.0} mm;  v_Ed,0 = {} × {} × 10^{{3}} / ({:.0} × {:.1});  ν = {:.3}", u0, beta, v_ed, u0, d, nu),
        &format!("v_{{Ed,0}} = {:.3} MPa;  v_{{Rd,max}} = {:.3} MPa", ved0, vmax),
        "Some National Annexes adopt 0.4·ν·f_cd: confirm the governing annex.",
    );
    let k = (1.0 + (200.0 / d).sqrt()).min(2.0);
    let vmin = 0.035 * k.powf(1.5) * fck.sqrt();
    let vrdc = (0.18 / ps.gamma_c * k * (100.0 * rho * fck).powf(1.0 / 3.0)).max(vmin);
    let ved1 = beta * v_ed * 1e3 / (u1 * d);
    t.step(
        s4,
        "Punching stress and resistance at the basic control perimeter",
        "EN 1992-1-1 cl. 6.4.2, 6.4.3(3) Eq 6.38, 6.4.4(1) Eq 6.47",
        &format!("{};   v_{{Ed}} = β·V_{{Ed}}/(u_1·d);   v_{{Rd,c}} = max[C_{{Rd,c}}·k·(100ρ_l f_{{ck}})^{{1/3}} ; v_{{min}}],  k = 1 + √(200/d) ≤ 2", f1),
        &format!("u_1 = {:.0} mm;  v_Ed = {} × {} × 10^{{3}} / ({:.0} × {:.1});  k = {:.3};  C_Rd,c = {:.4};  v_min = {:.3} MPa", u1, beta, v_ed, u1, d, k, 0.18 / ps.gamma_c, vmin),
        &format!("v_{{Ed}} = {:.3} MPa;  v_{{Rd,c}} = {:.3} MPa", ved1, vrdc),
        "The reduction of V_Ed by the soil or slab load inside the perimeter is conservatively ignored.",
    );
    let mut checks = vec![chk(
        "Punching: column face (u0)",
        "EN 1992-1-1 6.4.5(3)",
        format!("{:.3} MPa", ved0),
        format!("{:.3} MPa", vmax),
        ved0 / vmax,
        ved0 <= vmax,
    )];
    let needs = ved1 > vrdc;
    let fywd_ef = (250.0 + 0.25 * d).min(fyd);
    let mut extra: Vec<(&str, String)> = vec![];
    let mut summary_extra: Vec<KV> = vec![];
    if asw > 0.0 {
        let vrdcs = 0.75 * vrdc + 1.5 * (d / sr) * asw * fywd_ef / (u1 * d);
        let uout = beta * v_ed * 1e3 / (vrdc * d);
        t.step(
            s5,
            "Resistance with punching links",
            "EN 1992-1-1 cl. 6.4.5(1), Eq 6.52",
            "f_{ywd,ef} = 250 + 0.25d ≤ f_{ywd};   v_{Rd,cs} = 0.75·v_{Rd,c} + 1.5·(d/s_{r})·A_{sw}·f_{ywd,ef}·[1/(u_{1}·d)]·sin α  (α = 90°)",
            &format!("f_ywd,ef = min(250 + 0.25 × {:.1} ; {:.1}) = {:.1} MPa;  v_Rd,cs = 0.75 × {:.3} + 1.5 × ({:.1}/{}) × {} × {:.1} / ({:.0} × {:.1})", d, fyd, fywd_ef, vrdc, d, sr, asw, fywd_ef, u1, d),
            &format!("v_{{Rd,cs}} = {:.3} MPa", vrdcs),
            "",
        );
        t.step(
            s5,
            "Outer control perimeter where links are no longer required",
            "EN 1992-1-1 cl. 6.4.5(4), Eq 6.54",
            "u_{out,ef} = β·V_{Ed}/(v_{Rd,c}·d);   outermost link perimeter ≤ k·d = 1.5d inside u_{out,ef}",
            &format!("= {} × {} × 10^{{3}} / ({:.3} × {:.1})", beta, v_ed, vrdc, d),
            &format!("u_{{out,ef}} = {:.0} mm (u_1 = {:.0} mm)", uout, u1),
            "Detailing to cl. 9.4.3: radial spacing s_r ≤ 0.75d, first perimeter between 0.3d and 0.5d from the column face.",
        );
        checks.push(chk(
            "Punching: with links (u1)",
            "EN 1992-1-1 6.4.5 Eq 6.52",
            format!("{:.3} MPa", ved1),
            format!("{:.3} MPa", vrdcs),
            ved1 / vrdcs,
            ved1 <= vrdcs,
        ));
        checks.push(chk(
            "Link radial spacing",
            "EN 1992-1-1 9.4.3(1)",
            format!("{:.0} mm", sr),
            format!("{:.0} mm max", 0.75 * d),
            sr / (0.75 * d),
            sr <= 0.75 * d + 1e-9,
        ));
        summary_extra.push(kv("v Rd,cs (with links)", format!("{:.3} MPa", vrdcs)));
        summary_extra.push(kv("u out,ef", format!("{:.0} mm", uout)));
    } else {
        let asw_req = if needs { ((ved1 - 0.75 * vrdc) * u1 * d / (1.5 * d * fywd_ef)).max(0.0) } else { 0.0 };
        t.step(
            s5,
            "Punching reinforcement",
            "EN 1992-1-1 cl. 6.4.3(2), 6.4.5(1) Eq 6.52",
            "Required where v_{Ed} > v_{Rd,c}:  (A_{sw}/s_{r}) ≥ (v_{Ed} − 0.75v_{Rd,c})·u_{1}·d / (1.5·d·f_{ywd,ef})",
            &format!("v_Ed = {:.3} {} v_Rd,c = {:.3} MPa;  f_ywd,ef = {:.1} MPa", ved1, if needs { ">" } else { "≤" }, vrdc, fywd_ef),
            &if needs { format!("A_{{sw}}/s_{{r}} ≥ {:.3} mm^{{2}}/mm", asw_req) } else { "Not required".to_string() },
            if needs {
                "Enter A_sw and s_r to verify a link arrangement (Eq 6.52), or increase the slab depth or column size."
            } else {
                ""
            },
        );
        checks.push(chk(
            "Punching: no links (u1)",
            "EN 1992-1-1 6.4.4 Eq 6.47",
            format!("{:.3} MPa", ved1),
            format!("{:.3} MPa", vrdc),
            ved1 / vrdc,
            !needs,
        ));
        if needs {
            extra.push((
                "INFO",
                format!("Punching reinforcement is needed: A_sw/s_r ≥ {:.3} mm²/mm, or increase the slab depth or column size.", asw_req),
            ));
            summary_extra.push(kv("A sw / s r required", format!("{:.3} mm²/mm", asw_req)));
        }
    }
    let mut summary = vec![
        kv("Column position / β", format!("{} / {}", pos_name, beta)),
        kv("Effective depth d", format!("{:.1} mm", d)),
        kv("ρ l", format!("{:.5}", rho)),
        kv("u 0 / u 1", format!("{:.0} / {:.0} mm", u0, u1)),
        kv("v Ed,0 / v Rd,max", format!("{:.3} / {:.3} MPa", ved0, vmax)),
        kv("v Ed / v Rd,c", format!("{:.3} / {:.3} MPa", ved1, vrdc)),
    ];
    summary.extend(summary_extra);
    let mut r = finish(
        t,
        checks,
        &ps.status,
        summary,
        vec![],
        extra,
        vec![
            "Implemented: β from Figure 6.21N, u0 and u1 for internal, edge and corner columns, v Rd,max, v Rd,c, links by Eq 6.52, u out,ef, link radial spacing.".into(),
            "NOT implemented: unbalanced moment by Eq 6.39 (β taken from Figure 6.21N instead), openings near the column, column heads/drop panels, prestress, reduction of V Ed by loads inside u1, detailed link layout.".into(),
            "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into(),
        ],
        Drawing::Punching { c1, c2, h, d, pos: pos as u32 },
    );
    r.outputs = vec![out("v_ed", "Column reaction V Ed", v_ed, "kN")];
    r
}
