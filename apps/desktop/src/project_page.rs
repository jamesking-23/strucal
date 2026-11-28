//! Project dependency page: linked calculations, revisions and change impact (same engine as the web app).
use super::*;
use serde_json::{json, Value};
use strucal_engine::project::{self, NodeResult, Project, ProjectResult};

use crate::cloud::Client;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

/// Result of cloud work done on a worker thread, returned with the (possibly refreshed) client session.
enum CloudDone {
    Opened { id: String, name: String, nodes: Vec<Value>, ver: i64, revs: Result<Vec<Value>, String> },
    OpenFailed(String),
    /// Ok(Some(version)) saved; Ok(None) version conflict.
    Flushed(Result<Option<i64>, String>),
    RevSaved { saved: Result<(), String>, flushed: Result<Option<i64>, String>, revs: Result<Vec<Value>, String> },
}

pub struct Pj { nodes: Vec<Value>, revs: Vec<Value>, base: Option<usize>, add: usize, res: Option<ProjectResult>, msg: String, cloud: Option<(String, String)>, ver: i64, dirty: Option<std::time::Instant>, explain: String, ex_rx: Option<Receiver<Result<Value, String>>>, job: Option<Receiver<(CloudDone, Client)>>, confirm_remove: Option<usize> }

fn s_of(v: &Value, k: &str) -> String { v[k].as_str().unwrap_or("").to_string() }

/// Loads a cloud project's revisions in the shape the page stores locally.
fn fetch_revs(cl: &mut Client, id: &str) -> Result<Vec<Value>, String> {
    let v = cl.get(&format!("revisions?select=id,number,name,nodes,results,status,created_at&project_id=eq.{id}&order=number"))?;
    Ok(v.as_array().cloned().unwrap_or_default().iter().map(|r| json!({"name": format!("{} ({})", s_of(r, "name"), s_of(r, "status").replace('_', " ")), "savedAt": s_of(r, "created_at"), "project": r["nodes"], "results": r["results"]})).collect())
}

/// Saves the draft with an optimistic version check.
fn flush(cl: &mut Client, id: &str, nodes: &[Value], ver: i64) -> Result<Option<i64>, String> {
    let body = json!({"nodes": nodes, "version": ver + 1, "updated_by": cl.user_id});
    let v = cl.patch(&format!("project_drafts?project_id=eq.{id}&version=eq.{ver}"), &body)?;
    Ok(v.as_array().and_then(|a| a.first()).map(|row| row["version"].as_i64().unwrap_or(ver + 1)))
}

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
        let mut pj = Pj { nodes: vec![], revs: vec![], base: None, add: 0, res: None, msg: String::new(), cloud: None, ver: 1, dirty: None, explain: String::new(), ex_rx: None, job: None, confirm_remove: None };
        if let Ok(s) = std::fs::read_to_string(docs_dir().join("project.json")) {
            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                pj.nodes = v["nodes"].as_array().cloned().unwrap_or_default(); pj.revs = v["revisions"].as_array().cloned().unwrap_or_default();
                pj.base = v["baseline"].as_u64().map(|x| x as usize).filter(|&i| i < pj.revs.len());
            }
        }
        pj.recompute(); pj
    }
    /// Opens a cloud project in the background; the page shows a busy state until it arrives.
    pub fn open_cloud(&mut self, cl: &mut Client, id: String, name: String) {
        if self.job.is_some() { self.msg = "Please wait for the current cloud operation to finish.".into(); return; }
        self.msg = format!("Opening {name}…");
        self.spawn(cl, move |c| match c.get(&format!("project_drafts?select=nodes,version&project_id=eq.{id}")) {
            Ok(v) => {
                let d = v.as_array().and_then(|a| a.first()).cloned().unwrap_or(Value::Null);
                let revs = fetch_revs(c, &id);
                CloudDone::Opened { nodes: d["nodes"].as_array().cloned().unwrap_or_default(), ver: d["version"].as_i64().unwrap_or(1), revs, id, name }
            }
            Err(e) => CloudDone::OpenFailed(e),
        });
    }
    fn spawn(&mut self, cl: &Client, work: impl FnOnce(&mut Client) -> CloudDone + Send + 'static) {
        let (tx, rx) = channel(); self.job = Some(rx);
        let mut c = cl.clone();
        std::thread::spawn(move || { let d = work(&mut c); let _ = tx.send((d, c)); });
    }
    fn flushed(&mut self, r: Result<Option<i64>, String>) {
        match r {
            Ok(Some(v)) => { self.ver = v; self.msg = "Saved to cloud".into(); }
            Ok(None) => { self.msg = "Conflict: someone else changed this project. Reopen it from the Account page before editing.".into(); }
            Err(e) => self.msg = format!("Not saved: {e}"),
        }
    }
    fn set_revs(&mut self, r: Result<Vec<Value>, String>) {
        match r { Ok(v) => { self.revs = v; self.base = if self.revs.is_empty() { None } else { Some(self.revs.len() - 1) }; } Err(e) => self.msg = format!("Revisions could not be loaded: {e}") }
    }
    /// Applies a finished background job. Returns true when the page needs recomputing.
    fn poll_job(&mut self, ctx: &egui::Context, cl: &mut Client) -> bool {
        let Some(rx) = &self.job else { return false };
        let (done, c) = match rx.try_recv() {
            Ok(x) => x,
            Err(TryRecvError::Empty) => { ctx.request_repaint_after(std::time::Duration::from_millis(100)); return false; }
            Err(TryRecvError::Disconnected) => { self.job = None; self.msg = "The cloud request stopped unexpectedly. Try again.".into(); return false; }
        };
        self.job = None;
        // Keep a refreshed session; a sign-out in the meantime wins.
        if cl.signed_in() { *cl = c; }
        match done {
            CloudDone::Opened { id, name, nodes, ver, revs } => {
                self.nodes = nodes; self.ver = ver; self.cloud = Some((id, name)); self.dirty = None; self.revs.clear(); self.base = None;
                self.set_revs(revs);
                if !self.msg.starts_with("Revisions") { self.msg = "Cloud project opened. Changes save automatically.".into(); }
                true
            }
            CloudDone::OpenFailed(e) => { self.msg = format!("Could not open the cloud project: {e}"); false }
            CloudDone::Flushed(r) => { self.flushed(r); false }
            CloudDone::RevSaved { saved, flushed, revs } => {
                match saved { Ok(()) => { self.flushed(flushed); self.set_revs(revs); if !self.msg.starts_with("Revisions") && !self.msg.starts_with("Conflict") { self.msg = "Revision saved to cloud".into(); } } Err(e) => self.msg = format!("Revision not saved: {e}") }
                true
            }
        }
    }
    fn close_cloud(&mut self) { let fresh = Pj::new(); *self = fresh; }
    fn save(&mut self) {
        if self.cloud.is_some() { return; }
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

    pub fn ui(&mut self, ctx: &egui::Context, defs: &[ModDef], cl: &mut crate::cloud::Client) {
        let mut changed = false; let mut cloud_rev = false; let mut leave_cloud = false; let mut want_explain = false;
        if let Some(rx) = self.ex_rx.take() { match rx.try_recv() { Ok(r) => { self.explain = match r { Ok(v) => v["answer"].as_str().unwrap_or("").to_string(), Err(e) => format!("Assistant: {e}") }; } Err(std::sync::mpsc::TryRecvError::Empty) => { self.ex_rx = Some(rx); ctx.request_repaint_after(std::time::Duration::from_millis(250)); } Err(_) => {} } }
        if self.poll_job(ctx, cl) { self.recompute(); }
        let busy = self.job.is_some();
        if let (Some(t), false) = (self.dirty, busy) {
            if t.elapsed() >= std::time::Duration::from_millis(900) {
                if let Some((id, _)) = self.cloud.clone() { let (nodes, ver) = (self.nodes.clone(), self.ver); self.dirty = None; self.msg = "Saving…".into(); self.spawn(cl, move |c| CloudDone::Flushed(flush(c, &id, &nodes, ver))); }
            } else { ctx.request_repaint_after(std::time::Duration::from_millis(300)); }
        }
        egui::TopBottomPanel::top("pjtools").exact_height(44.0).frame(egui::Frame::new().fill(RIBBON).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::symmetric(8, 8))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("pjadd").selected_text(defs[self.add].name).show_ui(ui, |ui| { for (i, d) in defs.iter().enumerate() { ui.selectable_value(&mut self.add, i, d.name); } });
                if ui.add(egui::Button::new(RichText::new("+ Add calculation").color(Color32::WHITE)).fill(BLUE)).clicked() {
                    let d = &defs[self.add]; let mut n = 1; while self.nodes.iter().any(|x| x["id"] == format!("{}{}", d.id, n)) { n += 1; }
                    let mut vals = serde_json::Map::new(); for f in &d.fields { vals.insert(f.key.into(), f.default.into()); }
                    self.nodes.push(json!({"id": format!("{}{}", d.id, n), "name": format!("{} {}", d.name, n), "module": d.id, "mode": "design", "param_set": "EN1992-1-1:2004/RECOMMENDED", "values": vals, "links": []}));
                    changed = true;
                }
                if busy { ui.spinner(); }
                if ui.add_enabled(!busy && self.res.as_ref().is_some_and(|r| r.ok), egui::Button::new(if self.cloud.is_some() { "Save revision to cloud" } else { "Save revision" })).clicked() {
                    if self.cloud.is_some() { cloud_rev = true; } else if let Some(r) = &self.res { let id = self.revs.len() + 1; self.revs.push(json!({"name": format!("Rev {}", id), "savedAt": now_str(), "project": self.nodes, "results": r.nodes})); self.base = Some(self.revs.len() - 1); changed = true; }
                }
                if let Some((_, n)) = &self.cloud { ui.label(RichText::new(format!("Cloud: {n}")).strong().color(BLUE)); if ui.add_enabled(!busy, egui::Button::new("Switch to local project")).clicked() { leave_cloud = true; } }
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
                if !self.msg.is_empty() { let bad = ["Conflict", "Not saved", "Could not", "Revision not", "Export failed", "The cloud", "Revisions could"].iter().any(|p| self.msg.starts_with(p)); ui.label(RichText::new(&self.msg).size(11.5).color(if bad { RED } else { GREEN })); }
            });
        });

        let names: Vec<(String, String)> = self.nodes.iter().map(|n| (Self::s(n, "id"), Self::s(n, "name"))).collect();
        let name_of = |id: &str| names.iter().find(|(i, _)| i == id).map(|(_, n)| n.clone()).unwrap_or(id.to_string());
        egui::SidePanel::right("pjside").exact_width(330.0).frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::same(10))).show(ctx, |ui| {
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
                        ui.add_space(4.0);
                        if ui.add_enabled(self.ex_rx.is_none() && cl.signed_in(), egui::Button::new(if self.ex_rx.is_some() { "Thinking..." } else { "Explain these changes (AI)" })).clicked() { want_explain = true; }
                        if !cl.signed_in() { ui.label(RichText::new("Sign in on the Account tab to use the assistant.").size(11.0).color(MUTE)); }
                        if !self.explain.is_empty() { ui.label(&self.explain); ui.label(RichText::new("AI-generated summary of the change impact. It does not change any result.").size(10.5).color(MUTE)); }
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
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("Remove").clicked() { self.confirm_remove = Some(idx); } });
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
                                        if let Some(opts) = &f.options {
                                            let cur = opts.iter().find(|o| (o.0 - v).abs() < 1e-9).map(|o| o.1).unwrap_or("-");
                                            egui::ComboBox::from_id_salt(format!("po_{}_{}", id, f.key)).width(260.0).selected_text(cur).show_ui(ui, |ui| { for (val, lab) in opts { if ui.selectable_label((*val - v).abs() < 1e-9, *lab).clicked() { node["values"][f.key] = json!(val); changed = true; } } });
                                        } else if ui.add(egui::DragValue::new(&mut v).speed(1.0).max_decimals(3)).changed() { node["values"][f.key] = json!(v); changed = true; }
                                        if f.options.is_none() { ui.menu_button("Link", |ui| {
                                            let mut any = false;
                                            for (nid, nn, key, label, unit) in &avail {
                                                if *nid == id { continue; } any = true;
                                                if ui.button(format!("{} → {} ({})", nn, label, unit)).clicked() {
                                                    if let Some(a) = node["links"].as_array_mut() { a.retain(|k| k["field"] != f.key); a.push(json!({"field": f.key, "from": nid, "output": key, "factor": 1.0})); }
                                                    changed = true; ui.close_menu();
                                                }
                                            }
                                            if !any { ui.label("No other calculation has outputs yet"); }
                                        }); }
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
        if let Some(i) = self.confirm_remove {
            let name = self.nodes.get(i).map(|n| Self::s(n, "name")).unwrap_or_default();
            egui::Window::new("Remove calculation").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.label(format!("Remove {name}? Inputs linked to its results become manual inputs."));
                ui.horizontal(|ui| { if ui.add(egui::Button::new(RichText::new("Remove").color(Color32::WHITE)).fill(RED)).clicked() { remove = Some(i); self.confirm_remove = None; } if ui.button("Cancel").clicked() { self.confirm_remove = None; } });
            });
        }
        if let Some(i) = remove.filter(|&i| i < self.nodes.len()) { let id = Self::s(&self.nodes[i], "id"); self.nodes.remove(i); for n in self.nodes.iter_mut() { if let Some(a) = n["links"].as_array_mut() { a.retain(|l| l["from"] != id.as_str()); } } changed = true; }
        if want_explain {
            if let (Some(r), Some(b)) = (&self.res, self.base) {
                let names: serde_json::Map<String, Value> = self.nodes.iter().map(|n| (Self::s(n, "id"), json!(format!("{} ({})", Self::s(n, "name"), Self::s(n, "module"))))).collect();
                let ctxv = json!({"baseline": Self::s(&self.revs[b], "name"), "impact": {"edited": r.impact.edited, "affected": r.impact.affected, "newly_failing": r.impact.newly_failing, "newly_passing": r.impact.newly_passing, "added": r.impact.added, "removed": r.impact.removed, "diffs": r.impact.diffs.iter().map(|d| json!({"node": d.node, "output": d.output, "old": d.old, "new": d.new})).collect::<Vec<_>>()}, "calculations": names, "results": r.nodes.iter().map(|n| json!({"id": n.id, "name": n.name, "status": n.status, "pass": n.overall_pass, "max_util": n.max_util, "outputs": n.outputs})).collect::<Vec<_>>()});
                let (tx, rx) = std::sync::mpsc::channel(); self.ex_rx = Some(rx); self.explain.clear();
                let mut c = cl.clone();
                std::thread::spawn(move || { let _ = tx.send(c.function("assistant", &json!({"task": "compare", "question": "", "context": ctxv}))); });
            }
        }
        if leave_cloud { self.close_cloud(); return; }
        if cloud_rev {
            if let (Some((id, _)), Some(r)) = (self.cloud.clone(), &self.res) {
                let args = json!({"p_project": id, "p_name": format!("Rev {}", self.revs.len() + 1), "p_nodes": self.nodes, "p_results": r.nodes, "p_engine": r.engine_version});
                let (nodes, ver) = (self.nodes.clone(), self.ver); self.dirty = None; self.msg = "Saving revision…".into();
                self.spawn(cl, move |c| match c.rpc("save_revision", &args) {
                    Ok(_) => { let flushed = flush(c, &id, &nodes, ver); let revs = fetch_revs(c, &id); CloudDone::RevSaved { saved: Ok(()), flushed, revs } }
                    Err(e) => CloudDone::RevSaved { saved: Err(e), flushed: Ok(None), revs: Ok(vec![]) },
                });
            }
        }
        if changed { self.recompute(); if self.cloud.is_some() { self.dirty = Some(std::time::Instant::now()); } else { self.save(); } }
    }
}
