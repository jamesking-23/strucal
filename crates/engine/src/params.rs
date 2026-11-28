use serde::{Deserialize, Serialize};

/// A nationally determined parameter set. Unverified sets are flagged, never silently substituted.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParamSet {
    pub id: String,
    pub label: String,
    pub gamma_c: f64,
    pub gamma_s: f64,
    pub alpha_cc: f64,
    pub alpha_cw: f64,
    pub gamma_g: f64,
    pub gamma_q: f64,
    pub gamma_m0: f64,
    pub gamma_m1: f64,
    pub status: String,
    pub source: String,
}

pub fn available() -> Vec<ParamSet> {
    vec![
        ParamSet {
            id: "EN1992-1-1:2004/RECOMMENDED".into(),
            label: "EN 1992-1-1:2004 recommended values (no national annex)".into(),
            gamma_c: 1.5,
            gamma_s: 1.15,
            alpha_cc: 1.0,
            alpha_cw: 1.0,
            gamma_g: 1.35,
            gamma_q: 1.5,
            gamma_m0: 1.0,
            gamma_m1: 1.0,
            status: "RECOMMENDED VALUES - NATIONAL ANNEX NOT VERIFIED".into(),
            source: "EN 1992-1-1:2004 cl. 2.4.2.4 Table 2.1N, 3.1.6(1)P, 6.2.3".into(),
        },
        ParamSet {
            id: "EN1992-1-1:2004/UK-NA-STYLE".into(),
            label: "Comparison set: alpha_cc = 0.85 (UK NA style) - not an Uganda annex".into(),
            gamma_c: 1.5,
            gamma_s: 1.15,
            alpha_cc: 0.85,
            alpha_cw: 1.0,
            gamma_g: 1.35,
            gamma_q: 1.5,
            gamma_m0: 1.0,
            gamma_m1: 1.0,
            status: "USER-SELECTED - NOT A UGANDAN NATIONAL ANNEX".into(),
            source: "NA to BS EN 1992-1-1 cl. NA.2.4 (for comparison only)".into(),
        },
    ]
}

pub fn get(id: &str) -> Option<ParamSet> {
    available().into_iter().find(|p| p.id == id)
}
