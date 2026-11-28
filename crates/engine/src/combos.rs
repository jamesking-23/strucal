//! Load combination engine, EN 1990:2002 (persistent design situation, STR) and SLS combinations.
//! Works on the effect of each action (moment, shear, axial force or reaction) in one consistent unit.
use crate::module::*;
use crate::params;
use crate::rc_beam::Obs;
use crate::trace::Trace;
use serde_json::Value;

pub fn def() -> ModDef {
    ModDef {
        id: "combos",
        name: "Load combinations",
        title: "Load combinations (EN 1990)",
        subtitle: "EN 1990:2002 · Persistent situation, STR (Eq 6.10, 6.10a/b) and SLS combinations of action effects",
        available: true,
        has_check_mode: false,
        fields: vec![
            fld("gk", "Permanent action effect G k", "effect", "Permanent actions", 100.0),
            fld("q1", "Variable action 1: effect Q k,1", "effect", "Variable action 1", 50.0),
            fld("p01", "ψ0", "-", "Variable action 1", 0.7),
            fld("p11", "ψ1", "-", "Variable action 1", 0.5),
            fld("p21", "ψ2", "-", "Variable action 1", 0.3),
            fld("q2", "Variable action 2: effect Q k,2 (0 if unused)", "effect", "Variable action 2", 30.0),
            fld("p02", "ψ0", "-", "Variable action 2", 0.6),
            fld("p12", "ψ1", "-", "Variable action 2", 0.2),
            fld("p22", "ψ2", "-", "Variable action 2", 0.0),
            fld("q3", "Variable action 3: effect Q k,3 (0 if unused)", "effect", "Variable action 3", 0.0),
            fld("p03", "ψ0", "-", "Variable action 3", 0.5),
            fld("p13", "ψ1", "-", "Variable action 3", 0.2),
            fld("p23", "ψ2", "-", "Variable action 3", 0.0),
        ],
    }
}

struct A {
    name: String,
    q: f64,
    p0: f64,
    p1: f64,
    p2: f64,
}

pub fn run(v: &Value) -> ModResult {
    let ps = match v.get("param_set").and_then(|p| p.as_str()).and_then(params::get) {
        Some(p) => p,
        None => return fail(vec!["Unknown or unavailable parameter set - select one explicitly".into()]),
    };
    let g = match num(v, "gk") {
        Some(x) if x.is_finite() && x >= 0.0 => x,
        _ => return fail(vec!["Permanent action effect G k must be a number >= 0".into()]),
    };
    let mut acts: Vec<A> = vec![];
    let mut errs = vec![];
    for (i, (qk, a, b, c)) in [("q1", "p01", "p11", "p21"), ("q2", "p02", "p12", "p22"), ("q3", "p03", "p13", "p23")].iter().enumerate() {
        let q = num(v, qk).unwrap_or(0.0);
        if !q.is_finite() || q < 0.0 {
            errs.push(format!("Variable action {} must be >= 0", i + 1));
            continue;
        }
        let (p0, p1, p2) = (num(v, a).unwrap_or(0.0), num(v, b).unwrap_or(0.0), num(v, c).unwrap_or(0.0));
        for (n, p) in [("ψ0", p0), ("ψ1", p1), ("ψ2", p2)] {
            if !(0.0..=1.0).contains(&p) {
                errs.push(format!("{} of variable action {} must be between 0 and 1", n, i + 1));
            }
        }
        if p1 > p0 || p2 > p1 {
            errs.push(format!("Variable action {}: expect ψ0 ≥ ψ1 ≥ ψ2", i + 1));
        }
        if q > 0.0 {
            acts.push(A { name: format!("Q{}", i + 1), q, p0, p1, p2 });
        }
    }
    if !errs.is_empty() {
        return fail(errs);
    }
    if acts.is_empty() {
        return fail(vec!["Enter at least one variable action greater than zero".into()]);
    }
    let (gg, gq, xi) = (ps.gamma_g, ps.gamma_q, 0.85);
    let mut t = Trace::new();
    let (s1, s2, s3, s4) = ("1. Actions and factors", "2. ULS: Eq 6.10", "3. ULS: Eq 6.10a and 6.10b", "4. SLS combinations");
    t.step(
        s1,
        "Action effects",
        "",
        "",
        &format!(
            "G_{{k}} = {};  {}",
            g,
            acts.iter()
                .map(|a| format!("{} = {} (ψ0 = {}, ψ1 = {}, ψ2 = {})", a.name, a.q, a.p0, a.p1, a.p2))
                .collect::<Vec<_>>()
                .join(";  ")
        ),
        "",
        "All effects must be in the same unit and have the same sign convention (for example, sagging moment at one section).",
    );
    t.step(
        s1,
        "Partial factors",
        "EN 1990 Table A1.2(B), 6.4.3.2",
        "",
        &format!("γ_{{G}} = {:.2};  γ_{{Q}} = {:.2};  ξ = {:.2}", gg, gq, xi),
        "",
        &format!(
            "Parameter set: {} [{}]. ψ values must be taken from EN 1990 Table A1.1 and the National Annex for the action category.",
            ps.label, ps.status
        ),
    );
    // ULS 6.10
    let mut best610 = (0.0, 0usize);
    for (i, lead) in acts.iter().enumerate() {
        let comp: f64 = acts.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, a)| gq * a.p0 * a.q).sum();
        let e = gg * g + gq * lead.q + comp;
        t.step(
            s2,
            &format!("Leading action: {}", lead.name),
            "EN 1990 Eq 6.10",
            "E_d = γ_G·G_k + γ_Q·Q_{k,1} + Σ γ_Q·ψ_0,i·Q_{k,i}",
            &format!("= {:.2} × {} + {:.2} × {} + {:.2}", gg, g, gq, lead.q, comp),
            &format!("E_{{d}} = {:.2}", e),
            "",
        );
        if e > best610.0 {
            best610 = (e, i);
        }
    }
    t.step(
        s2,
        "Governing ULS value, Eq 6.10",
        "EN 1990 cl. 6.4.3.2",
        "E_{d,6.10} = max over leading actions",
        "",
        &format!("E_{{d,6.10}} = {:.2}  (leading {})", best610.0, acts[best610.1].name),
        "",
    );
    // 6.10a / 6.10b
    let comp_all: f64 = acts.iter().map(|a| gq * a.p0 * a.q).sum();
    let e_a = gg * g + comp_all;
    t.step(
        s3,
        "Eq 6.10a",
        "EN 1990 Eq 6.10a",
        "E_d = γ_G·G_k + Σ γ_Q·ψ_0,i·Q_{k,i}",
        &format!("= {:.2} × {} + {:.2}", gg, g, comp_all),
        &format!("E_{{d,6.10a}} = {:.2}", e_a),
        "",
    );
    let mut best_b = (0.0, 0usize);
    for (i, lead) in acts.iter().enumerate() {
        let comp: f64 = acts.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, a)| gq * a.p0 * a.q).sum();
        let e = xi * gg * g + gq * lead.q + comp;
        t.step(
            s3,
            &format!("Eq 6.10b, leading action: {}", lead.name),
            "EN 1990 Eq 6.10b",
            "E_d = ξ·γ_G·G_k + γ_Q·Q_{k,1} + Σ γ_Q·ψ_0,i·Q_{k,i}",
            &format!("= {:.2} × {:.2} × {} + {:.2} × {} + {:.2}", xi, gg, g, gq, lead.q, comp),
            &format!("E_{{d,6.10b}} = {:.2}", e),
            "",
        );
        if e > best_b.0 {
            best_b = (e, i);
        }
    }
    let e_ab = e_a.max(best_b.0);
    t.step(
        s3,
        "Governing ULS value, 6.10a/b",
        "EN 1990 cl. 6.4.3.2(3)",
        "E_{d,6.10a/b} = max(E_{d,6.10a} ; E_{d,6.10b})",
        &format!("= max({:.2} ; {:.2})", e_a, best_b.0),
        &format!("E_{{d,6.10a/b}} = {:.2}", e_ab),
        "The less favourable of 6.10a and 6.10b applies. Eq 6.10 is always on the safe side of it.",
    );
    let e_fav = 1.0 * g;
    t.step(s3, "Favourable permanent action (e.g. uplift or reduced restraint)", "EN 1990 Table A1.2(B)", "E_d = γ_{G,inf}·G_k  with γ_{G,inf} = 1.0, no variable action", &format!("= 1.0 × {}", g), &format!("E_{{d,fav}} = {:.2}", e_fav), "Use where the permanent action reduces the effect being checked. EQU (static equilibrium) factors of Table A1.2(A) are not applied here.");
    // SLS
    let mut char_best = (0.0, 0usize);
    let mut freq_best = (0.0, 0usize);
    for (i, lead) in acts.iter().enumerate() {
        let c: f64 = g + lead.q + acts.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, a)| a.p0 * a.q).sum::<f64>();
        let f: f64 = g + lead.p1 * lead.q + acts.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, a)| a.p2 * a.q).sum::<f64>();
        if c > char_best.0 {
            char_best = (c, i);
        }
        if f > freq_best.0 {
            freq_best = (f, i);
        }
    }
    t.step(
        s4,
        "Characteristic combination",
        "EN 1990 Eq 6.14b",
        "E_{d,char} = G_k + Q_{k,1} + Σ ψ_0,i·Q_{k,i}",
        "maximum over leading actions",
        &format!("E_{{d,char}} = {:.2}  (leading {})", char_best.0, acts[char_best.1].name),
        "Irreversible limit states.",
    );
    t.step(
        s4,
        "Frequent combination",
        "EN 1990 Eq 6.15b",
        "E_{d,freq} = G_k + ψ_1,1·Q_{k,1} + Σ ψ_2,i·Q_{k,i}",
        "maximum over leading actions",
        &format!("E_{{d,freq}} = {:.2}  (leading {})", freq_best.0, acts[freq_best.1].name),
        "Reversible limit states.",
    );
    let qp: f64 = g + acts.iter().map(|a| a.p2 * a.q).sum::<f64>();
    t.step(
        s4,
        "Quasi-permanent combination",
        "EN 1990 Eq 6.16b",
        "E_{d,qp} = G_k + Σ ψ_2,i·Q_{k,i}",
        &format!("= {} + {:.2}", g, qp - g),
        &format!("E_{{d,qp}} = {:.2}", qp),
        "Long-term effects and appearance.",
    );
    let hash = t.hash();
    let summary = vec![
        kv("ULS design value, Eq 6.10", format!("{:.2}  (leading {})", best610.0, acts[best610.1].name)),
        kv("ULS design value, 6.10a/b", format!("{:.2}", e_ab)),
        kv("Favourable permanent", format!("{:.2}", e_fav)),
        kv("SLS characteristic", format!("{:.2}", char_best.0)),
        kv("SLS frequent", format!("{:.2}", freq_best.0)),
        kv("SLS quasi-permanent", format!("{:.2}", qp)),
        kv("Equivalent γ on total (6.10)", format!("{:.3}", best610.0 / (g + acts.iter().map(|a| a.q).sum::<f64>()))),
    ];
    let mut obs = vec![];
    if ps.status.contains("NOT") {
        obs.push(Obs {
            id: "OBS-01".into(),
            level: "AMBER".into(),
            text: format!("Parameter set: {}. Confirm γ, ξ and ψ values in the governing National Annex.", ps.status),
        });
    }
    obs.push(Obs {
        id: format!("OBS-{:02}", obs.len() + 1),
        level: "INFO".into(),
        text: "ψ factors are user inputs: select them from EN 1990 Table A1.1 and the National Annex for each action category.".into(),
    });
    ModResult { outputs: vec![out("ed_610", "ULS design value, Eq 6.10", best610.0, "effect"), out("ed_610ab", "ULS design value, 6.10a/b", e_ab, "effect"), out("e_char", "SLS characteristic", char_best.0, "effect"), out("e_freq", "SLS frequent", freq_best.0, "effect"), out("e_qp", "SLS quasi-permanent", qp, "effect")], ok: true, param_status: ps.status.clone(), summary, observations: obs, trace: t, trace_hash: hash, engine_version: crate::ENGINE_VERSION.into(), drawing: None,
        scope_notes: vec!["Implemented: STR persistent situation (6.10, 6.10a/b), favourable permanent action, SLS characteristic, frequent and quasi-permanent combinations for up to three variable actions.".into(),
            "NOT implemented: EQU/GEO sets, accidental and seismic situations, groups of actions, combinations of effects from different sections, automatic ψ selection by category.".into(),
            "This tool combines effects linearly. It does not analyse the structure and does not give a PASS/FAIL result.".into()], ..Default::default() }
}
