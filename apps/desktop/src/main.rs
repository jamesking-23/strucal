#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod update;
use eframe::egui::{self, Color32, RichText};
use std::sync::mpsc::{channel, Receiver};
use strucal_engine::{params, rc_beam::{self, BeamInput, BeamResult}, registry};

const NAVY: Color32 = Color32::from_rgb(10, 37, 64);
const GOLD: Color32 = Color32::from_rgb(184, 137, 58);
const OK: Color32 = Color32::from_rgb(19, 121, 91);
const BAD: Color32 = Color32::from_rgb(180, 35, 24);
const MUTE: Color32 = Color32::from_rgb(107, 124, 144);

#[derive(PartialEq, Clone, Copy)] enum Page { Workspace, Standards, Settings }
enum Upd { Idle, Checking, UpToDate, Available(update::Available), Installing, Failed(String) }

struct App {
    page: Page, input: BeamInput, result: Option<BeamResult>, saved: String,
    upd: Upd, rx: Option<Receiver<Result<Option<update::Available>, String>>>, irx: Option<Receiver<Result<(), String>>>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut v = egui::Visuals::light();
        v.panel_fill = Color32::from_rgb(245, 247, 250);
        v.widgets.active.bg_fill = NAVY; v.selection.bg_fill = NAVY;
        cc.egui_ctx.set_visuals(v);
        let mut st = (*cc.egui_ctx.style()).clone();
        st.spacing.item_spacing = egui::vec2(8.0, 7.0); st.spacing.button_padding = egui::vec2(12.0, 6.0);
        cc.egui_ctx.set_style(st);
        update::cleanup_old();
        let input = BeamInput { mode: "design".into(), param_set: "EN1992-1-1:2004/RECOMMENDED".into(), b: 300.0, h: 500.0, cover: 30.0, link_dia: 8.0, bar_dia: 20.0, fck: 30.0, fyk: 500.0, m_ed: 200.0, v_ed: 120.0, as_provided: Some(1256.0), asw_s_provided: Some(0.67) };
        let mut a = App { page: Page::Workspace, input, result: None, saved: String::new(), upd: Upd::Idle, rx: None, irx: None };
        a.run(); a
    }
    fn run(&mut self) {
        let mut i = self.input.clone();
        if i.mode != "check" { i.as_provided = None; i.asw_s_provided = None; }
        self.result = Some(rc_beam::run(&i));
    }
    fn num(ui: &mut egui::Ui, label: &str, unit: &str, v: &mut f64, ch: &mut bool) {
        ui.label(RichText::new(format!("{label} ({unit})")).size(11.5).strong().color(Color32::from_rgb(52, 70, 92)));
        *ch |= ui.add(egui::DragValue::new(v).speed(1.0).max_decimals(3)).changed();
    }
    fn export(&mut self) {
        let Some(r) = &self.result else { return };
        if !r.ok { return; }
        let dir = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(std::path::PathBuf::from).unwrap_or_default().join("Documents").join("StruCal");
        let _ = std::fs::create_dir_all(&dir);
        let stem = format!("rc-beam-{}", &r.trace_hash[..8]);
        let mut rows = String::new();
        for s in &r.trace.steps { rows += &format!("<tr><td>{}</td><td>{}</td><td><b>{}</b></td></tr>", esc(&s.reference), esc(&s.calculation), esc(&s.output)); }
        let html = format!("<html><head><meta charset='utf-8'><title>StruCal calculation</title><style>body{{font-family:Segoe UI,sans-serif;margin:30px}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ccc;padding:7px;font-size:13px;text-align:left}}th{{background:#0a2540;color:#fff}}</style></head><body><h2>StruCal - RC beam calculation</h2><p>Result: <b>{}</b> | Trace hash: {} | Engine {}</p><table><tr><th>Reference</th><th>Calculation</th><th>Output</th></tr>{}</table><p><i>{}</i></p></body></html>",
            if r.overall_pass { "PASS" } else { "FAIL" }, r.trace_hash, r.engine_version, rows, esc(&r.scope_notes.join(" ")));
        let p = dir.join(format!("{stem}.html"));
        self.saved = match std::fs::write(&p, html) { Ok(_) => format!("Saved {} (open in a browser and print to PDF)", p.display()), Err(e) => format!("Save failed: {e}") };
    }
}
fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        if let Some(rx) = &self.rx { if let Ok(m) = rx.try_recv() { self.upd = match m { Ok(Some(a)) => Upd::Available(a), Ok(None) => Upd::UpToDate, Err(e) => Upd::Failed(e) }; self.rx = None; } ctx.request_repaint_after(std::time::Duration::from_millis(200)); }
        if let Some(rx) = &self.irx { if let Ok(m) = rx.try_recv() { if let Err(e) = m { self.upd = Upd::Failed(e); } self.irx = None; } ctx.request_repaint_after(std::time::Duration::from_millis(200)); }

        egui::TopBottomPanel::top("top").frame(egui::Frame::none().fill(NAVY).inner_margin(egui::Margin::symmetric(18, 12))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("S").color(GOLD).strong().size(20.0));
                ui.label(RichText::new("STRUCAL").color(Color32::WHITE).strong().size(16.0));
                ui.add_space(24.0);
                for (p, n) in [(Page::Workspace, "RC Beam"), (Page::Standards, "Standards"), (Page::Settings, "Settings")] {
                    let on = self.page == p;
                    if ui.selectable_label(on, RichText::new(n).color(if on { GOLD } else { Color32::from_rgb(201, 214, 230) })).clicked() { self.page = p; }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new(format!("Preview build v{}", update::CURRENT)).color(Color32::from_rgb(157, 176, 198)).size(12.0)); });
            });
        });

        match self.page {
            Page::Workspace => {
                let mut ch = false;
                egui::SidePanel::left("in").exact_width(270.0).frame(egui::Frame::none().fill(Color32::WHITE).inner_margin(16.0)).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(self.input.mode == "design", "Design").clicked() { self.input.mode = "design".into(); ch = true; }
                            if ui.selectable_label(self.input.mode == "check", "Check Mode").clicked() { self.input.mode = "check".into(); ch = true; }
                        });
                        ui.separator(); ui.label(RichText::new("DESIGN BASIS").strong().color(NAVY));
                        egui::ComboBox::from_id_salt("ps").width(240.0).selected_text(params::get(&self.input.param_set).map(|p| p.label).unwrap_or_default()).show_ui(ui, |ui| {
                            for p in params::available() { if ui.selectable_value(&mut self.input.param_set, p.id.clone(), p.label).changed() { ch = true; } }
                        });
                        ui.separator(); ui.label(RichText::new("GEOMETRY").strong().color(NAVY));
                        Self::num(ui, "Width b", "mm", &mut self.input.b, &mut ch); Self::num(ui, "Depth h", "mm", &mut self.input.h, &mut ch);
                        Self::num(ui, "Cover", "mm", &mut self.input.cover, &mut ch); Self::num(ui, "Link dia", "mm", &mut self.input.link_dia, &mut ch);
                        Self::num(ui, "Main bar dia (assumed)", "mm", &mut self.input.bar_dia, &mut ch);
                        ui.separator(); ui.label(RichText::new("MATERIALS").strong().color(NAVY));
                        Self::num(ui, "fck", "MPa", &mut self.input.fck, &mut ch); Self::num(ui, "fyk", "MPa", &mut self.input.fyk, &mut ch);
                        ui.separator(); ui.label(RichText::new("ACTIONS (ULS)").strong().color(NAVY));
                        Self::num(ui, "M Ed", "kNm", &mut self.input.m_ed, &mut ch); Self::num(ui, "V Ed", "kN", &mut self.input.v_ed, &mut ch);
                        if self.input.mode == "check" {
                            ui.separator(); ui.label(RichText::new("PROVIDED (DRAWING)").strong().color(NAVY));
                            let (mut a, mut s) = (self.input.as_provided.unwrap_or(0.0), self.input.asw_s_provided.unwrap_or(0.0));
                            Self::num(ui, "As provided", "mm2", &mut a, &mut ch); Self::num(ui, "Asw/s provided", "mm2/mm", &mut s, &mut ch);
                            self.input.as_provided = Some(a); self.input.asw_s_provided = Some(s);
                        }
                    });
                });
                if ch { self.run(); }

                egui::SidePanel::right("out").exact_width(300.0).frame(egui::Frame::none().fill(Color32::WHITE).inner_margin(16.0)).show(ctx, |ui| {
                    let mut do_export = false;
                    if let Some(r) = &self.result { if r.ok {
                        let (t, c) = if r.overall_pass { ("PASS", OK) } else { ("FAIL", BAD) };
                        ui.vertical_centered(|ui| ui.label(RichText::new(t).size(26.0).strong().color(c)));
                        ui.separator(); ui.label(RichText::new("UTILISATION").strong().color(NAVY));
                        for k in &r.checks {
                            ui.horizontal(|ui| { ui.label(RichText::new(&k.name).strong()); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(if k.utilisation.is_finite() { format!("{:.0}%", k.utilisation * 100.0) } else { "n/a".into() }).color(if k.pass { OK } else { BAD }).strong())); });
                            ui.add(egui::ProgressBar::new(if k.utilisation.is_finite() { k.utilisation.min(1.0) as f32 } else { 1.0 }).fill(if k.pass { OK } else { BAD }).desired_height(6.0));
                            ui.label(RichText::new(format!("{} vs {}", k.demand, k.capacity)).size(11.5).color(MUTE));
                        }
                        ui.separator(); ui.label(RichText::new("RESULT").strong().color(NAVY));
                        ui.label(format!("d = {:.0} mm\nAs,req = {:.0} mm2\nAs,min = {:.0} / As,max = {:.0} mm2\nTension: {}\nShear: {}", r.d, r.as_req, r.as_min, r.as_max, r.bars, r.links));
                        ui.add_space(8.0);
                        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new(RichText::new("Export calculation sheet").color(Color32::WHITE)).fill(NAVY)).clicked() { do_export = true; }
                        if !self.saved.is_empty() { ui.label(RichText::new(&self.saved).size(11.0).color(MUTE)); }
                    } }
                    if do_export { self.export(); }
                });

                egui::CentralPanel::default().frame(egui::Frame::none().inner_margin(24.0)).show(ctx, |ui| {
                    ui.label(RichText::new(format!("Reinforced concrete beam - {}", if self.input.mode == "check" { "independent check" } else { "design" })).size(22.0).strong());
                    ui.label(RichText::new("EN 1992-1-1:2004 - rectangular section, sagging moment and shear").color(MUTE));
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(r) = &self.result {
                            for e in &r.errors { ui.colored_label(BAD, e); }
                            for w in &r.warnings { ui.colored_label(Color32::from_rgb(154, 103, 0), w); }
                            if r.ok {
                                egui::Grid::new("calc").striped(true).num_columns(3).spacing([14.0, 9.0]).min_col_width(120.0).show(ui, |ui| {
                                    for h in ["REFERENCE", "CALCULATION", "OUTPUT"] { ui.label(RichText::new(h).strong().color(NAVY).size(11.5)); }
                                    ui.end_row();
                                    for s in &r.trace.steps {
                                        ui.label(RichText::new(&s.reference).color(MUTE)); ui.add(egui::Label::new(&s.calculation).wrap()); ui.label(RichText::new(&s.output).strong()); ui.end_row();
                                    }
                                });
                                ui.add_space(10.0);
                                ui.label(RichText::new(format!("Trace hash (SHA-256): {} - engine {}", r.trace_hash, r.engine_version)).size(11.0).color(MUTE));
                                for n in &r.scope_notes { ui.label(RichText::new(n).size(12.0).color(Color32::from_rgb(120, 90, 30))); }
                            }
                        }
                    });
                });
            }
            Page::Standards => { egui::CentralPanel::default().frame(egui::Frame::none().inner_margin(24.0)).show(ctx, |ui| {
                ui.label(RichText::new("Formula registry").size(22.0).strong());
                ui.label(RichText::new("Every implemented formula, its source clause and verification state.").color(MUTE)); ui.add_space(10.0);
                egui::Grid::new("reg").striped(true).spacing([16.0, 9.0]).show(ui, |ui| {
                    for h in ["ID", "NAME", "STANDARD", "CLAUSE", "VERIFICATION"] { ui.label(RichText::new(h).strong().color(NAVY).size(11.5)); } ui.end_row();
                    for f in registry::formulas() { ui.label(f.id); ui.label(f.name); ui.label(f.standard); ui.label(f.clause); ui.label(RichText::new(f.verification).color(Color32::from_rgb(154, 103, 0))); ui.end_row(); }
                });
            }); }
            Page::Settings => { egui::CentralPanel::default().frame(egui::Frame::none().inner_margin(24.0)).show(ctx, |ui| {
                ui.label(RichText::new("Settings").size(22.0).strong()); ui.add_space(10.0);
                ui.label(RichText::new("SOFTWARE UPDATES").strong().color(NAVY));
                ui.label(format!("Installed version: {}", update::CURRENT));
                let busy = matches!(self.upd, Upd::Checking | Upd::Installing);
                if ui.add_enabled(!busy, egui::Button::new(RichText::new("Check updates").color(Color32::WHITE)).fill(NAVY)).clicked() {
                    let (tx, rx) = channel(); self.rx = Some(rx); self.upd = Upd::Checking;
                    std::thread::spawn(move || { let _ = tx.send(update::check()); });
                }
                let mut go: Option<String> = None;
                match &self.upd {
                    Upd::Idle => {}, Upd::Checking => { ui.label("Checking for updates..."); }
                    Upd::UpToDate => { ui.colored_label(OK, "You are on the latest version."); }
                    Upd::Available(a) => { ui.colored_label(GOLD, format!("Version {} is available.", a.version)); if !a.notes.is_empty() { ui.label(RichText::new(&a.notes).size(12.0).color(MUTE)); }
                        if ui.button("Download and install (app will restart)").clicked() { go = Some(a.url.clone()); } }
                    Upd::Installing => { ui.label("Downloading and installing..."); }
                    Upd::Failed(e) => { ui.colored_label(BAD, e); }
                }
                if let Some(u) = go { let (tx, rx) = channel(); self.irx = Some(rx); self.upd = Upd::Installing; std::thread::spawn(move || { let _ = tx.send(update::install(&u)); }); }
                ui.add_space(20.0); ui.label(RichText::new("Your data and settings are kept across updates.").color(MUTE).size(12.0));
            }); }
        }
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([1360.0, 840.0]).with_min_inner_size([1000.0, 640.0]).with_title("StruCal"), ..Default::default() };
    eframe::run_native("StruCal", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
