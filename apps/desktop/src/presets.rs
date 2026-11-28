//! Input presets: standard values that fill a module's existing inputs in one step (mirrors apps/web/lib/presets.ts).
//! They never change the engine or its formulas; every value stays editable afterwards. A group is offered only
//! when the module has all of the input keys it sets.
use strucal_engine::module::ModDef;

pub struct PresetOption {
    pub label: String,
    pub hint: String,
    pub values: Vec<(String, f64)>,
}
pub struct PresetGroup {
    pub id: String,
    pub title: String,
    pub source: &'static str,
    pub options: Vec<PresetOption>,
}

const CONCRETE: [(&str, f64); 9] = [
    ("C20/25", 20.0),
    ("C25/30", 25.0),
    ("C28/35", 28.0),
    ("C30/37", 30.0),
    ("C32/40", 32.0),
    ("C35/45", 35.0),
    ("C40/50", 40.0),
    ("C45/55", 45.0),
    ("C50/60", 50.0),
];

// EN 1993-1-1 Table 3.1, nominal values for t ≤ 40 mm.
const STEEL: [(&str, f64, f64); 3] = [("S235", 235.0, 360.0), ("S275", 275.0, 430.0), ("S355", 355.0, 490.0)];

// Hot-rolled sections, EN 10365 dimensions: h, b, tw, tf, r (mm).
const SECTIONS: [(&str, f64, f64, f64, f64, f64); 23] = [
    ("IPE 160", 160.0, 82.0, 5.0, 7.4, 9.0),
    ("IPE 180", 180.0, 91.0, 5.3, 8.0, 9.0),
    ("IPE 200", 200.0, 100.0, 5.6, 8.5, 12.0),
    ("IPE 220", 220.0, 110.0, 5.9, 9.2, 12.0),
    ("IPE 240", 240.0, 120.0, 6.2, 9.8, 15.0),
    ("IPE 270", 270.0, 135.0, 6.6, 10.2, 15.0),
    ("IPE 300", 300.0, 150.0, 7.1, 10.7, 15.0),
    ("IPE 330", 330.0, 160.0, 7.5, 11.5, 18.0),
    ("IPE 360", 360.0, 170.0, 8.0, 12.7, 18.0),
    ("IPE 400", 400.0, 180.0, 8.6, 13.5, 21.0),
    ("IPE 450", 450.0, 190.0, 9.4, 14.6, 21.0),
    ("IPE 500", 500.0, 200.0, 10.2, 16.0, 21.0),
    ("HEB 100", 100.0, 100.0, 6.0, 10.0, 12.0),
    ("HEB 120", 120.0, 120.0, 6.5, 11.0, 12.0),
    ("HEB 140", 140.0, 140.0, 7.0, 12.0, 12.0),
    ("HEB 160", 160.0, 160.0, 8.0, 13.0, 15.0),
    ("HEB 180", 180.0, 180.0, 8.5, 14.0, 15.0),
    ("HEB 200", 200.0, 200.0, 9.0, 15.0, 18.0),
    ("HEB 220", 220.0, 220.0, 9.5, 16.0, 18.0),
    ("HEB 240", 240.0, 240.0, 10.0, 17.0, 21.0),
    ("HEB 260", 260.0, 260.0, 10.0, 17.5, 24.0),
    ("HEB 280", 280.0, 280.0, 10.5, 18.0, 24.0),
    ("HEB 300", 300.0, 300.0, 11.0, 19.0, 27.0),
];

// EN 1990 Table A1.1 recommended ψ factors for buildings.
const PSI: [(&str, f64, f64, f64); 11] = [
    ("Cat. A — domestic, residential", 0.7, 0.5, 0.3),
    ("Cat. B — offices", 0.7, 0.5, 0.3),
    ("Cat. C — congregation areas", 0.7, 0.7, 0.6),
    ("Cat. D — shopping areas", 0.7, 0.7, 0.6),
    ("Cat. E — storage areas", 1.0, 0.9, 0.8),
    ("Cat. F — traffic, vehicle ≤ 30 kN", 0.7, 0.7, 0.6),
    ("Cat. G — traffic, 30–160 kN", 0.7, 0.5, 0.3),
    ("Cat. H — roofs", 0.0, 0.0, 0.0),
    ("Snow (site altitude ≤ 1000 m)", 0.5, 0.2, 0.0),
    ("Wind", 0.6, 0.2, 0.0),
    ("Temperature (non-fire)", 0.6, 0.5, 0.0),
];

fn has(def: &ModDef, keys: &[&str]) -> bool {
    keys.iter().all(|k| def.fields.iter().any(|f| f.key == *k))
}

fn kv(pairs: &[(&str, f64)]) -> Vec<(String, f64)> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// Preset groups applicable to a module.
pub fn presets_for(def: &ModDef) -> Vec<PresetGroup> {
    let mut out = vec![];
    if has(def, &["h", "b", "tw", "tf", "r"]) {
        out.push(PresetGroup {
            id: "section".into(),
            title: "Rolled section".into(),
            source: "EN 10365",
            options: SECTIONS
                .iter()
                .map(|(n, h, b, tw, tf, r)| PresetOption {
                    label: n.to_string(),
                    hint: format!("{h}×{b}"),
                    values: kv(&[("h", *h), ("b", *b), ("tw", *tw), ("tf", *tf), ("r", *r)]),
                })
                .collect(),
        });
    }
    if has(def, &["fck"]) {
        out.push(PresetGroup {
            id: "concrete".into(),
            title: "Concrete class".into(),
            source: "EN 1992-1-1 Table 3.1",
            options: CONCRETE
                .iter()
                .map(|(n, fck)| PresetOption { label: n.to_string(), hint: format!("fck {fck}"), values: kv(&[("fck", *fck)]) })
                .collect(),
        });
    }
    let with_fu = has(def, &["fy", "fu"]);
    if with_fu || has(def, &["fy"]) {
        out.push(PresetGroup {
            id: "steel".into(),
            title: "Steel grade".into(),
            source: "EN 1993-1-1 Table 3.1 (t ≤ 40 mm)",
            options: STEEL
                .iter()
                .map(|(n, fy, fu)| PresetOption {
                    label: n.to_string(),
                    hint: if with_fu { format!("fy {fy} · fu {fu}") } else { format!("fy {fy}") },
                    values: if with_fu { kv(&[("fy", *fy), ("fu", *fu)]) } else { kv(&[("fy", *fy)]) },
                })
                .collect(),
        });
    }
    for i in 1..=3 {
        let (k0, k1, k2) = (format!("p0{i}"), format!("p1{i}"), format!("p2{i}"));
        if has(def, &[&k0, &k1, &k2]) {
            out.push(PresetGroup {
                id: format!("psi{i}"),
                title: format!("ψ factors · variable action {i}"),
                source: "EN 1990 Table A1.1",
                options: PSI
                    .iter()
                    .map(|(n, a, b, c)| PresetOption {
                        label: n.to_string(),
                        hint: format!("{a} · {b} · {c}"),
                        values: vec![(k0.clone(), *a), (k1.clone(), *b), (k2.clone(), *c)],
                    })
                    .collect(),
            });
        }
    }
    out
}

/// Index of the option whose values all match the current inputs.
pub fn matching(group: &PresetGroup, def: &ModDef, vals: &[f64]) -> Option<usize> {
    group
        .options
        .iter()
        .position(|o| o.values.iter().all(|(k, v)| def.fields.iter().position(|f| f.key == k).is_some_and(|i| (vals[i] - v).abs() < 1e-9)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use strucal_engine::modules;

    #[test]
    fn every_preset_targets_existing_inputs() {
        for d in modules::defs() {
            for g in presets_for(&d) {
                for o in &g.options {
                    for (k, _) in &o.values {
                        assert!(d.fields.iter().any(|f| f.key == k), "{} preset {} sets missing key {k}", d.id, g.id);
                    }
                }
            }
        }
    }

    #[test]
    fn rc_beam_offers_concrete_classes() {
        let d = modules::defs().into_iter().find(|d| d.id == "beam").expect("RC beam module");
        assert!(presets_for(&d).iter().any(|g| g.id == "concrete"));
    }
}
