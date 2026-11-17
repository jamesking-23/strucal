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

#[cfg(test)]
mod more_tests {
    use crate::modules::*;
    use serde_json::json;
    #[test] fn strip_width_from_bearing() { let r = run("strip", &json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","n_ed":300,"n_k":220,"q_allow":150,"t":225,"h":400,"cover":50,"bar_dia":12,"fck":30,"fyk":500})); assert!(r.ok, "{:?}", r.errors); assert!(r.summary[0].value.starts_with("1600"), "{}", r.summary[0].value); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test] fn strip_check_narrow_fails() { let r = run("strip", &json!({"mode":"check","param_set":"EN1992-1-1:2004/RECOMMENDED","n_ed":300,"n_k":220,"q_allow":150,"t":225,"h":400,"cover":50,"bar_dia":12,"fck":30,"fyk":500,"width":1000,"bar_spacing":300})); assert!(r.ok && !r.overall_pass); }
    #[test] fn stairs_design() { let r = run("stairs", &json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","span":3000,"h":150,"rise":175,"going":250,"cover":25,"bar_dia":12,"fck":30,"fyk":500,"gk":1.5,"qk":4})); assert!(r.ok, "{:?}", r.errors); assert!(r.trace.steps.iter().any(|s| s.title.contains("per m² of plan"))); }
    #[test] fn stairs_overloaded_fails() { let r = run("stairs", &json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","span":6000,"h":120,"rise":175,"going":250,"cover":25,"bar_dia":12,"fck":30,"fyk":500,"gk":1.5,"qk":4})); assert!(!r.ok || !r.overall_pass); }
}

#[cfg(test)]
mod wall_tests {
    use crate::modules::*;
    use serde_json::json;
    fn w() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","h":200,"len":3000,"beta":1.0,"cover":30,"bar_dia":12,"hbar_dia":10,"fck":30,"fyk":500,"phi_ef":1.0,"n_ed":600,"m02":12,"m01":6}) }
    #[test] fn wall_designs_and_passes() { let r = run("wall", &w()); assert!(r.ok, "{:?}", r.errors); assert!(r.overall_pass, "{:?}", r.checks); }
    #[test] fn wall_overloaded_fails() { let mut v = w(); v["n_ed"] = json!(5000); let r = run("wall", &v); assert!(!r.ok || !r.overall_pass); }
    #[test] fn wall_check_sparse_fails_min_steel() { let mut v = w(); v["mode"] = json!("check"); v["spacing"] = json!(400); v["bar_dia"] = json!(8); let r = run("wall", &v); assert!(r.ok && !r.overall_pass); }
    #[test] fn tall_wall_gets_second_order() { let mut v = w(); v["len"] = json!(6000); let r = run("wall", &v); assert!(r.ok); assert!(r.trace.steps.iter().any(|s| s.title.contains("nominal curvature"))); }
}

#[cfg(test)]
mod combo_tests {
    use crate::modules::*;
    use serde_json::json;
    fn c() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","gk":100,"q1":50,"p01":0.7,"p11":0.5,"p21":0.3,"q2":30,"p02":0.6,"p12":0.5,"p22":0.3,"q3":0,"p03":0.5,"p13":0.2,"p23":0}) }
    fn val(r: &crate::module::ModResult, label: &str) -> f64 { r.summary.iter().find(|k| k.label.starts_with(label)).unwrap().value.split_whitespace().next().unwrap().parse().unwrap() }
    #[test] fn uls_610() { let r = run("combos", &c()); assert!(r.ok, "{:?}", r.errors); assert!((val(&r, "ULS design value, Eq 6.10") - 237.0).abs() < 0.01); }
    #[test] fn uls_610ab() { let r = run("combos", &c()); assert!((val(&r, "ULS design value, 6.10a/b") - 216.75).abs() < 0.01); }
    #[test] fn sls_combinations() { let r = run("combos", &c()); assert!((val(&r, "SLS characteristic") - 168.0).abs() < 0.01); assert!((val(&r, "SLS frequent") - 134.0).abs() < 0.01); assert!((val(&r, "SLS quasi-permanent") - 124.0).abs() < 0.01); }
    #[test] fn rejects_psi_order() { let mut v = c(); v["p11"] = json!(0.9); assert!(!run("combos", &v).ok); }
    #[test] fn needs_a_variable_action() { let mut v = c(); v["q1"] = json!(0); v["q2"] = json!(0); assert!(!run("combos", &v).ok); }
    #[test] fn report_renders_without_checks() { let h = crate::report::html_json("combos", &c().to_string()); assert!(h.contains("RESULTS") && !h.contains("Summary of checks")); }
}

#[cfg(test)]
mod steel_tests {
    use crate::modules::*;
    use serde_json::json;
    fn s() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","span":6000,"lcr":6000,"load_level":1,"h":300,"b":150,"tw":7.1,"tf":10.7,"r":15,"fy":275,"gk":10,"qk":12,"defl_ratio":360}) }
    #[test] fn ipe300_properties_match_published() {
        // IPE 300 published (ArcelorMittal): A = 53.81 cm2, Iy = 8356 cm4, Iz = 603.8 cm4, Wpl,y = 628.4 cm3
        let p = crate::steel_beam::props(300.0, 150.0, 7.1, 10.7, 15.0);
        assert!((p.a - 5381.0).abs() / 5381.0 < 0.005, "A {}", p.a);
        assert!((p.iy - 83.56e6).abs() / 83.56e6 < 0.01, "Iy {}", p.iy);
        assert!((p.iz - 6.038e6).abs() / 6.038e6 < 0.02, "Iz {}", p.iz);
        assert!((p.wpl - 628.4e3).abs() / 628.4e3 < 0.01, "Wpl {}", p.wpl);
    }
    #[test] fn steel_beam_runs() { let r = run("steelbeam", &s()); assert!(r.ok, "{:?}", r.errors); assert_eq!(r.checks.len(), 5); }
    #[test] fn short_lcr_improves_ltb() { let a = run("steelbeam", &s()); let mut v = s(); v["lcr"] = json!(2000); let b = run("steelbeam", &v); let u = |r: &crate::module::ModResult| r.checks.iter().find(|c| c.name.starts_with("Lateral")).unwrap().utilisation; assert!(u(&b) < u(&a)); }
    #[test] fn overloaded_fails() { let mut v = s(); v["qk"] = json!(60); assert!(!run("steelbeam", &v).overall_pass); }
    #[test] fn class4_rejected() { let r = run("steelbeam", &json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","span":6000,"lcr":6000,"load_level":1,"h":600,"b":200,"tw":4,"tf":8,"r":0.5,"fy":355,"gk":10,"qk":12,"defl_ratio":360})); assert!(!r.ok); }
}

#[cfg(test)]
mod steelcol_tests {
    use crate::modules::*;
    use serde_json::json;
    fn c() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","n_ed":800,"len":4000,"ky":1.0,"kz":1.0,"h":200,"b":200,"tw":9,"tf":15,"r":18,"fy":275}) }
    #[test] fn heb200_properties_match_published() {
        // HEB 200 published: A = 78.08 cm2, Iy = 5696 cm4, Iz = 2003 cm4
        let p = crate::steel_beam::props(200.0, 200.0, 9.0, 15.0, 18.0);
        assert!((p.a - 7808.0).abs() / 7808.0 < 0.005, "A {}", p.a);
        assert!((p.iy - 56.96e6).abs() / 56.96e6 < 0.01, "Iy {}", p.iy);
        assert!((p.iz - 20.03e6).abs() / 20.03e6 < 0.01, "Iz {}", p.iz);
    }
    #[test] fn hand_calc_weak_axis() { // Ncr=2595 kN, lambda=0.910, curve c, chi=0.594, Nb=1275 kN
        let r = run("steelcol", &c()); assert!(r.ok, "{:?}", r.errors);
        let k = r.checks.iter().find(|k| k.name.ends_with("z-z")).unwrap(); let cap: f64 = k.capacity.split_whitespace().next().unwrap().parse().unwrap();
        assert!((cap - 1275.0).abs() < 20.0, "{}", cap); assert!(r.overall_pass); }
    #[test] fn curves_per_table_6_2() { assert_eq!(crate::steel_column::curve(300.0, 150.0, 10.7, true).0, "a"); assert_eq!(crate::steel_column::curve(300.0, 150.0, 10.7, false).0, "b"); assert_eq!(crate::steel_column::curve(200.0, 200.0, 15.0, true).0, "b"); assert_eq!(crate::steel_column::curve(200.0, 200.0, 15.0, false).0, "c"); }
    #[test] fn overloaded_fails() { let mut v = c(); v["n_ed"] = json!(2500); assert!(!run("steelcol", &v).overall_pass); }
    #[test] fn longer_column_buckles_more() { let a = run("steelcol", &c()); let mut v = c(); v["len"] = json!(8000); let b = run("steelcol", &v); let u = |r: &crate::module::ModResult| r.checks.iter().find(|k| k.name.ends_with("z-z")).unwrap().utilisation; assert!(u(&b) > u(&a)); }
}

#[cfg(test)]
mod retaining_tests {
    use crate::modules::*;
    use serde_json::json;
    fn r() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","hs":3000,"ts":350,"tb":400,"lt":800,"bw":3200,"gamma":18,"phi":30,"delta":30,"q_allow":200,"q_sur":10,"hw":0,"cs":40,"cb":50,"ps":16,"pb":16,"fck":30,"fyk":500}) }
    fn u(res: &crate::module::ModResult, name: &str) -> f64 { res.checks.iter().find(|c| c.name.starts_with(name)).unwrap().utilisation }
    #[test] fn stability_matches_independent_calc() {
        let x = run("retaining", &r()); assert!(x.ok, "{:?}", x.errors);
        assert!((u(&x, "Sliding, DA1-1") - 0.6543).abs() < 0.005, "{}", u(&x, "Sliding, DA1-1"));
        assert!((u(&x, "Sliding, DA1-2") - 0.7772).abs() < 0.005);
        assert!((u(&x, "Overturning") - 0.3098).abs() < 0.005);
        let b = x.checks.iter().find(|c| c.name == "Bearing pressure").unwrap(); let q: f64 = b.demand.split_whitespace().next().unwrap().parse().unwrap(); assert!((q - 59.4).abs() < 0.2, "{}", q);
    }
    #[test] fn default_wall_passes() { let x = run("retaining", &r()); assert!(x.overall_pass, "{:?}", x.checks.iter().filter(|c| !c.pass).map(|c| (&c.name, c.utilisation)).collect::<Vec<_>>()); }
    #[test] fn narrow_base_fails_stability() { let mut v = r(); v["bw"] = json!(1500); v["lt"] = json!(300); let x = run("retaining", &v); assert!(!x.ok || !x.overall_pass); }
    #[test] fn water_increases_demand_and_is_disclosed() { let a = run("retaining", &r()); let mut v = r(); v["hw"] = json!(2000); let b = run("retaining", &v); assert!(u(&b, "Sliding, DA1-1") > u(&a, "Sliding, DA1-1")); assert!(a.observations.iter().any(|o| o.text.contains("Drained"))); }
    #[test] fn rejects_delta_above_phi() { let mut v = r(); v["delta"] = json!(35); assert!(!run("retaining", &v).ok); }
    #[test] fn deterministic() { assert_eq!(run("retaining", &r()).trace_hash, run("retaining", &r()).trace_hash); }
}

#[cfg(test)]
mod raft_tests {
    use crate::modules::*;
    use serde_json::json;
    fn r() -> serde_json::Value { json!({"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","n_cols":3,"spacing":5000,"edge":1000,"width":5000,"n_ed":1500,"n_k":1100,"q_allow":150,"cx":400,"cy":400,"h":800,"cover":50,"bar_dia":20,"fck":30,"fyk":500}) }
    fn kvv(res: &crate::module::ModResult, label: &str) -> String { res.summary.iter().find(|k| k.label.starts_with(label)).unwrap().value.clone() }
    #[test] fn strip_statics_match_independent_calc() {
        let x = run("raft", &r()); assert!(x.ok, "{:?}", x.errors);
        let sag: f64 = kvv(&x, "Max sagging").split_whitespace().next().unwrap().parse().unwrap();
        let hog: f64 = kvv(&x, "Max hogging").split_whitespace().next().unwrap().parse().unwrap();
        assert!((sag - 187.0).abs() < 1.5, "{}", sag); assert!((hog - 1500.0).abs() < 2.0, "{}", hog);
    }
    #[test] fn pressure_matches() { let x = run("raft", &r()); assert!(kvv(&x, "Service").starts_with("75.0 / 75.0"), "{}", kvv(&x, "Service")); }
    #[test] fn default_passes() { let x = run("raft", &r()); assert!(x.overall_pass, "{:?}", x.checks.iter().filter(|c| !c.pass).map(|c| (&c.name, c.utilisation)).collect::<Vec<_>>()); }
    #[test] fn weak_soil_fails_bearing() { let mut v = r(); v["q_allow"] = json!(50); let x = run("raft", &v); assert!(!x.overall_pass); }
    #[test] fn two_columns_has_no_punching_check() { let mut v = r(); v["n_cols"] = json!(2); let x = run("raft", &v); assert!(x.ok); assert!(!x.checks.iter().any(|c| c.name.starts_with("Punching"))); }
    #[test] fn rejects_fractional_columns() { let mut v = r(); v["n_cols"] = json!(3.5); assert!(!run("raft", &v).ok); }
}
