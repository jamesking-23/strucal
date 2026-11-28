//! Project dependency engine: calculations linked into a load path, with change propagation and impact reporting.
use crate::module::Output;
use crate::modules;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

fn one() -> f64 { 1.0 }
#[derive(Deserialize, Clone, Debug)]
pub struct Link { pub field: String, pub from: String, pub output: String, #[serde(default = "one")] pub factor: f64 }
#[derive(Deserialize, Clone, Debug)]
pub struct Node {
    pub id: String, #[serde(default)] pub name: String, pub module: String,
    #[serde(default)] pub mode: String, pub param_set: String,
    #[serde(default)] pub values: Map<String, Value>, #[serde(default)] pub links: Vec<Link>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Project { pub nodes: Vec<Node> }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Resolved { pub field: String, pub from: String, pub output: String, pub value: f64 }
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeResult {
    pub id: String, pub name: String, pub module: String,
    /// "calculated" | "blocked" | "error"
    pub status: String, pub errors: Vec<String>,
    pub overall_pass: bool, pub has_verdict: bool, pub max_util: f64, pub trace_hash: String, pub own_hash: String,
    pub outputs: Vec<OutputS>, pub resolved: Vec<Resolved>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OutputS { pub key: String, pub label: String, pub value: f64, pub unit: String }
impl From<&Output> for OutputS { fn from(o: &Output) -> Self { OutputS { key: o.key.clone(), label: o.label.clone(), value: o.value, unit: o.unit.clone() } } }

#[derive(Serialize, Debug, Default)]
pub struct Diff { pub node: String, pub output: String, pub old: f64, pub new: f64 }
#[derive(Serialize, Debug, Default)]
pub struct Impact {
    pub has_previous: bool, pub edited: Vec<String>, pub affected: Vec<String>, pub newly_failing: Vec<String>, pub newly_passing: Vec<String>,
    pub added: Vec<String>, pub removed: Vec<String>, pub diffs: Vec<Diff>, pub review_required: bool, pub summary: String,
}
#[derive(Serialize, Debug, Default)]
pub struct ProjectResult { pub ok: bool, pub errors: Vec<String>, pub order: Vec<String>, pub nodes: Vec<NodeResult>, pub overall_pass: bool, pub impact: Impact, pub engine_version: String }

fn hash_str(s: &str) -> String { Sha256::digest(s.as_bytes()).iter().map(|b| format!("{:02x}", b)).collect() }

/// Order nodes so every node comes after the nodes it links from. Returns Err with the nodes in a cycle.
pub fn order(p: &Project) -> Result<Vec<usize>, Vec<String>> {
    let idx: HashMap<&str, usize> = p.nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let n = p.nodes.len();
    let mut indeg = vec![0usize; n]; let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for (i, node) in p.nodes.iter().enumerate() {
        let mut seen = HashSet::new();
        for l in &node.links { if let Some(&j) = idx.get(l.from.as_str()) { if seen.insert(j) { adj[j].push(i); indeg[i] += 1; } } }
    }
    let mut q: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect(); let mut out = vec![];
    while let Some(i) = q.pop_front() { out.push(i); for &j in &adj[i] { indeg[j] -= 1; if indeg[j] == 0 { q.push_back(j); } } }
    if out.len() == n { Ok(out) } else { Err((0..n).filter(|i| !out.contains(i)).map(|i| p.nodes[i].id.clone()).collect()) }
}

pub fn analyse(p: &Project, prev: Option<&Vec<NodeResult>>) -> ProjectResult {
    let mut res = ProjectResult { engine_version: crate::ENGINE_VERSION.into(), ..Default::default() };
    let mut ids = HashSet::new();
    for n in &p.nodes { if !ids.insert(n.id.clone()) { res.errors.push(format!("Duplicate node id '{}'", n.id)); } }
    for n in &p.nodes { for l in &n.links { if !p.nodes.iter().any(|x| x.id == l.from) { res.errors.push(format!("Node '{}' links field '{}' to unknown node '{}'", n.id, l.field, l.from)); } if l.from == n.id { res.errors.push(format!("Node '{}' links to itself", n.id)); } } }
    if !res.errors.is_empty() { return res; }
    let ord = match order(p) { Ok(o) => o, Err(c) => { res.errors.push(format!("Circular dependency between: {}", c.join(", "))); return res; } };
    res.order = ord.iter().map(|&i| p.nodes[i].id.clone()).collect();
    let mut done: HashMap<String, NodeResult> = HashMap::new();
    for &i in &ord {
        let n = &p.nodes[i];
        let mut vals = n.values.clone();
        let own = { let mut m = Map::new(); m.insert("module".into(), n.module.clone().into()); m.insert("mode".into(), n.mode.clone().into()); m.insert("param_set".into(), n.param_set.clone().into()); m.insert("values".into(), Value::Object(n.values.clone())); hash_str(&Value::Object(m).to_string()) };
        let mut resolved = vec![]; let mut blocked: Option<String> = None;
        for l in &n.links {
            match done.get(&l.from) {
                Some(up) if up.status == "calculated" => match up.outputs.iter().find(|o| o.key == l.output) {
                    Some(o) => { let v = o.value * l.factor; vals.insert(l.field.clone(), json!(v)); resolved.push(Resolved { field: l.field.clone(), from: l.from.clone(), output: l.output.clone(), value: v }); }
                    None => { blocked = Some(format!("Upstream '{}' has no output '{}'", l.from, l.output)); }
                },
                _ => { blocked = Some(format!("Upstream '{}' has no valid result", l.from)); }
            }
        }
        let name = if n.name.is_empty() { n.id.clone() } else { n.name.clone() };
        if let Some(msg) = blocked {
            done.insert(n.id.clone(), NodeResult { id: n.id.clone(), name, module: n.module.clone(), status: "blocked".into(), errors: vec![msg], overall_pass: false, has_verdict: true, max_util: 0.0, trace_hash: String::new(), own_hash: own, outputs: vec![], resolved });
            continue;
        }
        vals.insert("mode".into(), if n.mode.is_empty() { "design".into() } else { n.mode.clone().into() });
        vals.insert("param_set".into(), n.param_set.clone().into());
        let r = modules::run(&n.module, &Value::Object(vals));
        let nr = if r.ok { NodeResult { id: n.id.clone(), name, module: n.module.clone(), status: "calculated".into(), errors: vec![], overall_pass: r.overall_pass, has_verdict: !r.checks.is_empty(), max_util: r.max_util, trace_hash: r.trace_hash.clone(), own_hash: own, outputs: r.outputs.iter().map(OutputS::from).collect(), resolved } }
            else { NodeResult { id: n.id.clone(), name, module: n.module.clone(), status: "error".into(), errors: r.errors.clone(), overall_pass: false, has_verdict: true, max_util: 0.0, trace_hash: String::new(), own_hash: own, outputs: vec![], resolved } };
        done.insert(n.id.clone(), nr);
    }
    res.nodes = p.nodes.iter().map(|n| done.remove(&n.id).unwrap()).collect();
    res.overall_pass = res.nodes.iter().all(|n| n.status == "calculated" && (!n.has_verdict || n.overall_pass));
    res.ok = true;
    // change impact
    if let Some(prev) = prev {
        let im = &mut res.impact; im.has_previous = true;
        let pm: HashMap<&str, &NodeResult> = prev.iter().map(|n| (n.id.as_str(), n)).collect();
        for n in &res.nodes {
            match pm.get(n.id.as_str()) {
                None => im.added.push(n.id.clone()),
                Some(o) => {
                    if o.own_hash != n.own_hash { im.edited.push(n.id.clone()); }
                    else if o.trace_hash != n.trace_hash || o.status != n.status { im.affected.push(n.id.clone()); }
                    if o.has_verdict && n.has_verdict && o.overall_pass && !n.overall_pass { im.newly_failing.push(n.id.clone()); }
                    if o.has_verdict && n.has_verdict && !o.overall_pass && n.overall_pass { im.newly_passing.push(n.id.clone()); }
                    for no in &n.outputs { if let Some(oo) = o.outputs.iter().find(|x| x.key == no.key) { if (oo.value - no.value).abs() > 1e-9 * oo.value.abs().max(1.0) { im.diffs.push(Diff { node: n.id.clone(), output: no.key.clone(), old: oo.value, new: no.value }); } } }
                }
            }
        }
        for o in prev { if !res.nodes.iter().any(|n| n.id == o.id) { im.removed.push(o.id.clone()); } }
        im.affected.retain(|a| !im.edited.contains(a));
        im.review_required = !im.newly_failing.is_empty() || !im.edited.is_empty() || !im.affected.is_empty();
        im.summary = format!("{} edited, {} affected downstream, {} newly failing, {} newly passing, {} added, {} removed.", im.edited.len(), im.affected.len(), im.newly_failing.len(), im.newly_passing.len(), im.added.len(), im.removed.len());
    }
    res
}

pub fn analyse_json(project: &str, prev: &str) -> String {
    let p: Project = match serde_json::from_str(project) { Ok(p) => p, Err(e) => return serde_json::to_string(&ProjectResult { errors: vec![format!("Invalid project: {}", e)], ..Default::default() }).unwrap() };
    let prev: Option<Vec<NodeResult>> = serde_json::from_str(prev).ok();
    serde_json::to_string(&analyse(&p, prev.as_ref())).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wall() -> Value { json!({"h":200,"len":3000,"beta":1.0,"cover":30,"bar_dia":12,"hbar_dia":10,"fck":30,"fyk":500,"phi_ef":1.0,"n_ed":600,"m02":12,"m01":6}) }
    fn strip() -> Value { json!({"n_ed":1,"n_k":220,"q_allow":150,"t":225,"h":400,"cover":50,"bar_dia":12,"fck":30,"fyk":500}) }
    fn proj(wall_n: f64) -> Project {
        let mut w = wall(); w["n_ed"] = json!(wall_n);
        serde_json::from_value(json!({"nodes":[
            {"id":"strip","name":"Strip footing","module":"strip","param_set":"EN1992-1-1:2004/RECOMMENDED","values":strip(),"links":[{"field":"n_ed","from":"wall","output":"n_ed"}]},
            {"id":"wall","name":"Ground floor wall","module":"wall","param_set":"EN1992-1-1:2004/RECOMMENDED","values":w}]})).unwrap()
    }
    #[test] fn order_puts_upstream_first() { let o = order(&proj(600.0)).unwrap(); assert_eq!(proj(600.0).nodes[o[0]].id, "wall"); }
    #[test] fn load_is_passed_downstream() { let r = analyse(&proj(600.0), None); assert!(r.ok, "{:?}", r.errors); let s = r.nodes.iter().find(|n| n.id == "strip").unwrap(); assert_eq!(s.status, "calculated"); assert!((s.resolved[0].value - 600.0).abs() < 1e-9); }
    #[test] fn factor_scales_linked_value() { let mut p = proj(600.0); p.nodes[0].links[0].factor = 0.5; let r = analyse(&p, None); assert!((r.nodes[0].resolved[0].value - 300.0).abs() < 1e-9); }
    #[test] fn cycle_is_rejected() { let mut p = proj(600.0); p.nodes[1].links.push(Link { field: "n_ed".into(), from: "strip".into(), output: "n_ed".into(), factor: 1.0 }); let r = analyse(&p, None); assert!(!r.ok && r.errors[0].contains("Circular")); }
    #[test] fn unknown_node_rejected() { let mut p = proj(600.0); p.nodes[0].links[0].from = "nope".into(); assert!(!analyse(&p, None).ok); }
    #[test] fn editing_upstream_marks_downstream_affected() {
        let a = analyse(&proj(600.0), None); let b = analyse(&proj(900.0), Some(&a.nodes));
        assert!(b.impact.edited.contains(&"wall".to_string())); assert!(b.impact.affected.contains(&"strip".to_string())); assert!(b.impact.review_required);
        assert!(b.impact.diffs.iter().any(|d| d.node == "strip" || d.node == "wall"));
    }
    #[test] fn no_change_means_no_impact() { let a = analyse(&proj(600.0), None); let b = analyse(&proj(600.0), Some(&a.nodes)); assert!(b.impact.edited.is_empty() && b.impact.affected.is_empty() && !b.impact.review_required); }
    #[test] fn newly_failing_is_reported() { let a = analyse(&proj(600.0), None); let b = analyse(&proj(5000.0), Some(&a.nodes)); assert!(!b.impact.newly_failing.is_empty() || b.nodes.iter().any(|n| n.status != "calculated"), "{:?}", b.impact); }
    #[test] fn blocked_when_upstream_errors() { let mut p = proj(600.0); p.nodes[1].values.insert("h".into(), json!(-5)); let r = analyse(&p, None); assert_eq!(r.nodes[0].status, "blocked"); }
}
