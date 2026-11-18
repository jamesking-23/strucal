//! Project dependency page: linked calculations, revisions and change impact (same engine as the web app).
use super::*;
use serde_json::{json, Value};
use strucal_engine::project::{self, NodeResult, Project, ProjectResult};

pub struct Pj { nodes: Vec<Value>, revs: Vec<Value>, base: Option<usize>, add: usize, res: Option<ProjectResult>, msg: String }

fn docs_dir() -> std::path::PathBuf { std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(std::path::PathBuf::from).unwrap_or_default().join("Documents").join("StruCal") }
fn now_str() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let z = days + 719468; let era = z.div_euclid(146097); let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 }; let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", y, m, d, rem / 3600, (rem % 3600) / 60)
}

impl Pj {
    pub fn new() -> Self {
        let mut pj = Pj { nodes: vec![], revs: vec![], base: None, add: 0, res: None, msg: String::new() };
        if let Ok(s) = std::fs::read_to_string(docs_dir().join("project.json")) {
            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                pj.nodes = v["nodes"].as_array().cloned().unwrap_or_default(); pj.revs = v["revisions"].as_array().cloned().unwrap_or_default();
                pj.base = v["baseline"].as_u64().map(|x| x as usize).filter(|&i| i < pj.revs.len());
            }
        }
        pj.recompute(); pj
    }
    fn save(&mut self) {
        let dir = docs_dir(); let _ = std::fs::create_dir_all(&dir);
        let body = json!({"nodes": self.nodes, "revisions": self.revs, "baseline": self.base});
        if std::fs::write(dir.join("project.json"), serde_json::to_string_pretty(&body).unwrap_or_default()).is_err() { self.msg = "Could not save the project file".into(); }
    }
    fn recompute(&mut self) {
        if self.nodes.is_empty() { self.res = None; return; }
        let p: Result<Project, _> = serde_json::from_value(json!({"nodes": self.nodes}));
        let prev: Option<Vec<NodeResult>> = self.base.and_then(|b| serde_json::from_value(self.revs[b]["results"].clone()).ok());
        self.res = p.ok().map(|p| project::analyse(&p, prev.as_ref()));
    }
    fn s(v: &Value, k: &str) -> String { v[k].as_str().unwrap_or("").to_string() }

    pub fn ui(&mut self, ctx: &egui::Context, defs: &[ModDef]) {
        let mut changed = false;
        egui::TopBottomPanel::top("pjtools").exact_height(44.0).frame(egui::Frame::new().fill(RIBBON).stroke(Stroke::new(1.0, LINE)).inner_margin(egui::Margin::symmetric(8, 8))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("pjadd").selected_text(defs[self.add].name).show_ui(ui, |ui| { for (i, d) in defs.iter().enumerate() { ui.selectable_value(&mut self.add, i, d.name); } });
                if ui.add(egui::Button::new(RichText::new("+ Add calculation").color(Color32::WHITE)).fill(BLUE)).clicked() {
                    let d = &defs[self.add]; let mut n = 1; while self.nodes.iter().any(|x| x["id"] == format!("{}{}", d.id, n)) { n += 1; }
                    let mut vals = serde_json::Map::new(); for f in &d.fields { vals.insert(f.key.into(), f.default.into()); }
                    self.nodes.push(json!({"id": format!("{}{}", d.id, n), "name": format!("{} {}", d.name, n), "module": d.id, "mode": "design", "param_set": "EN1992-1-1:2004/RECOMMENDED", "values": vals, "links": []}));
                    changed = true;
                }
                if ui.add_enabled(self.res.as_ref().map_or(false, |r| r.ok), egui::Button::new("Save revision")).clicked() {
                    if let Some(r) = &self.res { let id = self.revs.len() + 1; self.revs.push(json!({"name": format!("Rev {}", id), "savedAt": now_str(), "project": self.nodes, "results": r.nodes})); self.base = Some(self.revs.len() - 1); changed = true; }
                }
                ui.label(RichText::new("Compare against:").color(MUTE));
                let bt = self.base.map(|b| format!("{} · {}", Self::s(&self.revs[b], "name"), Self::s(&self.revs[b], "savedAt"))).unwrap_or("(no baseline)".into());
                egui::ComboBox::from_id_salt("pjbase").width(220.0).selected_text(bt).show_ui(ui, |ui| {
                    if ui.selectable_label(self.base.is_none(), "(no baseline)").clicked() { self.base = None; changed = true; }
                    for i in 0..self.revs.len() { let l = format!("{} · {}", Self::s(&self.revs[i], "name"), Self::s(&self.revs[i], "savedAt")); if ui.selectable_label(self.base == Some(i), l).clicked() { self.base = Some(i); changed = true; } }
                });
                if ui.add_enabled(self.base.is_some(), egui::Button::new("Restore baseline inputs")).clicked() { if let Some(b) = self.base { self.nodes = self.revs[b]["project"].as_array().cloned().unwrap_or_default(); changed = true; } }
                if ui.add_enabled(!self.nodes.is_empty(), egui::Button::new("Export project")).clicked() {
                    let dir = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(std::path::PathBuf::from).unwrap_or_default().join("Downloads"); let _ = std::fs::create_dir_all(&dir);
                    let p = dir.join("strucal-project.json");
                    self.msg = match std::fs::write(&p, serde_json::to_string_pretty(&json!({"nodes": self.nodes, "revisions": self.revs})).unwrap_or_default()) { Ok(_) => "Project exported to Downloads: strucal-project.json".into(), Err(e) => format!("Export failed: {e}") };
                }
                if !self.msg.is_empty() { ui.label(RichText::new(&self.msg).size(11.5).color(GREEN)); }
            });
        });

        let names: Vec<(String, String)> = self.nodes.iter().map(|n| (Self::s(n, "id"), Self::s(n, "name"))).collect();
        let name_of = |id: &str| names.iter().find(|(i, _)| i == id).map(|(_, n)| n.clone()).unwrap_or(id.to_string());
        egui::SidePanel::right("pjside").exact_width(330.0).frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0, LINE)).inner_margin(egui::Margin::same(10))).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                heading(ui, "PROJECT STATUS");
                match &self.res {
                    Some(r) if r.ok => {
                        let (t, c) = if r.overall_pass { ("ALL CHECKS PASS", GREEN) } else { ("ATTENTION REQUIRED", RED) };
                        ui.label(RichText::new(t).strong().color(c));
                        ui.label(RichText::new(format!("Order: {}", r.order.iter().map(|i| name_of(i)).collect::<Vec<_>>().join(" → "))).size(11.5).color(MUTE));
                    }
                    Some(r) => { for e in &r.errors { ui.colored_label(RED, e); } }
                    None => { ui.label(RichText::new("Add calculations to begin.").color(MUTE)); }
                }
                heading(ui, "CHANGE IMPACT");
                match (&self.res, self.base) {
                    (Some(r), Some(b)) if r.ok => {
                        let im = &r.impact;
                        ui.label(format!("Compared with {}", Self::s(&self.revs[b], "name")));
                        for (l, n) in [("Edited", im.edited.len()), ("Affected downstream", im.affected.len()), ("Newly failing", im.newly_failing.len()), ("Newly passing", im.newly_passing.len())] { ui.horizontal(|ui| { ui.label(l); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(n.to_string()).strong())); }); }
                        if im.review_required { ui.colored_label(AMBER, "Results changed since the baseline. Review before the next issue."); }
                        if !im.newly_failing.is_empty() { ui.colored_label(RED, format!("Now failing: {}", im.newly_failing.iter().map(|i| name_of(i)).collect::<Vec<_>>().join(", "))); }
                        for d in &im.diffs { ui.label(RichText::new(format!("{} · {}: {:.1} → {:.1}", name_of(&d.node), d.output, d.old, d.new)).font(FontId::monospace(11.0))); }
                    }
                    _ => { ui.label(RichText::new("Save a revision, then edit inputs to see what changes. Saved revisions are never overwritten.").size(12.0).color(MUTE)); }
                }
                heading(ui, "REVISIONS");
                if self.revs.is_empty() { ui.label(RichText::new("None saved.").color(MUTE)); }
                for r in &self.revs { ui.label(format!("{} · {}", Self::s(r, "name"), Self::s(r, "savedAt"))); }
                ui.add_space(8.0);
                ui.label(RichText::new("Links pass design values only. Combine load effects with the Load combinations module first, and confirm each linked value is the right design action for the next member.").size(11.5).color(AMBER));
            });
        });

        // available outputs and resolved values from the last analysis
        let avail: Vec<(String, String, String, String, String)> = self.res.as_ref().map(|r| r.nodes.iter().flat_map(|n| n.outputs.iter().map(move |o| (n.id.clone(), n.name.clone(), o.key.clone(), o.label.clone(), o.unit.clone()))).collect()).unwrap_or_default();
        let resolved: Vec<(String, String, f64)> = self.res.as_ref().map(|r| r.nodes.iter().flat_map(|n| n.resolved.iter().map(move |x| (n.id.clone(), x.field.clone(), x.value))).collect()).unwrap_or_default();
        let nres: Vec<NodeResult> = self.res.as_ref().map(|r| r.nodes.clone()).unwrap_or_default();
        let imp = self.res.as_ref().map(|r| (r.impact.edited.clone(), r.impact.affected.clone()));
        let mut remove: Option<usize> = None;
        egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::same(8))).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.nodes.is_empty() { ui.add_space(30.0); ui.vertical_centered(|ui| ui.label(RichText::new("No calculations yet. Add calculations and link one result to the next input, for example a wall's ULS load to a strip footing's load.").color(MUTE))); }
                for idx in 0..self.nodes.len() {
                    let node = &mut self.nodes[idx];
                    let id = Self::s(node, "id"); let module = Self::s(node, "module");
                    let Some(def) = defs.iter().find(|d| d.id == module) else { continue };
                    let r = nres.iter().find(|n| n.id == id);
                    card(ui, |ui| {
                        egui::Frame::new().fill(HEAD).inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let mut nm = Self::s(node, "name"); if ui.add(egui::TextEdit::singleline(&mut nm).desired_width(200.0)).changed() { node["name"] = json!(nm); changed = true; }
                                ui.label(RichText::new(def.name).size(11.0).color(MUTE));
                                if let Some(r) = r { let (t, c) = if r.status == "blocked" { ("BLOCKED", AMBER) } else if r.status == "error" { ("INPUT ERROR", RED) } else if !r.has_verdict { ("RESULTS", BLUE) } else if r.overall_pass { ("PASS", GREEN) } else { ("FAIL", RED) }; ui.label(RichText::new(format!("{}{}", t, if r.status == "calculated" && r.has_verdict { format!(" {:.0}%", r.max_util * 100.0) } else { String::new() })).strong().color(c)); }
                                if let Some((e, a)) = &imp { if e.contains(&id) { ui.label(RichText::new("EDITED").strong().color(AMBER)); } else if a.contains(&id) { ui.label(RichText::new("AFFECTED").strong().color(AMBER)); } }
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("Remove").clicked() { remove = Some(idx); } });
                            });
                        });
                        egui::Frame::new().inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                            let mode = Self::s(node, "mode");
                            egui::Grid::new(format!("pjg{}", id)).num_columns(3).spacing([12.0, 6.0]).show(ui, |ui| {
                                for f in &def.fields {
                                    if f.check_only && mode != "check" { continue; }
                                    ui.label(RichText::new(format!("{} ({})", f.label, f.unit)).size(11.5));
                                    let link = node["links"].as_array().and_then(|a| a.iter().find(|l| l["field"] == f.key)).cloned();
                                    if let Some(l) = &link {
                                        let rv = resolved.iter().find(|(n, fld, _)| *n == id && fld == f.key).map(|x| x.2);
                                        ui.label(RichText::new(rv.map(|v| format!("{:.3}", v)).unwrap_or("-".into())).font(FontId::monospace(12.0)).color(BLUE));
                                        let mut fac = l["factor"].as_f64().unwrap_or(1.0);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("← {} · {}  ×", Self::s(l, "from"), Self::s(l, "output"))).size(11.0).color(MUTE));
                                            if ui.add(egui::DragValue::new(&mut fac).speed(0.05)).changed() { if let Some(a) = node["links"].as_array_mut() { for k in a.iter_mut() { if k["field"] == f.key { k["factor"] = json!(fac); } } } changed = true; }
                                            if ui.small_button("Unlink").clicked() { if let Some(a) = node["links"].as_array_mut() { a.retain(|k| k["field"] != f.key); } changed = true; }
                                        });
                                    } else {
                                        let mut v = node["values"][f.key].as_f64().unwrap_or(f.default);
                                        if ui.add(egui::DragValue::new(&mut v).speed(1.0).max_decimals(3)).changed() { node["values"][f.key] = json!(v); changed = true; }
                                        ui.menu_button("Link", |ui| {
                                            let mut any = false;
                                            for (nid, nn, key, label, unit) in &avail {
                                                if *nid == id { continue; } any = true;
                                                if ui.button(format!("{} → {} ({})", nn, label, unit)).clicked() {
                                                    if let Some(a) = node["links"].as_array_mut() { a.retain(|k| k["field"] != f.key); a.push(json!({"field": f.key, "from": nid, "output": key, "factor": 1.0})); }
                                                    changed = true; ui.close_menu();
                                                }
                                            }
                                            if !any { ui.label("No other calculation has outputs yet"); }
                                        });
                                    }
                                    ui.end_row();
                                }
                            });
                            if let Some(r) = r { for e in &r.errors { ui.colored_label(RED, e); } if !r.outputs.is_empty() { ui.separator(); ui.horizontal_wrapped(|ui| { for o in &r.outputs { ui.label(RichText::new(format!("{}: {:.2} {}", o.label, o.value, o.unit)).size(11.5).color(MUTE)); } }); } }
                        });
                    });
                    ui.add_space(6.0);
                }
            });
        });
        if let Some(i) = remove { let id = Self::s(&self.nodes[i], "id"); self.nodes.remove(i); for n in self.nodes.iter_mut() { if let Some(a) = n["links"].as_array_mut() { a.retain(|l| l["from"] != id.as_str()); } } changed = true; }
        if changed { self.recompute(); self.save(); }
    }
}
