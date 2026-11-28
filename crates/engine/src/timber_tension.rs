//! Solid timber member in axial tension parallel to the grain, optionally with major-axis bending (rules and values as
//! printed in the supplied Eurocode 5 source: EN 1995-1-1 cl. 6.1.2 and 6.2.3 with the UK National Annex values).
use crate::module::*;
use crate::params;
use crate::timber_data::*;
use crate::trace::Trace;
use serde_json::Value;

pub fn def() -> ModDef {
    ModDef {
        id: "timbertension",
        name: "Timber tension member",
        title: "Solid timber tension member (tension and bending)",
        subtitle:
            "Eurocode 5 (values as printed in the supplied source, UK National Annex) · Tension parallel to the grain, Eq 6.1 and 6.17",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("n_ed", "Design axial tension N t,d", "kN", "Actions - ULS", 40.0),
            fld("my_ed", "Design moment about y-y M y,d (0 if none)", "kNm", "Actions - ULS", 0.8),
            fld("b", "Breadth b", "mm", "Geometry", 75.0),
            fld("h", "Depth h", "mm", "Geometry", 150.0),
            fld("a_hole", "Area removed by holes or notches (0 if none)", "mm²", "Geometry", 1050.0),
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
    let (n_ed, my, b, h, a_hole, cls_v, svc_v, dur_v) = (x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]);
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
    if a_hole >= 0.5 * b * h {
        errs.push("Holes or notches remove half the section or more: outside this module".to_string());
    }
    if !errs.is_empty() {
        return fail(errs);
    }
    let km = kmod(svc, dur);
    let gm = GAMMA_M;
    let mut t = Trace::new();
    let (s1, s2, s3) = ("1. Design data", "2. Design strengths", "3. Stresses and verification");
    t.step(s1, "Member, actions and material", "", "", &format!("N_t,d = {} kN;  M_y,d = {} kNm;  b × h = {} × {} mm;  holes A = {} mm^{{2}};  strength class {};  service class {};  load-duration class {}", n_ed, my, b, h, a_hole, cls.name, svc, dur), "", "Bending about the major axis only. Holes are deducted from the area and, conservatively, from the section modulus in proportion.");
    t.step(
        s1,
        "Factors",
        "Supplied source: Tables 11.4, 11.5",
        "",
        &format!("γ_M = {:.2};  k_mod = {:.2}", gm, km),
        "",
        &format!(
            "UK National Annex values as printed in the source. Check the Ugandan values. Concrete parameter set ({}) not used.",
            ps.label
        ),
    );
    let kh_t = kh(h.max(b));
    let kh_m = kh(h);
    let ft0d = kh_t * km * cls.ft0k / gm;
    let fmyd = kh_m * km * cls.fmk / gm;
    t.step(
        s2,
        "Design strengths",
        "EN 1995-1-1 cl. 3.2(3), 2.4.1, Eq 2.14; supplied source Eq 11.2, 11.3",
        "k_{h} = min[(150/h)^{0.2} ; 1.3] for h < 150 mm (tension: largest cross-section dimension);   f_{t,0,d} = k_{h}·k_{mod}·f_{t,0,k}/γ_M;   f_{m,y,d} = k_{h}·k_{mod}·f_{m,k}/γ_M",
        &format!("k_h,t = {:.3};  f_t,0,d = {:.3} × {} × {} / {};  k_h,m = {:.3};  f_m,y,d = {:.3} × {} × {} / {}", kh_t, kh_t, km, cls.ft0k, gm, kh_m, kh_m, km, cls.fmk, gm),
        &format!("f_{{t,0,d}} = {:.2} N/mm^{{2}};  f_{{m,y,d}} = {:.2} N/mm^{{2}}", ft0d, fmyd),
        "",
    );
    let a_net = b * h - a_hole;
    let w_net = b * h * h / 6.0 * (a_net / (b * h));
    let st = n_ed * 1e3 / a_net;
    let sm = my * 1e6 / w_net;
    let u_t = st / ft0d;
    let u_c = st / ft0d + sm / fmyd;
    t.step(
        s3,
        "Design stresses on the net section",
        "EN 1995-1-1 cl. 5.2(3), 6.1.2, 6.1.6",
        "A_{net} = b·h − A_{holes};   W_{net} = (b·h^{2}/6)·A_{net}/(b·h);   σ_{t,0,d} = N_{t,d}/A_{net};   σ_{m,y,d} = M_{y,d}/W_{net}",
        &format!(
            "A_net = {:.0} mm^{{2}};  W_net = {:.0} mm^{{3}};  σ_t,0,d = {}×10^{{3}}/{:.0};  σ_m,y,d = {}×10^{{6}}/{:.0}",
            a_net, w_net, n_ed, a_net, my, w_net
        ),
        &format!("σ_{{t,0,d}} = {:.2};  σ_{{m,y,d}} = {:.2} N/mm^{{2}}", st, sm),
        "",
    );
    t.step(
        s3,
        "Tension parallel to the grain",
        "EN 1995-1-1 cl. 6.1.2, Eq 6.1",
        "σ_{t,0,d} ≤ f_{t,0,d}",
        &format!("{:.2} / {:.2}", st, ft0d),
        &format!("η = {:.3}", u_t),
        "",
    );
    t.step(
        s3,
        "Combined bending and axial tension",
        "EN 1995-1-1 cl. 6.2.3, Eq 6.17 (σ_m,z,d = 0)",
        "σ_{t,0,d}/f_{t,0,d} + σ_{m,y,d}/f_{m,y,d} ≤ 1",
        &format!("{:.2}/{:.2} + {:.2}/{:.2}", st, ft0d, sm, fmyd),
        &format!("{:.3}", u_c),
        "",
    );
    let checks = vec![
        chk(
            "Tension parallel to grain",
            "EN 1995-1-1 6.1.2 Eq 6.1",
            format!("{:.2} N/mm²", st),
            format!("{:.2} N/mm²", ft0d),
            u_t,
            u_t <= 1.0,
        ),
        chk("Tension and bending, Eq 6.17", "EN 1995-1-1 6.2.3", format!("{:.3}", u_c), "1.000".into(), u_c, u_c <= 1.0),
    ];
    let summary = vec![
        kv("Strength class", cls.name.to_string()),
        kv("k mod / γ M", format!("{:.2} / {:.2}", km, gm)),
        kv("A net / W net", format!("{:.0} mm² / {:.0} mm³", a_net, w_net)),
        kv("f t,0,d / f m,y,d", format!("{:.2} / {:.2} N/mm²", ft0d, fmyd)),
        kv("Tension capacity N t,Rd", format!("{:.1} kN", ft0d * a_net / 1e3)),
    ];
    let mut r = finish(
        t,
        checks,
        &ps.status,
        summary,
        vec![],
        vec![],
        vec![
            "Implemented: tension parallel to the grain with k h (Eq 6.1), combined tension and major-axis bending (Eq 6.17), net section at holes.".into(),
            "NOT implemented: minor-axis bending (Eq 6.17/6.18 with k m), tension perpendicular to the grain, connection design, fire.".into(),
            "A PASS applies only to the checks listed and within the stated assumptions. It is not an approval by a registered engineer.".into(),
        ],
        Drawing::Rect { b, h },
    );
    r.outputs = vec![out("n_ed", "Design tension N t,d", n_ed, "kN"), out("ntrd", "Tension capacity N t,Rd", ft0d * a_net / 1e3, "kN")];
    r
}
