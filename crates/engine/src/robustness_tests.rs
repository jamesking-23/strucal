//! Robustness sweep across every registered module: no input may panic the engine (a panic aborts the
//! WebAssembly instance in the browser), and every accepted result must be numerically finite.
use crate::modules::{self, defs, run};
use serde_json::{json, Map, Value};

const PS: &str = "EN1992-1-1:2004/RECOMMENDED";
const PROBES: [f64; 9] = [0.0, -1.0, 1e-9, 0.5, 1.0, 3.0, 1e4, 1e9, 1e300];

fn defaults(id: &str, mode: &str) -> Value {
    let d = modules::def(id).expect("registered module");
    let mut m = Map::new();
    m.insert("mode".into(), mode.into());
    m.insert("param_set".into(), PS.into());
    for f in &d.fields {
        m.insert(f.key.into(), json!(f.default));
    }
    Value::Object(m)
}

/// Returns a description of the first problem found in a result, if any.
fn problem(r: &crate::module::ModResult) -> Option<String> {
    if !r.ok {
        return if r.errors.is_empty() { Some("rejected without an error message".into()) } else { None };
    }
    if !r.max_util.is_finite() {
        return Some(format!("max_util not finite: {}", r.max_util));
    }
    for o in &r.outputs {
        if !o.value.is_finite() {
            return Some(format!("output {} not finite: {}", o.key, o.value));
        }
    }
    for c in &r.checks {
        if !c.utilisation.is_finite() {
            return Some(format!("check {} utilisation not finite", c.name));
        }
    }
    for k in &r.summary {
        if modules::has_non_finite_text(&k.value) {
            return Some(format!("summary '{}' = '{}'", k.label, k.value));
        }
    }
    None
}

#[test]
fn defaults_calculate_for_every_module() {
    for d in defs() {
        let r = run(d.id, &defaults(d.id, "design"));
        assert!(r.ok, "module {} fails on its own defaults: {:?}", d.id, r.errors);
        assert!(problem(&r).is_none(), "module {}: {:?}", d.id, problem(&r));
        if d.has_check_mode {
            let r = run(d.id, &defaults(d.id, "check"));
            assert!(r.ok, "module {} check mode fails on its own defaults: {:?}", d.id, r.errors);
        }
    }
}

#[test]
fn no_input_panics_or_yields_non_finite_results() {
    let mut failures = vec![];
    for d in defs() {
        let modes: &[&str] = if d.has_check_mode { &["design", "check"] } else { &["design"] };
        for &mode in modes {
            for f in &d.fields {
                for &p in PROBES.iter() {
                    let mut v = defaults(d.id, mode);
                    v[f.key] = json!(p);
                    let id = d.id;
                    let outcome = std::panic::catch_unwind(|| run(id, &v));
                    match outcome {
                        Err(_) => failures.push(format!("{id}/{mode}: {} = {p} panics", f.key)),
                        Ok(r) => {
                            if let Some(e) = problem(&r) {
                                failures.push(format!("{id}/{mode}: {} = {p}: {e}", f.key));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} robustness failures:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn malformed_payloads_are_rejected_not_panicking() {
    for d in defs() {
        for bad in [json!(null), json!([]), json!("x"), json!({}), json!({"mode": 3, "param_set": null})] {
            let id = d.id;
            let r = std::panic::catch_unwind(|| run(id, &bad));
            assert!(r.is_ok(), "{id} panics on {bad}");
            let r = r.unwrap();
            assert!(!r.ok && !r.errors.is_empty(), "{id} accepted malformed payload {bad}");
        }
        let mut v = defaults(d.id, "design");
        v["param_set"] = json!("NOT-A-SET");
        let r = run(d.id, &v);
        assert!(!r.ok, "{} accepted an unknown parameter set", d.id);
    }
}

#[test]
fn json_entry_points_never_emit_null_numbers() {
    for d in defs() {
        let s = modules::run_json(d.id, &defaults(d.id, "design").to_string());
        for k in ["\"value\":null", "\"utilisation\":null", "\"max_util\":null"] {
            assert!(!s.contains(k), "{} serialised a null number ({k})", d.id);
        }
    }
    let s = modules::run_json("beam", "{not json");
    assert!(s.contains("Invalid input"));
}
