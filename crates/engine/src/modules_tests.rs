#[cfg(test)]
mod tests {
    use crate::modules::*;
    use serde_json::json;
    fn slab() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","span":4000,"h":175,"cover":25,"bar_dia":12,"fck":30,"fyk":500,"gk":1.5,"qk":3}) }
    fn pad() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","n_ed":1200,"n_k":870,"q_allow":200,"cx":300,"cy":300,"h":500,"cover":50,"bar_dia":16,"fck":30,"fyk":500}) }
    #[test] fn slab_hand_calc() { let r = run("slab", &slab()); assert!(r.ok, "{:?}", r.errors); let s = &r.summary; assert!(s[3].value.starts_with("41") || s[3].value.starts_with("42"), "{}", s[3].value); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test] fn slab_fails_thin() { let mut v = slab(); v["h"] = json!(100); v["span"] = json!(6000); let r = run("slab", &v); assert!(!r.ok || !r.overall_pass); }
    #[test] fn pad_hand_calc() { let r = run("pad", &pad()); assert!(r.ok, "{:?}", r.errors); assert!(r.summary[0].value.starts_with("2200 × 2200"), "{}", r.summary[0].value); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test] fn pad_check_mode_small_fails() { let mut v = pad(); v["mode"] = json!("check"); v["bx"] = json!(1500); v["by"] = json!(1500); v["bar_spacing"] = json!(200); let r = run("pad", &v); assert!(r.ok && !r.overall_pass); }
    #[test] fn deterministic() { assert_eq!(run("pad", &pad()).trace_hash, run("pad", &pad()).trace_hash); }
    #[test] fn unknown_module() { assert!(!run("zzz", &pad()).ok); }
    #[test] fn beam_via_registry() { let v = json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","b":300,"h":500,"cover":30,"link_dia":8,"bar_dia":20,"fck":30,"fyk":500,"m_ed":200,"v_ed":120}); let r = run("beam", &v); assert!(r.ok && r.overall_pass); }
}

#[cfg(test)]
mod column_tests {
    use crate::modules::*;
    use serde_json::json;
    fn col() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","b":300,"h":400,"len":3500,"beta":1.0,"cover":35,"link_dia":8,"bar_dia":20,"fck":30,"fyk":500,"phi_ef":1.0,"n_ed":1500,"m02":80,"m01":40}) }
    #[test] fn column_designs_and_passes() { let r = run("column", &col()); assert!(r.ok, "{:?}", r.errors); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test] fn pure_axial_capacity_exceeds_concrete() { let r = run("column", &col()); let s = r.summary.iter().find(|k| k.label.starts_with("Axial capacity")).unwrap(); let v: f64 = s.value.split_whitespace().next().unwrap().parse().unwrap(); assert!(v > 0.85 * 30.0 / 1.5 * 300.0 * 400.0 / 1e3, "{}", v); }
    #[test] fn overloaded_column_fails() { let mut v = col(); v["n_ed"] = json!(9000); let r = run("column", &v); assert!(!r.ok || !r.overall_pass); }
    #[test] fn check_mode_underreinforced_fails() { let mut v = col(); v["mode"] = json!("check"); v["n_face"] = json!(2); v["link_spacing"] = json!(250); v["m02"] = json!(250); v["m01"] = json!(100); let r = run("column", &v); assert!(r.ok && !r.overall_pass); }
    #[test] fn rejects_bad_end_moments() { let mut v = col(); v["m01"] = json!(100); assert!(!run("column", &v).ok); }
    #[test] fn slender_gets_second_order() { let mut v = col(); v["len"] = json!(7000); let r = run("column", &v); assert!(r.ok); assert!(r.trace.steps.iter().any(|s| s.title.contains("nominal curvature"))); }
}
