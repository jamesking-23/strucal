#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod update;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, StrokeKind};
use std::sync::mpsc::{channel, Receiver};
use strucal_engine::{modules, module::{Drawing, ModDef, ModResult}, params, registry, report};

const BAR: Color32 = Color32::from_rgb(11, 18, 32);
const BG: Color32 = Color32::from_rgb(238, 241, 245);
const LINE: Color32 = Color32::from_rgb(221, 227, 234);
const TX: Color32 = Color32::from_rgb(15, 23, 42);
const MUTE: Color32 = Color32::from_rgb(100, 116, 139);
const BLUE: Color32 = Color32::from_rgb(37, 99, 235);
const GREEN: Color32 = Color32::from_rgb(15, 123, 90);
const GREEN_BG: Color32 = Color32::from_rgb(230, 246, 239);
const AMBER: Color32 = Color32::from_rgb(217, 138, 0);
const RED: Color32 = Color32::from_rgb(180, 35, 24);
const RED_BG: Color32 = Color32::from_rgb(253, 236, 235);

#[derive(PartialEq, Clone, Copy)] enum Page { Workspace, Standards, Settings }
enum Upd { Idle, Checking, UpToDate, Available(update::Available), Installing, Failed(String) }

struct ModState { mode: String, param: String, vals: Vec<f64> }
struct App {
    page: Page, defs: Vec<ModDef>, mi: usize, st: Vec<ModState>, result: Option<ModResult>, saved: String,
    upd: Upd, rx: Option<Receiver<Result<Option<update::Available>, String>>>, irx: Option<Receiver<Result<(), String>>>,
}

fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0, LINE)).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(0)).show(ui, add).inner
}
fn heading(ui: &mut egui::Ui, t: &str) { ui.add_space(6.0); ui.label(RichText::new(t.to_uppercase()).size(10.5).strong().color(Color32::from_rgb(71, 85, 105))); }

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut v = egui::Visuals::light();
        v.panel_fill = BG; v.window_fill = Color32::WHITE;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(207, 216, 227));
        v.selection.bg_fill = BLUE;
        cc.egui_ctx.set_visuals(v);
        let mut st = (*cc.egui_ctx.style()).clone();
        st.spacing.item_spacing = egui::vec2(8.0, 6.0); st.spacing.button_padding = egui::vec2(12.0, 6.0);
        cc.egui_ctx.set_style(st);
        update::cleanup_old();
        let defs = modules::defs();
        let st = defs.iter().map(|d| ModState { mode: "design".into(), param: "EN1992-1-1:2004/RECOMMENDED".into(), vals: d.fields.iter().map(|f| f.default).collect() }).collect();
        let mut a = App { page: Page::Workspace, defs, mi: 0, st, result: None, saved: String::new(), upd: Upd::Idle, rx: None, irx: None };
        a.run(); a
    }
    fn json(&self) -> serde_json::Value {
        let (d, s) = (&self.defs[self.mi], &self.st[self.mi]);
        let mut m = serde_json::Map::new();
        m.insert("mode".into(), s.mode.clone().into()); m.insert("param_set".into(), s.param.clone().into());
        for (f, v) in d.fields.iter().zip(&s.vals) { m.insert(f.key.into(), (*v).into()); }
        serde_json::Value::Object(m)
    }
    fn run(&mut self) { let id = self.defs[self.mi].id; self.result = Some(modules::run(id, &self.json())); }

    fn field(ui: &mut egui::Ui, label: &str, unit: &str, v: &mut f64, ch: &mut bool) {
        ui.label(RichText::new(label).size(12.0).color(Color32::from_rgb(51, 65, 85)));
        *ch |= ui.add_sized([ui.available_width(), 28.0], egui::DragValue::new(v).speed(1.0).max_decimals(3).suffix(format!(" {unit}"))).changed();
    }

    fn export(&mut self) {
        let Some(r) = &self.result else { return };
        if !r.ok { return; }
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(std::path::PathBuf::from).unwrap_or_default();
        let dir = home.join("Downloads");
        let _ = std::fs::create_dir_all(&dir);
        let def = &self.defs[self.mi];
        let html = report::generic(def, &self.json(), r);
        let stem = format!("StruCal-{}-{}", def.id, &r.trace_hash[..8]);
        let (h, w) = (dir.join(format!("{stem}.html")), dir.join(format!("{stem}.doc")));
        self.saved = match (std::fs::write(&h, &html), std::fs::write(&w, &html)) {
            (Ok(_), Ok(_)) => {
                #[cfg(windows)] { let _ = std::process::Command::new("cmd").args(["/C", "start", "", &h.to_string_lossy()]).spawn(); }
                format!("Saved to Downloads:\n{stem}.html (print to PDF)\n{stem}.doc (opens in Word)")
            }
            _ => "Could not write to the Downloads folder".to_string(),
        };
    }

    fn draw_section(ui: &mut egui::Ui, d: &Drawing) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(220.0, 300.0), egui::Sense::hover());
        let p = ui.painter_at(rect);
        let lab = |p: &egui::Painter, pos: egui::Pos2, t: String| p.text(pos, egui::Align2::CENTER_CENTER, t, FontId::monospace(11.0), BLUE);
        match d {
            Drawing::Beam { b, h, cover, link_dia, bar_dia, n_bars } => {
                let sc = (150.0 / b).min(250.0 / h) as f32;
                let (w, hh) = (*b as f32 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0, TX), StrokeKind::Inside);
                p.rect_stroke(outer.shrink(((cover + link_dia / 2.0) as f32) * sc), 0.0, Stroke::new(1.2, Color32::from_rgb(51, 65, 85)), StrokeKind::Inside);
                let ed = ((cover + link_dia + bar_dia / 2.0) as f32) * sc; let n = (*n_bars).max(2);
                let br = ((*bar_dia as f32 * sc) / 2.0).max(2.4);
                for k in 0..n { p.circle_filled(egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32 / (n - 1) as f32, o.y + hh - ed), br, TX); }
                for k in 0..2 { p.circle_filled(egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32, o.y + ed), (6.0 * sc).max(2.2), Color32::from_rgb(100, 116, 139)); }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("b = {b}"));
                lab(&p, egui::pos2(o.x - 12.0, rect.center().y - 8.0), format!("h = {h}"));
            }
            Drawing::Column { b, h, cover, link_dia, bar_dia, n_face } => {
                let sc = (170.0 / b).min(250.0 / h) as f32;
                let (w, hh) = (*b as f32 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0, TX), StrokeKind::Inside);
                p.rect_stroke(outer.shrink(((cover + link_dia / 2.0) as f32) * sc), 0.0, Stroke::new(1.2, Color32::from_rgb(51, 65, 85)), StrokeKind::Inside);
                let ed = ((cover + link_dia + bar_dia / 2.0) as f32) * sc; let n = (*n_face).max(2);
                let br = ((*bar_dia as f32 * sc) / 2.0).max(2.4);
                for y in [o.y + ed, o.y + hh - ed] { for k in 0..n { p.circle_filled(egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32 / (n - 1) as f32, y), br, TX); } }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{b} × {h}"));
            }
            Drawing::Slab { h, cover, bar_dia, spacing } => {
                let sc = 0.5f32; let (w, hh) = (400.0 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0, TX), StrokeKind::Inside);
                let mut x = *spacing as f32 / 2.0;
                while x < 400.0 { p.circle_filled(egui::pos2(o.x + x * sc, o.y + hh - ((cover + bar_dia / 2.0) as f32) * sc), ((*bar_dia as f32 * sc) / 2.0).max(2.2), TX); x += *spacing as f32; }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("T{} @ {}", bar_dia, spacing));
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 32.0), format!("h = {h} (1 m strip)"));
            }
            Drawing::Pad { bx, by, cx, cy, h } => {
                let sc = (170.0 / bx.max(*by)) as f32; let (w, hh) = (*bx as f32 * sc, *by as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0, TX), StrokeKind::Inside);
                for i in 1..7 { let y = o.y + hh * i as f32 / 7.0; p.line_segment([egui::pos2(o.x + 6.0, y), egui::pos2(o.x + w - 6.0, y)], Stroke::new(1.0, Color32::from_rgb(148, 163, 184))); }
                p.rect_filled(egui::Rect::from_center_size(outer.center(), egui::vec2(*cx as f32 * sc, *cy as f32 * sc)), 0.0, Color32::from_rgb(51, 65, 85));
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{bx} × {by}, h = {h}"));
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        if let Some(rx) = &self.rx { if let Ok(m) = rx.try_recv() { self.upd = match m { Ok(Some(a)) => Upd::Available(a), Ok(None) => Upd::UpToDate, Err(e) => Upd::Failed(e) }; self.rx = None; } ctx.request_repaint_after(std::time::Duration::from_millis(200)); }
        if let Some(rx) = &self.irx { if let Ok(m) = rx.try_recv() { if let Err(e) = m { self.upd = Upd::Failed(e); } self.irx = None; } ctx.request_repaint_after(std::time::Duration::from_millis(200)); }

        let mut mode_changed = false; let mut do_export = false;
        egui::TopBottomPanel::top("top").exact_height(52.0).frame(egui::Frame::new().fill(BAR).inner_margin(egui::Margin::symmetric(16, 0))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, BLUE); ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "S", FontId::proportional(13.0), Color32::WHITE);
                ui.label(RichText::new("StruCal").color(Color32::WHITE).strong().size(16.0));
                ui.add_space(14.0);
                for (p, n) in [(Page::Workspace, "Workspace"), (Page::Standards, "Standards"), (Page::Settings, "Settings")] {
                    let on = self.page == p;
                    if ui.selectable_label(on, RichText::new(n).color(if on { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) })).clicked() { self.page = p; }
                }
                if self.page == Page::Workspace {
                    ui.add_space(18.0);
                    egui::ComboBox::from_id_salt("mod").selected_text(self.defs[self.mi].name).show_ui(ui, |ui| {
                        for i in 0..self.defs.len() { if ui.selectable_value(&mut self.mi, i, self.defs[i].name).changed() { mode_changed = true; } }
                        for u in modules::upcoming() { ui.add_enabled(false, egui::Label::new(format!("{} - {}", u.name, u.note))); }
                    });
                    for (m, n) in [("design", "Design"), ("check", "Check Mode")] {
                        let on = self.st[self.mi].mode == m;
                        if ui.selectable_label(on, RichText::new(n).color(if on { Color32::WHITE } else { Color32::from_rgb(159, 176, 204) })).clicked() { self.st[self.mi].mode = m.into(); mode_changed = true; }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.page == Page::Workspace && ui.add(egui::Button::new(RichText::new("Export").color(BAR).strong()).fill(Color32::WHITE)).clicked() { do_export = true; }
                    ui.label(RichText::new(format!("v{}", update::CURRENT)).color(Color32::from_rgb(100, 116, 139)).size(12.0));
                });
            });
        });
        if mode_changed { self.run(); }
        if do_export { self.export(); }

        match self.page {
            Page::Workspace => {
                let mut ch = false;
                egui::SidePanel::left("in").exact_width(290.0).frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0, LINE)).inner_margin(egui::Margin::same(16))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        heading(ui, "Design basis");
                        let stt = &mut self.st[self.mi];
                        egui::ComboBox::from_id_salt("ps").width(256.0).selected_text(params::get(&stt.param).map(|p| p.label).unwrap_or_default()).show_ui(ui, |ui| {
                            for p in params::available() { if ui.selectable_value(&mut stt.param, p.id.clone(), p.label).changed() { ch = true; } }
                        });
                        let mut last = "";
                        for (f, v) in self.defs[self.mi].fields.iter().zip(stt.vals.iter_mut()) {
                            if f.check_only && stt.mode != "check" { continue; }
                            if f.group != last { heading(ui, f.group); last = f.group; }
                            Self::field(ui, f.label, f.unit, v, &mut ch);
                        }
                    });
                });
                if ch { self.run(); }

                egui::SidePanel::right("out").exact_width(330.0).frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0, LINE)).inner_margin(egui::Margin::same(16))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(r) = &self.result { if r.ok {
                            heading(ui, "Verification status");
                            let (t, c, bg) = if r.overall_pass { ("PASS", GREEN, GREEN_BG) } else { ("FAIL", RED, RED_BG) };
                            egui::Frame::new().fill(bg).stroke(Stroke::new(1.0, c.gamma_multiply(0.45))).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| { ui.label(RichText::new(t).font(FontId::monospace(22.0)).strong().color(c)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(format!("Max {:.0}%", r.max_util * 100.0)).strong().color(c))); });
                                ui.label(RichText::new(format!("{}/{} checks satisfied. Implemented checks only; not an engineer's approval.", r.checks.iter().filter(|k| k.pass).count(), r.checks.len())).size(12.0).color(c));
                            });
                            heading(ui, "Utilisation ratios");
                            for k in &r.checks {
                                let col = if !k.pass { RED } else if k.utilisation > 0.85 { AMBER } else { Color32::from_rgb(15, 157, 106) };
                                ui.horizontal(|ui| { ui.label(&k.name); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(if k.utilisation >= 9.99 { "n/a".to_string() } else { format!("{:.2}", k.utilisation) }).font(FontId::monospace(13.0)).strong().color(col))); });
                                ui.label(RichText::new(format!("{} vs {}", k.demand, k.capacity)).font(FontId::monospace(10.5)).color(MUTE));
                                ui.add(egui::ProgressBar::new(k.utilisation.min(1.0) as f32).fill(col).desired_height(5.0));
                                ui.add_space(4.0);
                            }
                            heading(ui, &format!("Observations ({} open)", r.observations.len()));
                            for o in &r.observations {
                                let c = match o.level.as_str() { "RED" => RED, "AMBER" => AMBER, _ => BLUE };
                                egui::Frame::new().stroke(Stroke::new(1.0, LINE)).corner_radius(CornerRadius::same(6)).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| { ui.label(RichText::new(&o.id).font(FontId::monospace(11.0)).strong()); ui.label(RichText::new(&o.level).font(FontId::monospace(10.0)).strong().color(c)); });
                                    ui.label(RichText::new(&o.text).size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                });
                            }
                            if !self.saved.is_empty() { ui.add_space(8.0); ui.label(RichText::new(&self.saved).size(11.5).color(GREEN)); }
                        } }
                    });
                });

                egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::symmetric(26, 22))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(RichText::new(format!("{} - {}", self.defs[self.mi].title, if self.st[self.mi].mode == "check" { "independent check" } else { "design" })).size(24.0).strong().color(TX));
                        ui.label(RichText::new(self.defs[self.mi].subtitle).color(MUTE));
                        ui.add_space(10.0);
                        if let Some(r) = &self.result {
                            for e in &r.errors { ui.colored_label(RED, e); }
                            if r.ok {
                                if r.param_status.contains("NOT") {
                                    egui::Frame::new().fill(Color32::from_rgb(255, 247, 224)).stroke(Stroke::new(1.0, Color32::from_rgb(241, 211, 138))).corner_radius(CornerRadius::same(4)).inner_margin(egui::Margin::same(11)).show(ui, |ui| { ui.set_width(ui.available_width()); ui.label(RichText::new(format!("Parameter set: {}. Confirm the governing National Annex before issue for construction.", r.param_status)).size(12.5).color(Color32::from_rgb(91, 65, 8))); });
                                    ui.add_space(10.0);
                                }
                                for w in &r.warnings { ui.colored_label(AMBER, w); }
                                card(ui, |ui| {
                                    ui.horizontal(|ui| { ui.add_space(16.0); ui.label(RichText::new("Section definition").strong()); ui.label(RichText::new(self.defs[self.mi].name).font(FontId::monospace(11.5)).color(MUTE)); });
                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        if let Some(d) = &r.drawing { Self::draw_section(ui, d); }
                                        ui.vertical(|ui| {
                                            ui.set_width(ui.available_width() - 12.0);
                                            for k in &r.summary {
                                                ui.horizontal(|ui| { ui.label(RichText::new(&k.label).color(Color32::from_rgb(71, 85, 105))); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(&k.value).font(FontId::monospace(13.0)).strong())); });
                                                ui.separator();
                                            }
                                        });
                                    });
                                });
                                ui.add_space(14.0);
                                card(ui, |ui| {
                                    ui.horizontal(|ui| { ui.add_space(16.0); ui.label(RichText::new("Calculation trace").strong()); ui.label(RichText::new(format!("{} steps · deterministic", r.trace.steps.len())).size(12.0).color(MUTE)); });
                                    ui.separator();
                                    egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                                        egui::Grid::new("calc").striped(true).num_columns(4).spacing([16.0, 12.0]).min_col_width(40.0).show(ui, |ui| {
                                            for h in ["#", "REFERENCE", "CALCULATION", "OUTPUT"] { ui.label(RichText::new(h).size(10.5).strong().color(MUTE)); }
                                            ui.end_row();
                                            let (mut cur, mut n) = (String::new(), 0);
                                            for s in &r.trace.steps {
                                                if s.section != cur { cur = s.section.clone(); ui.label(""); ui.label(RichText::new(&s.section).strong().color(Color32::from_rgb(30, 58, 138))); ui.label(""); ui.label(""); ui.end_row(); }
                                                n += 1;
                                                ui.label(RichText::new(format!("{:02}", n)).font(FontId::monospace(11.0)).color(MUTE));
                                                ui.add_sized([150.0, 0.0], egui::Label::new(RichText::new(if s.reference.is_empty() { "-" } else { &s.reference }).font(FontId::monospace(11.0)).color(BLUE)).wrap());
                                                ui.vertical(|ui| { ui.set_max_width(440.0); ui.label(RichText::new(&s.title).strong().size(12.5)); ui.add(egui::Label::new(RichText::new(&s.calculation).font(FontId::monospace(12.0))).wrap()); if !s.comment.is_empty() { ui.label(RichText::new(&s.comment).size(11.0).color(MUTE)); } });
                                                ui.add(egui::Label::new(RichText::new(&s.output).font(FontId::monospace(12.0)).strong()).wrap());
                                                ui.end_row();
                                            }
                                        });
                                    });
                                    ui.label(RichText::new(format!("  SHA-256 {} · engine v{}", r.trace_hash, r.engine_version)).font(FontId::monospace(10.0)).color(MUTE));
                                    ui.add_space(8.0);
                                });
                                ui.add_space(8.0);
                                for n in &r.scope_notes { ui.label(RichText::new(n).size(12.0).color(MUTE)); }
                            }
                        }
                    });
                });
            }
            Page::Standards => { egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::same(26))).show(ctx, |ui| {
                ui.label(RichText::new("Formula registry").size(24.0).strong());
                ui.label(RichText::new("Every implemented formula, its source clause and verification state.").color(MUTE)); ui.add_space(10.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                    egui::Grid::new("reg").striped(true).spacing([18.0, 10.0]).show(ui, |ui| {
                        for h in ["ID", "NAME", "STANDARD", "CLAUSE", "VERIFICATION"] { ui.label(RichText::new(h).strong().color(MUTE).size(10.5)); } ui.end_row();
                        for f in registry::formulas() { ui.label(RichText::new(f.id).font(FontId::monospace(11.5))); ui.label(f.name); ui.label(f.standard); ui.label(RichText::new(f.clause).color(BLUE)); ui.label(RichText::new(f.verification).color(AMBER)); ui.end_row(); }
                    });
                }); });
            }); }
            Page::Settings => { egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::same(26))).show(ctx, |ui| {
                ui.label(RichText::new("Settings").size(24.0).strong()); ui.add_space(10.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(18)).show(ui, |ui| {
                    ui.label(RichText::new("SOFTWARE UPDATES").size(11.0).strong().color(MUTE));
                    ui.label(format!("Installed version: {}", update::CURRENT));
                    let busy = matches!(self.upd, Upd::Checking | Upd::Installing);
                    if ui.add_enabled(!busy, egui::Button::new(RichText::new("Check updates").color(Color32::WHITE)).fill(BLUE)).clicked() {
                        let (tx, rx) = channel(); self.rx = Some(rx); self.upd = Upd::Checking;
                        std::thread::spawn(move || { let _ = tx.send(update::check()); });
                    }
                    let mut go: Option<String> = None;
                    match &self.upd {
                        Upd::Idle => {}, Upd::Checking => { ui.label("Checking for updates..."); }
                        Upd::UpToDate => { ui.colored_label(GREEN, "You are on the latest version."); }
                        Upd::Available(a) => { ui.colored_label(BLUE, format!("Version {} is available.", a.version)); if !a.notes.is_empty() { ui.label(RichText::new(&a.notes).size(12.0).color(MUTE)); }
                            if ui.button("Download and install (app will restart)").clicked() { go = Some(a.url.clone()); } }
                        Upd::Installing => { ui.label("Downloading the installer. StruCal will close and reopen when it finishes..."); }
                        Upd::Failed(e) => { ui.colored_label(RED, e); }
                    }
                    if let Some(u) = go { let (tx, rx) = channel(); self.irx = Some(rx); self.upd = Upd::Installing; std::thread::spawn(move || { let _ = tx.send(update::install(&u)); }); }
                    ui.add_space(12.0); ui.label(RichText::new("Updates install in place. You do not need to uninstall first.").color(MUTE).size(12.0));
                }); });
            }); }
        }
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 860.0]).with_min_inner_size([1100.0, 650.0]).with_title("StruCal"), ..Default::default() };
    eframe::run_native("StruCal", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
