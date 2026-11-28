//! Engineering assistant window: explains the deterministic calculation, proposes inputs (never applied without confirmation).
use super::*;
use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver};

#[derive(Default)]
pub struct Asst {
    open: bool,
    q: String,
    step: usize,
    busy: bool,
    rx: Option<Receiver<(String, Result<Value, String>)>>,
    answer: String,
    err: String,
    prop: Option<(Vec<(String, f64)>, Vec<String>, String)>,
}

impl App {
    fn asst_context(&self) -> Value {
        let (d, st) = (&self.defs[self.mi], &self.st[self.mi]);
        let inputs: serde_json::Map<String, Value> = d
            .fields
            .iter()
            .zip(&st.vals)
            .filter(|(f, _)| !f.check_only || st.mode == "check")
            .map(|(f, v)| (f.label.to_string(), json!(format!("{} {}", v, f.unit))))
            .collect();
        let mut c = json!({"module": d.title, "module_id": d.id, "mode": st.mode, "inputs": inputs});
        if let Some(r) = &self.result {
            c["parameter_set_status"] = json!(r.param_status);
            c["summary"] = json!(r.summary);
            c["checks"] = json!(r.checks);
            c["observations"] = json!(r.observations);
            c["scope_notes"] = json!(r.scope_notes);
            c["errors"] = json!(r.errors);
            c["trace_hash"] = json!(r.trace_hash);
            c["trace"] = json!(r.trace.steps.iter().enumerate().map(|(i, s)| json!({"n": i + 1, "section": s.section, "title": s.title, "reference": s.reference, "calculation": s.calculation, "output": s.output, "comment": s.comment})).collect::<Vec<_>>());
        }
        c
    }
    fn asst_call(&mut self, task: &str, question: String, context: Value) {
        if !self.cl.signed_in() {
            self.asst.err = "Sign in on the Account tab to use the assistant.".into();
            return;
        }
        let (tx, rx) = channel();
        self.asst.rx = Some(rx);
        self.asst.busy = true;
        self.asst.err.clear();
        self.asst.answer.clear();
        self.asst.prop = None;
        let mut c = self.cl.clone();
        let t = task.to_string();
        std::thread::spawn(move || {
            let r = c.function("assistant", &json!({"task": t, "question": question, "context": context}));
            let _ = tx.send((t, r));
        });
    }
    pub(crate) fn assistant_ui(&mut self, ctx: &egui::Context) {
        if let Some(rx) = self.asst.rx.take() {
            match rx.try_recv() {
                Ok((task, r)) => {
                    self.asst.busy = false;
                    match r {
                        Err(e) => self.asst.err = e,
                        Ok(v) => {
                            let text = v["answer"].as_str().unwrap_or("").to_string();
                            if task == "propose" {
                                let t = text
                                    .trim()
                                    .trim_start_matches("```json")
                                    .trim_start_matches("```")
                                    .trim_end_matches("```")
                                    .trim()
                                    .to_string();
                                match serde_json::from_str::<Value>(&t) {
                                    Ok(j) => {
                                        let d = &self.defs[self.mi];
                                        let props: Vec<(String, f64)> = j["proposals"]
                                            .as_object()
                                            .map(|m| {
                                                m.iter()
                                                    .filter_map(|(k, v)| {
                                                        v.as_f64()
                                                            .filter(|x| x.is_finite())
                                                            .filter(|_| d.fields.iter().any(|f| f.key == k))
                                                            .map(|x| (k.clone(), x))
                                                    })
                                                    .collect()
                                            })
                                            .unwrap_or_default();
                                        let missing = j["missing"]
                                            .as_array()
                                            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                                            .unwrap_or_default();
                                        self.asst.prop = Some((props, missing, j["notes"].as_str().unwrap_or("").to_string()));
                                    }
                                    Err(_) => self.asst.err = "The assistant's reply could not be read as inputs. Try rephrasing.".into(),
                                }
                            } else {
                                self.asst.answer = text;
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.asst.rx = Some(rx);
                    ctx.request_repaint_after(std::time::Duration::from_millis(250));
                }
                Err(_) => {
                    self.asst.busy = false;
                }
            }
        }
        egui::Area::new("asstbtn".into()).anchor(egui::Align2::RIGHT_BOTTOM, [-14.0, -34.0]).show(ctx, |ui| {
            if !self.asst.open && ui.add(egui::Button::new(RichText::new("Assistant").strong().color(Color32::WHITE)).fill(TX)).clicked() {
                self.asst.open = true;
            }
        });
        if !self.asst.open {
            return;
        }
        let mut open = true;
        let mut call: Option<(&'static str, String)> = None;
        let mut apply = false;
        let mut discard = false;
        egui::Window::new("StruCal Assistant").open(&mut open).anchor(egui::Align2::RIGHT_BOTTOM, [-14.0, -34.0]).default_width(380.0).resizable(true).show(ctx, |ui| {
            ui.label(RichText::new("The assistant explains the calculation shown. It never calculates or changes results, and cannot approve a design. All numbers come from the deterministic engine.").size(11.5).color(MUTE));
            if !self.cl.signed_in() { ui.colored_label(AMBER, "Sign in on the Account tab to use the assistant."); return; }
            let ok = self.result.as_ref().is_some_and(|r| r.ok);
            ui.horizontal_wrapped(|ui| { for (t, l) in [("summarise", "Summarise result"), ("governing", "Why does this check govern?"), ("explain", "Explain the working"), ("missing", "What might be missing?")] { if ui.add_enabled(!self.asst.busy && ok, egui::Button::new(l)).clicked() { call = Some((t, String::new())); } } });
            if let Some(r) = &self.result { if r.ok { ui.horizontal(|ui| {
                ui.label("Explain one step:");
                let cur = r.trace.steps.get(self.asst.step).map(|s| format!("{}. {}", self.asst.step + 1, s.title)).unwrap_or_default();
                egui::ComboBox::from_id_salt("asststep").width(200.0).selected_text(cur).show_ui(ui, |ui| { for (i, s) in r.trace.steps.iter().enumerate() { ui.selectable_value(&mut self.asst.step, i, format!("{}. {}", i + 1, s.title)); } });
                if ui.add_enabled(!self.asst.busy, egui::Button::new("Explain")).clicked() { call = Some(("explain_step", String::new())); }
            }); } }
            ui.separator();
            ui.add(egui::TextEdit::multiline(&mut self.asst.q).desired_rows(3).desired_width(f32::INFINITY).hint_text("Ask a question, or describe the member in words (for example: 300 wide, 500 deep beam, C30/37 concrete) to get proposed inputs"));
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.asst.busy && !self.asst.q.trim().is_empty(), egui::Button::new("Ask")).clicked() { call = Some(("chat", self.asst.q.clone())); }
                if ui.add_enabled(!self.asst.busy && !self.asst.q.trim().is_empty(), egui::Button::new("Propose inputs from description")).clicked() { call = Some(("propose", self.asst.q.clone())); }
            });
            if self.asst.busy { ui.label(RichText::new("Thinking...").color(MUTE)); }
            if !self.asst.err.is_empty() { ui.colored_label(RED, &self.asst.err); }
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                if !self.asst.answer.is_empty() { ui.label(&self.asst.answer); ui.label(RichText::new("AI-generated explanation of the deterministic calculation. Check it against the calculation sheet and the standard.").size(10.5).color(MUTE)); }
                if let Some((props, missing, notes)) = &self.asst.prop {
                    ui.label(RichText::new("Proposed inputs (not applied yet)").strong());
                    if props.is_empty() { ui.label(RichText::new("No input values could be taken from the description.").color(MUTE)); }
                    let d = &self.defs[self.mi];
                    egui::Grid::new("asstprop").num_columns(3).show(ui, |ui| { for (k, v) in props { if let Some((i, f)) = d.fields.iter().enumerate().find(|(_, f)| f.key == k) { ui.label(f.label); ui.label(RichText::new(self.st[self.mi].vals[i].to_string()).color(MUTE)); ui.label(RichText::new(format!("→ {} {}", v, f.unit)).strong()); ui.end_row(); } } });
                    if !missing.is_empty() { ui.colored_label(AMBER, format!("Still needed from you: {}", missing.join("; "))); }
                    if !notes.is_empty() { ui.label(RichText::new(notes).size(11.0).color(MUTE)); }
                    if !props.is_empty() { ui.horizontal(|ui| { if ui.add(egui::Button::new(RichText::new("Apply to the form").color(Color32::WHITE)).fill(TX)).clicked() { apply = true; } if ui.button("Discard").clicked() { discard = true; } }); }
                    ui.label(RichText::new("Review each value. The calculation only runs on values you confirm.").size(10.5).color(MUTE));
                }
            });
        });
        if !open {
            self.asst.open = false;
        }
        if apply {
            if let Some((props, _, _)) = self.asst.prop.take() {
                let keys: Vec<&'static str> = self.defs[self.mi].fields.iter().map(|f| f.key).collect();
                for (k, v) in props {
                    if let Some(i) = keys.iter().position(|x| *x == k) {
                        self.st[self.mi].vals[i] = v;
                    }
                }
                self.run();
            }
        }
        if discard {
            self.asst.prop = None;
        }
        if let Some((t, q)) = call {
            let mut c = self.asst_context();
            let task = if t == "explain_step" {
                if let Some(s) = self.result.as_ref().and_then(|r| r.trace.steps.get(self.asst.step)) {
                    c["step"] = json!({"title": s.title, "reference": s.reference, "calculation": s.calculation, "output": s.output});
                }
                "explain"
            } else if t == "propose" {
                let d = &self.defs[self.mi];
                let st = &self.st[self.mi];
                c = json!({"module": d.title, "allowed_fields": d.fields.iter().filter(|f| !f.check_only || st.mode == "check").map(|f| json!({"key": f.key, "label": f.label, "unit": f.unit})).collect::<Vec<_>>(), "current_inputs": d.fields.iter().zip(&st.vals).map(|(f, v)| (f.key.to_string(), json!(v))).collect::<serde_json::Map<_, _>>()});
                "propose"
            } else {
                t
            };
            self.asst_call(task, q, c);
        }
    }
}
