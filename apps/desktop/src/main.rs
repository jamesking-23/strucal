#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod account_page;
mod assistant_page;
mod cloud;
mod project_page;
mod update;
mod viewport;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke};
use std::sync::mpsc::{channel, Receiver};
use strucal_engine::{
    module::{ModDef, ModResult},
    modules, params, registry, report,
};

// Palette shared with the web design system (apps/web/app/globals.css): ink, warm neutrals, one accent, semantic result colours.
const BG: Color32 = Color32::from_rgb(247, 247, 245);
const LINE: Color32 = Color32::from_rgb(225, 225, 220);
const HEAD: Color32 = Color32::from_rgb(240, 240, 237);
const RIBBON: Color32 = Color32::WHITE;
const SEL: Color32 = Color32::from_rgb(232, 237, 252);
const TX: Color32 = Color32::from_rgb(17, 17, 17);
const MUTE: Color32 = Color32::from_rgb(95, 95, 92);
const BLUE: Color32 = Color32::from_rgb(31, 69, 214);
const GREEN: Color32 = Color32::from_rgb(11, 115, 65);
const AMBER: Color32 = Color32::from_rgb(143, 88, 0);
const AMBER_BG: Color32 = Color32::from_rgb(253, 243, 223);
const RED: Color32 = Color32::from_rgb(184, 35, 27);
const RED_BG: Color32 = Color32::from_rgb(251, 236, 235);

#[derive(PartialEq, Clone, Copy)]
enum Page {
    Workspace,
    Project,
    Account,
    Standards,
    Settings,
}
enum Upd {
    Idle,
    Checking,
    UpToDate,
    Available(update::Available),
    Installing,
    Failed(String),
}

struct ModState {
    mode: String,
    param: String,
    vals: Vec<f64>,
}
struct App {
    page: Page,
    defs: Vec<ModDef>,
    mi: usize,
    st: Vec<ModState>,
    result: Option<ModResult>,
    saved: String,
    upd: Upd,
    rx: Option<Receiver<Result<Option<update::Available>, String>>>,
    irx: Option<Receiver<Result<(), String>>>,
    pj: project_page::Pj,
    cl: cloud::Client,
    acct: account_page::Acct,
    acct_job: account_page::AcctJob,
    asst: assistant_page::Asst,
    /// Section viewport shows dimension lines.
    dims: bool,
}

fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, LINE))
        .corner_radius(CornerRadius::same(2))
        .inner_margin(egui::Margin::same(0))
        .show(ui, add)
        .inner
}
/// Full-width hairline.
fn rule(ui: &mut egui::Ui) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(r, 0.0, LINE);
}
/// Toolbar button; `on` renders the selected (ink) state of a segmented control.
fn tbtn(ui: &mut egui::Ui, t: &str, on: bool) -> egui::Response {
    let text = RichText::new(t).size(12.5).color(if on { Color32::WHITE } else { TX });
    ui.add(
        egui::Button::new(text)
            .fill(if on { TX } else { Color32::WHITE })
            .stroke(Stroke::new(1.0_f32, if on { TX } else { Color32::from_rgb(207, 207, 201) }))
            .corner_radius(CornerRadius::same(2))
            .min_size(egui::vec2(0.0, 30.0)),
    )
}
/// Primary (ink) action button.
fn primary(ui: &mut egui::Ui, t: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(t).size(12.5).strong().color(Color32::WHITE))
            .fill(TX)
            .corner_radius(CornerRadius::same(2))
            .min_size(egui::vec2(0.0, 30.0)),
    )
}
/// Pane header strip: uppercase title and a muted monospace meta value.
fn pane_header(ui: &mut egui::Ui, title: &str, meta: &str) {
    egui::Frame::new().fill(HEAD).inner_margin(egui::Margin::symmetric(16, 9)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(title.to_uppercase()).size(10.5).strong().color(MUTE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(meta).font(FontId::monospace(10.5)).color(MUTE))
            });
        });
    });
    rule(ui);
}
/// Lettered calculation-sheet heading with a rule below.
fn sheet_heading(ui: &mut egui::Ui, letter: &str, title: &str) {
    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(letter).font(FontId::monospace(12.0)).color(MUTE));
        ui.label(RichText::new(title).strong().size(14.0).color(TX));
    });
    ui.add_space(2.0);
    rule(ui);
    ui.add_space(8.0);
}
/// Status colour of a check: fail, review (above 0.85) or ok.
fn status_color(pass: bool, util: f64) -> Color32 {
    if !pass {
        RED
    } else if util > 0.85 {
        AMBER
    } else {
        GREEN
    }
}
/// Utilisation as shown to engineers; the engine saturates unbounded ratios at 9.99.
fn util_text(u: f64) -> String {
    if u >= 9.99 {
        "n/a".into()
    } else {
        format!("{u:.2}")
    }
}
/// Utilisation bar with the 0.85 review tick.
fn util_bar(ui: &mut egui::Ui, u: f64, col: Color32, h: f32) {
    let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), egui::Sense::hover());
    ui.painter().rect_filled(bar, 0.0, HEAD);
    ui.painter().rect_filled(egui::Rect::from_min_size(bar.min, egui::vec2(bar.width() * u.clamp(0.0, 1.0) as f32, h)), 0.0, col);
    let tx = bar.left() + bar.width() * 0.85;
    ui.painter().line_segment([egui::pos2(tx, bar.top() - 2.0), egui::pos2(tx, bar.bottom() + 2.0)], Stroke::new(1.0_f32, MUTE));
}
/// A folder under the user's home (Windows: %USERPROFILE%), created if missing.
pub(crate) fn user_dir(parts: &[&str]) -> std::path::PathBuf {
    let mut p = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(std::path::PathBuf::from).unwrap_or_default();
    for x in parts {
        p.push(x);
    }
    let _ = std::fs::create_dir_all(&p);
    p
}
/// Section title inside a pane: small uppercase muted label with a hairline below.
fn heading(ui: &mut egui::Ui, t: &str) {
    ui.add_space(14.0);
    ui.label(RichText::new(t.to_uppercase()).size(10.5).strong().color(MUTE));
    ui.add_space(4.0);
    rule(ui);
    ui.add_space(6.0);
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut v = egui::Visuals::light();
        v.panel_fill = BG;
        v.window_fill = Color32::WHITE;
        v.extreme_bg_color = Color32::WHITE;
        v.faint_bg_color = Color32::from_rgb(250, 250, 248);
        v.window_stroke = Stroke::new(1.0_f32, LINE);
        v.selection.bg_fill = BLUE;
        v.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
        v.hyperlink_color = BLUE;
        v.window_corner_radius = CornerRadius::same(2);
        v.menu_corner_radius = CornerRadius::same(2);
        for w in
            [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open]
        {
            w.corner_radius = CornerRadius::same(2);
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, LINE);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TX);
        v.widgets.inactive.bg_fill = Color32::WHITE;
        v.widgets.inactive.weak_bg_fill = Color32::WHITE;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(207, 207, 201));
        v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TX);
        v.widgets.hovered.weak_bg_fill = HEAD;
        v.widgets.hovered.bg_fill = HEAD;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, TX);
        v.widgets.active.bg_stroke = Stroke::new(1.0_f32, BLUE);
        cc.egui_ctx.set_visuals(v);
        let mut st = (*cc.egui_ctx.style()).clone();
        st.spacing.item_spacing = egui::vec2(8.0, 6.0);
        st.spacing.button_padding = egui::vec2(10.0, 4.0);
        st.spacing.interact_size.y = 26.0;
        for (t, sz) in [
            (egui::TextStyle::Body, 13.0),
            (egui::TextStyle::Button, 12.5),
            (egui::TextStyle::Small, 11.0),
            (egui::TextStyle::Monospace, 12.0),
        ] {
            if let Some(f) = st.text_styles.get_mut(&t) {
                f.size = sz;
            }
        }
        cc.egui_ctx.set_style(st);
        update::cleanup_old();
        let defs = modules::defs();
        let st = defs
            .iter()
            .map(|d| ModState {
                mode: "design".into(),
                param: "EN1992-1-1:2004/RECOMMENDED".into(),
                vals: d.fields.iter().map(|f| f.default).collect(),
            })
            .collect();
        let mut a = App {
            page: Page::Workspace,
            defs,
            mi: 0,
            st,
            result: None,
            saved: String::new(),
            upd: Upd::Idle,
            rx: None,
            irx: None,
            pj: project_page::Pj::new(),
            cl: cloud::Client::default(),
            acct: account_page::Acct::new(),
            acct_job: account_page::AcctJob::default(),
            asst: assistant_page::Asst::default(),
            dims: true,
        };
        a.run();
        a
    }
    fn json(&self) -> serde_json::Value {
        let (d, s) = (&self.defs[self.mi], &self.st[self.mi]);
        let mut m = serde_json::Map::new();
        m.insert("mode".into(), s.mode.clone().into());
        m.insert("param_set".into(), s.param.clone().into());
        for (f, v) in d.fields.iter().zip(&s.vals) {
            m.insert(f.key.into(), (*v).into());
        }
        serde_json::Value::Object(m)
    }
    fn run(&mut self) {
        let id = self.defs[self.mi].id;
        self.result = Some(modules::run(id, &self.json()));
    }

    fn export(&mut self) {
        let Some(r) = &self.result else { return };
        if !r.ok {
            return;
        }
        let dir = user_dir(&["Downloads"]);
        let def = &self.defs[self.mi];
        let html = report::generic(def, &self.json(), r);
        let stem = format!("StruCal-{}-{}", def.id, &r.trace_hash[..8]);
        let (h, w) = (dir.join(format!("{stem}.html")), dir.join(format!("{stem}.doc")));
        self.saved = match (std::fs::write(&h, &html), std::fs::write(&w, &html)) {
            (Ok(_), Ok(_)) => {
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("cmd").args(["/C", "start", "", &h.to_string_lossy()]).spawn();
                }
                format!("Saved to Downloads:\n{stem}.html (print to PDF)\n{stem}.doc (opens in Word)")
            }
            _ => "Could not write to the Downloads folder".to_string(),
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        if let Some(rx) = &self.rx {
            if let Ok(m) = rx.try_recv() {
                self.upd = match m {
                    Ok(Some(a)) => Upd::Available(a),
                    Ok(None) => Upd::UpToDate,
                    Err(e) => Upd::Failed(e),
                };
                self.rx = None;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        if let Some(rx) = &self.irx {
            if let Ok(m) = rx.try_recv() {
                if let Err(e) = m {
                    self.upd = Upd::Failed(e);
                }
                self.irx = None;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        let mut mode_changed = false;
        let mut do_export = false;
        egui::TopBottomPanel::top("tabs")
            .exact_height(44.0)
            .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::symmetric(14, 0)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.horizontal_centered(|ui| {
                    let (m, _) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
                    ui.painter().rect_filled(m, 0.0, TX);
                    ui.painter().text(m.center(), egui::Align2::CENTER_CENTER, "S", FontId::proportional(16.0), Color32::WHITE);
                    ui.add_space(14.0);
                    for (p, n) in [
                        (Page::Workspace, "Workspace"),
                        (Page::Project, "Project"),
                        (Page::Standards, "Standards"),
                        (Page::Account, "Account"),
                        (Page::Settings, "Settings"),
                    ] {
                        let on = self.page == p;
                        let w = 22.0 + n.len() as f32 * 7.2;
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(w, 44.0), egui::Sense::click());
                        let col = if on || resp.hovered() { TX } else { MUTE };
                        ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, n, FontId::proportional(13.0), col);
                        if on {
                            ui.painter().rect_filled(
                                egui::Rect::from_min_max(
                                    egui::pos2(r.left() + 6.0, r.bottom() - 2.0),
                                    egui::pos2(r.right() - 6.0, r.bottom()),
                                ),
                                0.0,
                                TX,
                            );
                        }
                        if resp.clicked() {
                            self.page = p;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("v{}", update::CURRENT)).font(FontId::monospace(11.0)).color(MUTE));
                    });
                });
            });
        if self.page == Page::Workspace {
            egui::TopBottomPanel::top("ribbon")
                .exact_height(48.0)
                .frame(egui::Frame::new().fill(RIBBON).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::symmetric(14, 0)))
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(RichText::new("MODULE").size(10.5).strong().color(MUTE));
                        egui::ComboBox::from_id_salt("mod")
                            .width(220.0)
                            .selected_text(RichText::new(self.defs[self.mi].name).strong())
                            .show_ui(ui, |ui| {
                                for i in 0..self.defs.len() {
                                    if ui.selectable_value(&mut self.mi, i, self.defs[i].name).changed() {
                                        mode_changed = true;
                                    }
                                }
                                for u in modules::upcoming() {
                                    ui.add_enabled(false, egui::Label::new(format!("{} - {}", u.name, u.note)));
                                }
                            });
                        ui.add_space(6.0);
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (m, n) in [("design", "Design"), ("check", "Check")] {
                            let enabled = m == "design" || self.defs[self.mi].has_check_mode;
                            let on = self.st[self.mi].mode == m;
                            if ui
                                .add_enabled_ui(enabled, |ui| tbtn(ui, n, on))
                                .inner
                                .on_disabled_hover_text("This module has no Check Mode")
                                .clicked()
                            {
                                self.st[self.mi].mode = m.into();
                                mode_changed = true;
                            }
                        }
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.add_space(10.0);
                        ui.label(RichText::new("PARAMETERS").size(10.5).strong().color(MUTE));
                        let stt = &mut self.st[self.mi];
                        egui::ComboBox::from_id_salt("ps")
                            .width(300.0)
                            .selected_text(params::get(&stt.param).map(|p| p.label).unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for p in params::available() {
                                    if ui.selectable_value(&mut stt.param, p.id.clone(), p.label).changed() {
                                        mode_changed = true;
                                    }
                                }
                            });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can_export = self.result.as_ref().is_some_and(|r| r.ok);
                            if ui
                                .add_enabled_ui(can_export, |ui| primary(ui, "Export"))
                                .inner
                                .on_hover_text("Save the calculation sheet (.html and .doc) to Downloads")
                                .clicked()
                            {
                                do_export = true;
                            }
                            if tbtn(ui, "Reset", false).on_hover_text("Reset this module's inputs to their defaults").clicked() {
                                let d = &self.defs[self.mi];
                                self.st[self.mi].vals = d.fields.iter().map(|f| f.default).collect();
                                self.saved.clear();
                                mode_changed = true;
                            }
                        });
                    });
                });
        }
        egui::TopBottomPanel::bottom("status")
            .exact_height(26.0)
            .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::symmetric(14, 0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let t = |s: String| RichText::new(s).font(FontId::monospace(11.0)).color(MUTE);
                    let (state, dot) = match &self.result {
                        Some(r) if r.ok && r.checks.is_empty() => ("Calculated", MUTE),
                        Some(r) if r.ok && r.overall_pass => ("All checks satisfied", GREEN),
                        Some(r) if r.ok => ("Checks failing", RED),
                        _ => ("Not calculated", MUTE),
                    };
                    let (d, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(d.center(), 3.5, dot);
                    ui.label(t(state.into()));
                    ui.add_space(14.0);
                    ui.label(t(self.st[self.mi].param.clone()));
                    if let Some(r) = self.result.as_ref().filter(|r| r.ok) {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(t("preview build".into()));
                            ui.add_space(14.0);
                            ui.label(t(format!("engine v{}", r.engine_version)));
                            ui.add_space(14.0);
                            ui.label(t(format!("trace {}", &r.trace_hash[..8])));
                        });
                    }
                });
            });
        if mode_changed {
            self.run();
        }
        if do_export {
            self.export();
        }

        match self.page {
            Page::Workspace => {
                self.assistant_ui(ctx);
                let mut ch = false;
                let mi = self.mi;
                // ---------- parameter inspector ----------
                egui::SidePanel::left("in")
                    .exact_width(340.0)
                    .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::ZERO))
                    .show(ctx, |ui| {
                        pane_header(
                            ui,
                            "Parameters",
                            &format!(
                                "{} · {}",
                                self.defs[mi].fields.iter().filter(|f| !f.check_only || self.st[mi].mode == "check").count(),
                                self.st[mi].mode
                            ),
                        );
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            let def = &self.defs[mi];
                            let stt = &mut self.st[mi];
                            let mut groups: Vec<&str> = vec![];
                            for f in &def.fields {
                                if (!f.check_only || stt.mode == "check") && !groups.contains(&f.group) {
                                    groups.push(f.group);
                                }
                            }
                            for g in groups {
                                let idx: Vec<usize> = def
                                    .fields
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, f)| f.group == g && (!f.check_only || stt.mode == "check"))
                                    .map(|(i, _)| i)
                                    .collect();
                                let n_mod = idx.iter().filter(|&&i| (stt.vals[i] - def.fields[i].default).abs() > 1e-12).count();
                                let title = RichText::new(format!(
                                    "{}{}",
                                    g.to_uppercase(),
                                    if n_mod > 0 { format!("   · {n_mod} changed") } else { String::new() }
                                ))
                                .size(10.5)
                                .strong()
                                .color(if n_mod > 0 { BLUE } else { TX });
                                egui::CollapsingHeader::new(title).id_salt((def.id, g)).default_open(true).show(ui, |ui| {
                                    for &i in &idx {
                                        let f = &def.fields[i];
                                        let v = &mut stt.vals[i];
                                        let modified = (*v - f.default).abs() > 1e-12;
                                        let row = ui.horizontal(|ui| {
                                            ui.add_space(14.0);
                                            ui.allocate_ui_with_layout(
                                                egui::vec2(150.0, 24.0),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    ui.set_width(150.0);
                                                    ui.add(egui::Label::new(RichText::new(f.label).size(12.0).color(TX)).wrap());
                                                },
                                            );
                                            if let Some(opts) = &f.options {
                                                let cur = opts.iter().find(|o| (o.0 - *v).abs() < 1e-9).map(|o| o.1).unwrap_or("-");
                                                egui::ComboBox::from_id_salt(format!("opt_{}", f.key))
                                                    .width(150.0)
                                                    .selected_text(cur)
                                                    .show_ui(ui, |ui| {
                                                        for (val, lab) in opts {
                                                            if ui.selectable_label((*val - *v).abs() < 1e-9, *lab).clicked() {
                                                                *v = *val;
                                                                ch = true;
                                                            }
                                                        }
                                                    });
                                            } else {
                                                ch |= ui
                                                    .add_sized([96.0, 24.0], egui::DragValue::new(v).speed(1.0).max_decimals(3))
                                                    .on_hover_text(format!("Default {} {}", f.default, f.unit))
                                                    .changed();
                                                ui.add_sized(
                                                    [40.0, 24.0],
                                                    egui::Label::new(RichText::new(f.unit).font(FontId::monospace(10.5)).color(MUTE)),
                                                );
                                                if modified
                                                    && ui
                                                        .small_button("↺")
                                                        .on_hover_text(format!("Reset to {} {}", f.default, f.unit))
                                                        .clicked()
                                                {
                                                    *v = f.default;
                                                    ch = true;
                                                }
                                            }
                                        });
                                        let r = row.response.rect;
                                        if modified {
                                            ui.painter().rect_filled(
                                                egui::Rect::from_min_max(
                                                    egui::pos2(r.left() + 2.0, r.top() + 4.0),
                                                    egui::pos2(r.left() + 4.0, r.bottom() - 4.0),
                                                ),
                                                0.0,
                                                BLUE,
                                            );
                                        }
                                        ui.painter().line_segment(
                                            [egui::pos2(r.left(), r.bottom() + 2.0), egui::pos2(r.right(), r.bottom() + 2.0)],
                                            Stroke::new(1.0_f32, LINE),
                                        );
                                        ui.add_space(3.0);
                                    }
                                    if idx.iter().any(|&i| def.fields[i].check_only) {
                                        ui.label(RichText::new("Values as shown on the drawing (Check Mode).").size(11.0).color(MUTE));
                                    }
                                });
                            }
                            ui.add_space(12.0);
                        });
                    });
                if ch {
                    self.run();
                }

                // ---------- verification rail ----------
                egui::SidePanel::right("out")
                    .exact_width(320.0)
                    .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::ZERO))
                    .show(ctx, |ui| {
                        pane_header(
                            ui,
                            "Verification",
                            &self
                                .result
                                .as_ref()
                                .filter(|r| r.ok)
                                .map(|r| format!("{} checks", r.checks.len()))
                                .unwrap_or_else(|| "—".into()),
                        );
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            egui::Frame::new().inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                                let Some(r) = self.result.as_ref().filter(|r| r.ok) else {
                                    ui.add_space(30.0);
                                    ui.vertical_centered(|ui| {
                                        ui.label(RichText::new("No verification yet").strong());
                                        ui.label(RichText::new("Results appear when every input is valid.").color(MUTE));
                                    });
                                    return;
                                };
                                let gov = r.checks.iter().max_by(|a, b| a.utilisation.total_cmp(&b.utilisation));
                                let review = r.checks.iter().any(|k| k.utilisation > 0.85);
                                let (t, c) = if r.checks.is_empty() {
                                    ("RESULTS", TX)
                                } else if !r.overall_pass {
                                    ("FAIL", RED)
                                } else if review {
                                    ("PASS", AMBER)
                                } else {
                                    ("PASS", GREEN)
                                };
                                let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
                                ui.painter().rect_filled(bar, 0.0, if r.checks.is_empty() { MUTE } else { c });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(t).font(FontId::monospace(30.0)).strong().color(c));
                                    if let Some(g) = gov {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                RichText::new(format!("η max {}", util_text(g.utilisation)))
                                                    .font(FontId::monospace(12.0))
                                                    .color(TX),
                                            )
                                        });
                                    }
                                });
                                ui.label(
                                    RichText::new(if r.checks.is_empty() {
                                        "Design values only; no PASS/FAIL verdict.".to_string()
                                    } else {
                                        format!(
                                            "{} of {} checks satisfied{}.",
                                            r.checks.iter().filter(|k| k.pass).count(),
                                            r.checks.len(),
                                            if review && r.overall_pass { "; at least one above the 0.85 review threshold" } else { "" }
                                        )
                                    })
                                    .size(12.0)
                                    .color(MUTE),
                                );
                                ui.label(RichText::new("Implemented checks only. Not an engineer's approval.").size(11.0).color(MUTE));
                                if let Some(g) = gov {
                                    heading(ui, "Governing limit state");
                                    egui::Frame::new()
                                        .fill(HEAD)
                                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(201, 201, 194)))
                                        .inner_margin(egui::Margin::same(10))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.label(RichText::new(&g.name).strong());
                                            ui.label(RichText::new(&g.reference).font(FontId::monospace(10.5)).color(BLUE));
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(format!("Ed {}", g.demand)).font(FontId::monospace(11.5)));
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(RichText::new(format!("Rd {}", g.capacity)).font(FontId::monospace(11.5)))
                                                });
                                            });
                                            util_bar(ui, g.utilisation, status_color(g.pass, g.utilisation), 6.0);
                                        });
                                }
                                if !r.checks.is_empty() {
                                    heading(ui, "Utilisation (Ed / Rd)");
                                }
                                for k in &r.checks {
                                    let col = status_color(k.pass, k.utilisation);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&k.name).size(12.5));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                RichText::new(util_text(k.utilisation)).font(FontId::monospace(12.5)).strong().color(col),
                                            )
                                        });
                                    });
                                    util_bar(ui, k.utilisation, col, 4.0);
                                    ui.label(
                                        RichText::new(format!("{} / {}", k.demand, k.capacity)).font(FontId::monospace(10.5)).color(MUTE),
                                    );
                                    ui.add_space(4.0);
                                    rule(ui);
                                    ui.add_space(4.0);
                                }
                                heading(ui, &format!("Observations  {}", r.observations.len()));
                                if r.observations.is_empty() {
                                    ui.label(RichText::new("None.").color(MUTE));
                                }
                                for o in &r.observations {
                                    let (c, bg) = match o.level.as_str() {
                                        "RED" => (RED, RED_BG),
                                        "AMBER" => (AMBER, AMBER_BG),
                                        _ => (BLUE, SEL),
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&o.id).font(FontId::monospace(10.5)).color(MUTE));
                                        ui.label(
                                            RichText::new(format!(" {} ", o.level))
                                                .font(FontId::monospace(10.0))
                                                .strong()
                                                .color(c)
                                                .background_color(bg),
                                        );
                                    });
                                    ui.label(RichText::new(&o.text).size(12.0).color(TX));
                                    ui.add_space(4.0);
                                    rule(ui);
                                    ui.add_space(4.0);
                                }
                                if !self.saved.is_empty() {
                                    ui.add_space(8.0);
                                    ui.label(RichText::new(&self.saved).size(11.5).color(GREEN));
                                }
                            });
                        });
                    });

                // ---------- viewport + calculation sheet ----------
                egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::ZERO)).show(ctx, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        let def = &self.defs[mi];
                        // viewport bar
                        egui::Frame::new().fill(HEAD).inner_margin(egui::Margin::symmetric(16, 6)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("SECTION · {}", def.name.to_uppercase())).size(10.5).strong().color(MUTE));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if tbtn(ui, "Dims", self.dims).on_hover_text("Show or hide dimensions").clicked() {
                                        self.dims = !self.dims;
                                    }
                                });
                            });
                        });
                        let (drawing, empty) = match &self.result {
                            Some(r) if r.ok => (r.drawing.as_ref(), "This module works on action effects; it has no section geometry."),
                            _ => (None, "Geometry appears when every input is valid."),
                        };
                        viewport::paint(ui, drawing, empty, self.dims, 340.0);
                        rule(ui);
                        egui::Frame::new().inner_margin(egui::Margin::symmetric(32, 24)).show(ui, |ui| {
                            let sheet_w = ui.available_width().min(980.0);
                            ui.set_max_width(sheet_w);
                            let Some(r) = &self.result else { return };
                            if !r.ok {
                                egui::Frame::new().fill(RED_BG).inner_margin(egui::Margin::symmetric(14, 11)).show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(RichText::new("The calculation cannot run.").strong().color(TX));
                                    for e in &r.errors {
                                        ui.label(RichText::new(format!("· {e}")).color(TX));
                                    }
                                });
                                return;
                            }
                            ui.label(
                                RichText::new(if self.st[mi].mode == "check" {
                                    "INDEPENDENT CHECK · CALCULATION SHEET"
                                } else {
                                    "DESIGN · CALCULATION SHEET"
                                })
                                .size(10.5)
                                .strong()
                                .color(MUTE),
                            );
                            ui.label(RichText::new(def.title).size(24.0).strong().color(TX));
                            ui.label(RichText::new(def.subtitle).font(FontId::monospace(11.5)).color(MUTE));
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(format!(
                                    "Parameter set {} · engine v{} · trace {}",
                                    self.st[mi].param,
                                    r.engine_version,
                                    &r.trace_hash[..12]
                                ))
                                .font(FontId::monospace(11.0))
                                .color(MUTE),
                            );
                            ui.add_space(10.0);
                            let (hr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), egui::Sense::hover());
                            ui.painter().rect_filled(hr, 0.0, TX);
                            ui.add_space(14.0);
                            if r.param_status.contains("NOT") {
                                egui::Frame::new().fill(AMBER_BG).inner_margin(egui::Margin::symmetric(14, 11)).show(ui, |ui| {
                                    ui.set_width(sheet_w - 28.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Parameter set: {}. Confirm the governing National Annex before issue for construction.",
                                            r.param_status
                                        ))
                                        .size(12.5)
                                        .color(TX),
                                    );
                                });
                                ui.add_space(10.0);
                            }
                            for w in &r.warnings {
                                ui.colored_label(AMBER, w);
                            }

                            // A. limit-state verification
                            sheet_heading(ui, "A", if r.checks.is_empty() { "Design values" } else { "Limit-state verification" });
                            if r.checks.is_empty() {
                                ui.label(
                                    RichText::new("This module combines action effects. It gives design values, not a PASS/FAIL verdict.")
                                        .size(12.0)
                                        .color(MUTE),
                                );
                            } else {
                                let gov_i =
                                    r.checks.iter().enumerate().max_by(|a, b| a.1.utilisation.total_cmp(&b.1.utilisation)).map(|x| x.0);
                                // Clause sits under the check name so the table fits the sheet width.
                                egui::Grid::new("ls").num_columns(6).spacing([14.0, 8.0]).show(ui, |ui| {
                                    for h in ["#", "CHECK · CLAUSE", "Ed", "Rd", "η", "STATUS"] {
                                        ui.label(RichText::new(h).size(10.5).strong().color(MUTE));
                                    }
                                    ui.end_row();
                                    for (i, k) in r.checks.iter().enumerate() {
                                        let col = status_color(k.pass, k.utilisation);
                                        ui.label(RichText::new(format!("{:02}", i + 1)).font(FontId::monospace(11.0)).color(MUTE));
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(&k.name).strong());
                                                if Some(i) == gov_i {
                                                    ui.label(
                                                        RichText::new(" GOVERNING ")
                                                            .font(FontId::monospace(9.5))
                                                            .strong()
                                                            .color(Color32::WHITE)
                                                            .background_color(TX),
                                                    );
                                                }
                                            });
                                            ui.label(RichText::new(&k.reference).font(FontId::monospace(10.5)).color(BLUE));
                                        });
                                        ui.label(RichText::new(&k.demand).font(FontId::monospace(11.5)));
                                        ui.label(RichText::new(&k.capacity).font(FontId::monospace(11.5)));
                                        ui.label(RichText::new(util_text(k.utilisation)).font(FontId::monospace(12.0)).strong().color(col));
                                        let (txt, bg) = if !k.pass {
                                            ("FAIL", RED_BG)
                                        } else if k.utilisation > 0.85 {
                                            ("REVIEW", AMBER_BG)
                                        } else {
                                            ("OK", Color32::from_rgb(231, 244, 236))
                                        };
                                        ui.label(
                                            RichText::new(format!(" {txt} "))
                                                .font(FontId::monospace(10.5))
                                                .strong()
                                                .color(col)
                                                .background_color(bg),
                                        );
                                        ui.end_row();
                                    }
                                });
                                ui.label(
                                    RichText::new("η = Ed / Rd. Review threshold 0.85; limit 1.00. Implemented checks only.")
                                        .size(11.0)
                                        .color(MUTE),
                                );
                            }

                            // B. result summary
                            sheet_heading(ui, "B", "Result summary");
                            egui::Grid::new("sum").num_columns(2).spacing([24.0, 6.0]).striped(false).show(ui, |ui| {
                                for k in &r.summary {
                                    ui.label(RichText::new(&k.label).color(MUTE));
                                    ui.label(RichText::new(&k.value).font(FontId::monospace(12.5)).strong());
                                    ui.end_row();
                                }
                            });

                            // C. calculation, collapsible by section
                            sheet_heading(ui, "C", &format!("Calculation  ·  {} steps, deterministic", r.trace.steps.len()));
                            // widths: number · reference · working · output, minus grid spacing and the collapsing-header indent
                            let calc_w = (sheet_w - 36.0 - 140.0 - 150.0 - 3.0 * 14.0 - 28.0).max(200.0);
                            let mut n = 0usize;
                            let mut i = 0usize;
                            while i < r.trace.steps.len() {
                                let sec = &r.trace.steps[i].section;
                                let j =
                                    r.trace.steps[i..].iter().position(|s| &s.section != sec).map(|p| i + p).unwrap_or(r.trace.steps.len());
                                egui::CollapsingHeader::new(RichText::new(format!("{}   · {} steps", sec, j - i)).strong().color(TX))
                                    .id_salt(("sec", def.id, i))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        egui::Grid::new(("steps", def.id, i)).num_columns(4).spacing([14.0, 10.0]).show(ui, |ui| {
                                            for s in &r.trace.steps[i..j] {
                                                n += 1;
                                                ui.label(RichText::new(format!("{:02}", n)).font(FontId::monospace(11.0)).color(MUTE));
                                                ui.allocate_ui_with_layout(
                                                    egui::vec2(140.0, 0.0),
                                                    egui::Layout::top_down(egui::Align::Min),
                                                    |ui| {
                                                        ui.set_width(140.0);
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(if s.reference.is_empty() { "—" } else { &s.reference })
                                                                    .font(FontId::monospace(11.0))
                                                                    .color(BLUE),
                                                            )
                                                            .wrap(),
                                                        );
                                                    },
                                                );
                                                ui.vertical(|ui| {
                                                    ui.set_width(calc_w);
                                                    ui.label(RichText::new(&s.title).strong().size(12.5));
                                                    if !s.calculation.is_empty() {
                                                        ui.add(
                                                            egui::Label::new(RichText::new(&s.calculation).font(FontId::monospace(11.5)))
                                                                .wrap(),
                                                        );
                                                    }
                                                    if !s.comment.is_empty() {
                                                        ui.add(egui::Label::new(RichText::new(&s.comment).size(11.0).color(MUTE)).wrap());
                                                    }
                                                });
                                                ui.allocate_ui_with_layout(
                                                    egui::vec2(150.0, 0.0),
                                                    egui::Layout::top_down(egui::Align::Min),
                                                    |ui| {
                                                        ui.set_width(150.0);
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(&s.output).font(FontId::monospace(11.5)).strong().color(
                                                                    match s.output.as_str() {
                                                                        "PASS" => GREEN,
                                                                        "FAIL" => RED,
                                                                        _ => TX,
                                                                    },
                                                                ),
                                                            )
                                                            .wrap(),
                                                        );
                                                    },
                                                );
                                                ui.end_row();
                                            }
                                        });
                                    });
                                i = j;
                            }

                            // D. scope
                            sheet_heading(ui, "D", "Scope and limitations");
                            for t in &r.scope_notes {
                                ui.add(egui::Label::new(RichText::new(format!("· {t}")).size(12.0).color(TX)).wrap());
                            }
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!("SHA-256 {} · engine v{}", r.trace_hash, r.engine_version))
                                    .font(FontId::monospace(10.0))
                                    .color(MUTE),
                            );
                        });
                    });
                });
            }
            Page::Project => {
                self.pj.ui(ctx, &self.defs, &mut self.cl);
            }
            Page::Account => {
                if let Some((id, name)) = self.acct.ui(ctx, &mut self.cl, &mut self.acct_job) {
                    self.pj.open_cloud(&mut self.cl, id, name);
                    self.page = Page::Project;
                }
            }
            Page::Standards => {
                egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::same(26))).show(ctx, |ui| {
                    ui.label(RichText::new("Formula registry").size(24.0).strong());
                    ui.label(RichText::new("Every implemented formula, its source clause and verification state.").color(MUTE));
                    ui.add_space(10.0);
                    card(ui, |ui| {
                        egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                            egui::Grid::new("reg").striped(true).spacing([18.0, 10.0]).show(ui, |ui| {
                                for h in ["ID", "NAME", "STANDARD", "CLAUSE", "VERIFICATION"] {
                                    ui.label(RichText::new(h).strong().color(MUTE).size(10.5));
                                }
                                ui.end_row();
                                for f in registry::formulas() {
                                    ui.label(RichText::new(f.id).font(FontId::monospace(11.5)));
                                    ui.label(f.name);
                                    ui.label(f.standard);
                                    ui.label(RichText::new(f.clause).color(BLUE));
                                    ui.label(RichText::new(f.verification).color(AMBER));
                                    ui.end_row();
                                }
                            });
                        });
                    });
                });
            }
            Page::Settings => {
                egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::same(26))).show(ctx, |ui| {
                    ui.label(RichText::new("Settings").size(24.0).strong());
                    ui.add_space(10.0);
                    card(ui, |ui| {
                        egui::Frame::new().inner_margin(egui::Margin::same(18)).show(ui, |ui| {
                            ui.label(RichText::new("SOFTWARE UPDATES").size(11.0).strong().color(MUTE));
                            ui.label(format!("Installed version: {}", update::CURRENT));
                            let busy = matches!(self.upd, Upd::Checking | Upd::Installing);
                            if ui
                                .add_enabled(!busy, egui::Button::new(RichText::new("Check updates").color(Color32::WHITE)).fill(TX))
                                .clicked()
                            {
                                let (tx, rx) = channel();
                                self.rx = Some(rx);
                                self.upd = Upd::Checking;
                                std::thread::spawn(move || {
                                    let _ = tx.send(update::check());
                                });
                            }
                            let mut go: Option<String> = None;
                            match &self.upd {
                                Upd::Idle => {}
                                Upd::Checking => {
                                    ui.label("Checking for updates...");
                                }
                                Upd::UpToDate => {
                                    ui.colored_label(GREEN, "You are on the latest version.");
                                }
                                Upd::Available(a) => {
                                    ui.colored_label(BLUE, format!("Version {} is available.", a.version));
                                    if !a.notes.is_empty() {
                                        ui.label(RichText::new(&a.notes).size(12.0).color(MUTE));
                                    }
                                    if ui.button("Download and install (app will restart)").clicked() {
                                        go = Some(a.url.clone());
                                    }
                                }
                                Upd::Installing => {
                                    ui.label("Downloading the installer. StruCal will close and reopen when it finishes...");
                                }
                                Upd::Failed(e) => {
                                    ui.colored_label(RED, e);
                                }
                            }
                            if let Some(u) = go {
                                let (tx, rx) = channel();
                                self.irx = Some(rx);
                                self.upd = Upd::Installing;
                                std::thread::spawn(move || {
                                    let _ = tx.send(update::install(&u));
                                });
                            }
                            ui.add_space(12.0);
                            ui.label(RichText::new("Updates install in place. You do not need to uninstall first.").color(MUTE).size(12.0));
                        });
                    });
                });
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 860.0])
            .with_min_inner_size([1100.0, 650.0])
            .with_title("StruCal")
            .with_icon(std::sync::Arc::new(egui::IconData { rgba: include_bytes!("../assets/icon.rgba").to_vec(), width: 64, height: 64 })),
        ..Default::default()
    };
    eframe::run_native("StruCal", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
