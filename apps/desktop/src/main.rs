#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod update;
mod project_page;
mod cloud;
mod account_page;
mod assistant_page;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, StrokeKind};
use std::sync::mpsc::{channel, Receiver};
use strucal_engine::{modules, module::{Drawing, ModDef, ModResult}, params, registry, report};

const BG: Color32 = Color32::from_rgb(240, 240, 240);
const LINE: Color32 = Color32::from_rgb(207, 212, 218);
const HEAD: Color32 = Color32::from_rgb(236, 238, 241);
const RIBBON: Color32 = Color32::from_rgb(247, 248, 250);
const SEL: Color32 = Color32::from_rgb(197, 220, 242);
const TX: Color32 = Color32::from_rgb(15, 23, 42);
const MUTE: Color32 = Color32::from_rgb(100, 116, 139);
const BLUE: Color32 = Color32::from_rgb(11, 92, 173);
const GREEN: Color32 = Color32::from_rgb(15, 123, 90);
const GREEN_BG: Color32 = Color32::from_rgb(230, 246, 239);
const AMBER: Color32 = Color32::from_rgb(217, 138, 0);
const RED: Color32 = Color32::from_rgb(180, 35, 24);
const RED_BG: Color32 = Color32::from_rgb(253, 236, 235);

#[derive(PartialEq, Clone, Copy)] enum Page { Workspace, Project, Account, Standards, Settings }
enum Upd { Idle, Checking, UpToDate, Available(update::Available), Installing, Failed(String) }

struct ModState { mode: String, param: String, vals: Vec<f64> }
struct App {
    page: Page, defs: Vec<ModDef>, mi: usize, st: Vec<ModState>, result: Option<ModResult>, saved: String,
    upd: Upd, rx: Option<Receiver<Result<Option<update::Available>, String>>>, irx: Option<Receiver<Result<(), String>>>,
    pj: project_page::Pj, cl: cloud::Client, acct: account_page::Acct, acct_job: account_page::AcctJob, asst: assistant_page::Asst,
}

fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).corner_radius(CornerRadius::ZERO).inner_margin(egui::Margin::same(0)).show(ui, add).inner
}
fn panel_header(ui: &mut egui::Ui, t: &str) {
    egui::Frame::new().fill(HEAD).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::symmetric(7, 2)).show(ui, |ui| { ui.set_width(ui.available_width()); ui.label(RichText::new(t).size(11.5).color(Color32::from_rgb(51, 51, 51))); });
    ui.add_space(4.0);
}
fn group(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| { ui.horizontal(add); ui.add_space(2.0); ui.label(RichText::new(label).size(10.5).color(MUTE)); });
    ui.add(egui::Separator::default().vertical());
}
fn rbtn(ui: &mut egui::Ui, t: &str, on: bool) -> egui::Response {
    ui.add_sized([74.0, 50.0], egui::Button::new(RichText::new(t).size(12.0)).selected(on).fill(if on { SEL } else { Color32::TRANSPARENT }).corner_radius(CornerRadius::ZERO))
}
fn heading(ui: &mut egui::Ui, t: &str) { ui.add_space(6.0); ui.label(RichText::new(t).size(11.0).strong().color(BLUE)); }

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut v = egui::Visuals::light();
        v.panel_fill = BG; v.window_fill = Color32::WHITE;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(207, 216, 227));
        v.selection.bg_fill = BLUE;
        v.window_corner_radius = CornerRadius::ZERO; v.menu_corner_radius = CornerRadius::ZERO;
        for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] { w.corner_radius = CornerRadius::ZERO; }
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(219, 232, 246); v.widgets.hovered.bg_fill = Color32::from_rgb(219, 232, 246);
        cc.egui_ctx.set_visuals(v);
        let mut st = (*cc.egui_ctx.style()).clone();
        st.spacing.item_spacing = egui::vec2(6.0, 4.0); st.spacing.button_padding = egui::vec2(8.0, 3.0);
        for (t, sz) in [(egui::TextStyle::Body, 12.5), (egui::TextStyle::Button, 12.5), (egui::TextStyle::Small, 10.5), (egui::TextStyle::Monospace, 12.0)] { if let Some(f) = st.text_styles.get_mut(&t) { f.size = sz; } }
        cc.egui_ctx.set_style(st);
        update::cleanup_old();
        let defs = modules::defs();
        let st = defs.iter().map(|d| ModState { mode: "design".into(), param: "EN1992-1-1:2004/RECOMMENDED".into(), vals: d.fields.iter().map(|f| f.default).collect() }).collect();
        let mut a = App { page: Page::Workspace, defs, mi: 0, st, result: None, saved: String::new(), upd: Upd::Idle, rx: None, irx: None, pj: project_page::Pj::new(), cl: cloud::Client::default(), acct: account_page::Acct::new(), acct_job: account_page::AcctJob::default(), asst: assistant_page::Asst::default() };
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
        ui.label(RichText::new(label).size(11.5).color(Color32::from_rgb(51, 51, 51)));
        *ch |= ui.add_sized([ui.available_width(), 22.0], egui::DragValue::new(v).speed(1.0).max_decimals(3).suffix(format!(" {unit}"))).changed();
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
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                p.rect_stroke(outer.shrink(((cover + link_dia / 2.0) as f32) * sc), 0.0, Stroke::new(1.2_f32, Color32::from_rgb(51, 65, 85)), StrokeKind::Inside);
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
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                p.rect_stroke(outer.shrink(((cover + link_dia / 2.0) as f32) * sc), 0.0, Stroke::new(1.2_f32, Color32::from_rgb(51, 65, 85)), StrokeKind::Inside);
                let ed = ((cover + link_dia + bar_dia / 2.0) as f32) * sc; let n = (*n_face).max(2);
                let br = ((*bar_dia as f32 * sc) / 2.0).max(2.4);
                for y in [o.y + ed, o.y + hh - ed] { for k in 0..n { p.circle_filled(egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32 / (n - 1) as f32, y), br, TX); } }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{b} × {h}"));
            }
            Drawing::Bolts { w, n1, n2, e1, p1, e2: _, p2, d } => {
                let len = 2.0 * e1 + (*n1 as f64 - 1.0) * p1; let sc = (200.0 / len.max(*w)) as f32; let (ww, hh) = (len as f32 * sc, *w as f32 * sc);
                let o = egui::pos2(rect.center().x - ww / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(ww, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                let y0 = o.y + (((*w - (*n2 as f64 - 1.0) * p2) / 2.0) as f32) * sc;
                for i in 0..*n1 { for j in 0..*n2 { let c = egui::pos2(o.x + ((e1 + i as f64 * p1) as f32) * sc, y0 + (j as f64 * p2) as f32 * sc); p.circle_filled(c, ((*d as f32 * sc) / 2.0).max(3.0), Color32::WHITE); p.circle_stroke(c, ((*d as f32 * sc) / 2.0).max(3.0), Stroke::new(1.6_f32, TX)); } }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{n1} × {n2} bolts, plate {w} wide"));
            }
            Drawing::Weld { lw, a } => {
                let c = rect.center(); let steel = Color32::from_rgb(51, 65, 85);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(c.x - 90.0, c.y - 20.0), egui::vec2(180.0, 16.0)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(c.x - 90.0, c.y - 4.0), egui::vec2(180.0, 16.0)), 0.0, Color32::from_rgb(148, 163, 184));
                p.add(egui::Shape::convex_polygon(vec![egui::pos2(c.x - 90.0, c.y + 12.0), egui::pos2(c.x - 90.0, c.y + 25.0), egui::pos2(c.x - 77.0, c.y + 12.0)], TX, Stroke::NONE));
                lab(&p, egui::pos2(c.x, c.y + 50.0), format!("a = {a} mm, L = {lw} mm"));
            }
            Drawing::Raft { l, w, n, spacing, edge } => {
                let sc = (200.0 / l.max(*w)) as f32; let (ww, hh) = (*l as f32 * sc, *w as f32 * sc);
                let o = egui::pos2(rect.center().x - ww / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(ww, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                for i in 0..*n { p.rect_filled(egui::Rect::from_center_size(egui::pos2(o.x + ((edge + i as f64 * spacing) as f32) * sc, o.y + hh / 2.0), egui::vec2(10.0, 10.0)), 0.0, Color32::from_rgb(51, 65, 85)); }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("L = {:.0} / w = {w} (plan)", l));
            }
            Drawing::Retain { hs, tb, ts, lt, b } => {
                let sc = (190.0 / (hs + tb).max(*b)) as f32; let (w, hh) = (*b as f32 * sc, (*hs + *tb) as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0); let steel = Color32::from_rgb(51, 65, 85);
                let (lts, tss, hss, tbs) = (*lt as f32 * sc, *ts as f32 * sc, *hs as f32 * sc, *tb as f32 * sc);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x + lts + tss, o.y), egui::vec2(w - lts - tss, hss)), 0.0, Color32::from_rgb(226, 232, 240));
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x, o.y + hss), egui::vec2(w, tbs)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x + lts, o.y), egui::vec2(tss, hss)), 0.0, steel);
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("H = {} / B = {b}", hs + tb));
            }
            Drawing::Rect { b, h } => {
                let sc = (150.0 / b).min(250.0 / h) as f32; let (w, hh) = (*b as f32 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0); let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::from_rgb(232, 216, 189)); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{b} × {h}"));
            }
            Drawing::ISection { h, b, tw, tf } => {
                let sc = (160.0 / b).min(250.0 / h) as f32; let (w, hh, wt, ft) = (*b as f32 * sc, *h as f32 * sc, *tw as f32 * sc, *tf as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0); let steel = Color32::from_rgb(51, 65, 85);
                p.rect_filled(egui::Rect::from_min_size(o, egui::vec2(w, ft)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x, o.y + hh - ft), egui::vec2(w, ft)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x + (w - wt) / 2.0, o.y), egui::vec2(wt, hh)), 0.0, steel);
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{h} × {b} × {tw} / {tf}"));
            }
            Drawing::Wall { h, cover, bar_dia, spacing } => {
                let sc = (120.0 / h).min(1.0) as f32; let (w, hh) = (*h as f32 * sc, 260.0f32);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                let mut y = *spacing as f32 / 2.0;
                while y < 400.0 { for xx in [o.x + ((cover + 10.0) as f32) * sc, o.x + w - ((cover + 10.0) as f32) * sc] { p.circle_filled(egui::pos2(xx, o.y + y / 400.0 * hh), ((*bar_dia as f32 * sc) / 2.0).max(2.2), TX); } y += *spacing as f32; }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("h = {h}, T{bar_dia} @ {spacing}"));
            }
            Drawing::Strip { width, t, h } => {
                let sc = (190.0 / width) as f32; let (w, hh, tw) = (*width as f32 * sc, *h as f32 * sc, *t as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x + (w - tw) / 2.0, o.y - 50.0), egui::vec2(tw, 50.0)), 0.0, Color32::from_rgb(51, 65, 85));
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                p.line_segment([egui::pos2(o.x + 8.0, o.y + hh - 8.0), egui::pos2(o.x + w - 8.0, o.y + hh - 8.0)], Stroke::new(2.5_f32, TX));
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("B = {width}, h = {h}"));
            }
            Drawing::Slab { h, cover, bar_dia, spacing } => {
                let sc = 0.5f32; let (w, hh) = (400.0 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                let mut x = *spacing as f32 / 2.0;
                while x < 400.0 { p.circle_filled(egui::pos2(o.x + x * sc, o.y + hh - ((cover + bar_dia / 2.0) as f32) * sc), ((*bar_dia as f32 * sc) / 2.0).max(2.2), TX); x += *spacing as f32; }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("T{} @ {}", bar_dia, spacing));
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 32.0), format!("h = {h} (1 m strip)"));
            }
            Drawing::Pad { bx, by, cx, cy, h } => {
                let sc = (170.0 / bx.max(*by)) as f32; let (w, hh) = (*bx as f32 * sc, *by as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE); p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                for i in 1..7 { let y = o.y + hh * i as f32 / 7.0; p.line_segment([egui::pos2(o.x + 6.0, y), egui::pos2(o.x + w - 6.0, y)], Stroke::new(1.0_f32, Color32::from_rgb(148, 163, 184))); }
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
        egui::TopBottomPanel::top("tabs").exact_height(26.0).frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(egui::Margin::ZERO)).show(ctx, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal_centered(|ui| {
                let (m, _) = ui.allocate_exact_size(egui::vec2(30.0, 26.0), egui::Sense::hover());
                ui.painter().text(m.center(), egui::Align2::CENTER_CENTER, "S", FontId::proportional(21.0), BLUE);
                let (r, _) = ui.allocate_exact_size(egui::vec2(64.0, 26.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 0.0, BLUE); ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "FILE", FontId::proportional(12.0), Color32::WHITE);
                for (p, n) in [(Page::Workspace, "WORKSPACE"), (Page::Project, "PROJECT"), (Page::Account, "ACCOUNT"), (Page::Standards, "STANDARDS"), (Page::Settings, "SETTINGS")] {
                    let on = self.page == p;
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(96.0, 26.0), egui::Sense::click());
                    if resp.hovered() { ui.painter().rect_filled(r, 0.0, Color32::from_rgb(219, 232, 246)); }
                    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, n, FontId::proportional(12.0), if on { BLUE } else { Color32::from_rgb(51, 51, 51) });
                    if on { ui.painter().line_segment([egui::pos2(r.left(), r.bottom() - 1.0), egui::pos2(r.right(), r.bottom() - 1.0)], Stroke::new(2.0_f32, BLUE)); }
                    if resp.clicked() { self.page = p; }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.add_space(10.0); ui.label(RichText::new(format!("StruCal v{}", update::CURRENT)).color(MUTE).size(11.0)); });
            });
        });
        if self.page == Page::Workspace {
            egui::TopBottomPanel::top("ribbon").exact_height(86.0).frame(egui::Frame::new().fill(RIBBON).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::symmetric(6, 4))).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    group(ui, "Document", |ui| {
                        if rbtn(ui, "Reset", false).on_hover_text("Reset this module's inputs to their defaults").clicked() { let d = &self.defs[self.mi]; self.st[self.mi].vals = d.fields.iter().map(|f| f.default).collect(); self.saved.clear(); mode_changed = true; }
                        ui.add_enabled_ui(self.result.as_ref().is_some_and(|r| r.ok), |ui| { if rbtn(ui, "Export", false).clicked() { do_export = true; } });
                    });
                    group(ui, "Mode", |ui| {
                        for (m, n) in [("design", "Design"), ("check", "Check Mode")] {
                        if m == "check" && !self.defs[self.mi].has_check_mode { continue; }
                            let on = self.st[self.mi].mode == m;
                            if rbtn(ui, n, on).clicked() { self.st[self.mi].mode = m.into(); mode_changed = true; }
                        }
                    });
                    group(ui, "Design basis", |ui| { ui.vertical(|ui| {
                        ui.horizontal(|ui| { ui.label("Module"); ui.add_space(14.0);
                            egui::ComboBox::from_id_salt("mod").width(230.0).selected_text(self.defs[self.mi].name).show_ui(ui, |ui| {
                                for i in 0..self.defs.len() { if ui.selectable_value(&mut self.mi, i, self.defs[i].name).changed() { mode_changed = true; } }
                                for u in modules::upcoming() { ui.add_enabled(false, egui::Label::new(format!("{} - {}", u.name, u.note))); }
                            });
                        });
                        let stt = &mut self.st[self.mi];
                        ui.horizontal(|ui| { ui.label("Parameters");
                            egui::ComboBox::from_id_salt("ps").width(230.0).selected_text(params::get(&stt.param).map(|p| p.label).unwrap_or_default()).show_ui(ui, |ui| {
                                for p in params::available() { if ui.selectable_value(&mut stt.param, p.id.clone(), p.label).changed() { mode_changed = true; } }
                            });
                        });
                    }); });
                });
            });
        }
        egui::TopBottomPanel::bottom("status").exact_height(22.0).frame(egui::Frame::new().fill(BLUE).inner_margin(egui::Margin::symmetric(10, 0))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let t = |s: String| RichText::new(s).size(11.0).color(Color32::WHITE);
                match &self.result { Some(r) if r.ok => { ui.label(t(if r.checks.is_empty() { "● Calculated".into() } else if r.overall_pass { "● All checks satisfied".into() } else { "● Checks failing".into() })); ui.add_space(14.0); ui.label(t(self.st[self.mi].param.clone())); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(t("Preview build".into())); ui.add_space(14.0); ui.label(t(format!("Engine v{}", r.engine_version))); ui.add_space(14.0); ui.label(t(format!("Trace {}", &r.trace_hash[..8]))); }); }
                    _ => { ui.label(t("● Not calculated".into())); } }
            });
        });
        if mode_changed { self.run(); }
        if do_export { self.export(); }

        match self.page {
            Page::Workspace => {
                self.assistant_ui(ctx);
                let mut ch = false;
                egui::SidePanel::left("in").exact_width(280.0).frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::same(8))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        panel_header(ui, if self.st[self.mi].mode == "check" { "Input data - Check" } else { "Input data - Design" });
                        let stt = &mut self.st[self.mi];
                        let mut last = "";
                        for (f, v) in self.defs[self.mi].fields.iter().zip(stt.vals.iter_mut()) {
                            if f.check_only && stt.mode != "check" { continue; }
                            if f.group != last { heading(ui, f.group); last = f.group; }
                            if let Some(opts) = &f.options {
                                ui.label(RichText::new(f.label).size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                let cur = opts.iter().find(|o| (o.0 - *v).abs() < 1e-9).map(|o| o.1).unwrap_or("-");
                                egui::ComboBox::from_id_salt(format!("opt_{}", f.key)).width(ui.available_width() - 8.0).selected_text(cur).show_ui(ui, |ui| { for (val, lab) in opts { if ui.selectable_label((*val - *v).abs() < 1e-9, *lab).clicked() { *v = *val; ch = true; } } });
                            } else { Self::field(ui, f.label, f.unit, v, &mut ch); }
                        }
                    });
                });
                if ch { self.run(); }

                egui::SidePanel::right("out").exact_width(320.0).frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::same(8))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        panel_header(ui, "Verification");
                        if let Some(r) = &self.result { if r.ok {
                            let (t, c, bg) = if r.checks.is_empty() { ("RESULTS", BLUE, Color32::from_rgb(232, 240, 253)) } else if r.overall_pass { ("PASS", GREEN, GREEN_BG) } else { ("FAIL", RED, RED_BG) };
                            egui::Frame::new().fill(bg).stroke(Stroke::new(1.0_f32, c.gamma_multiply(0.45))).corner_radius(CornerRadius::ZERO).inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| { ui.label(RichText::new(t).font(FontId::monospace(22.0)).strong().color(c)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(if r.checks.is_empty() { String::new() } else { format!("Max {:.0}%", r.max_util * 100.0) }).strong().color(c))); });
                                ui.label(RichText::new(if r.checks.is_empty() { "Combined action effects. Design values only, no PASS/FAIL verdict.".to_string() } else { format!("{}/{} checks satisfied. Implemented checks only; not an engineer's approval.", r.checks.iter().filter(|k| k.pass).count(), r.checks.len()) }).size(12.0).color(c));
                            });
                            if !r.checks.is_empty() { heading(ui, "Utilisation ratios"); }
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
                                egui::Frame::new().stroke(Stroke::new(1.0_f32, LINE)).corner_radius(CornerRadius::ZERO).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| { ui.label(RichText::new(&o.id).font(FontId::monospace(11.0)).strong()); ui.label(RichText::new(&o.level).font(FontId::monospace(10.0)).strong().color(c)); });
                                    ui.label(RichText::new(&o.text).size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                });
                            }
                            if !self.saved.is_empty() { ui.add_space(8.0); ui.label(RichText::new(&self.saved).size(11.5).color(GREEN)); }
                        } }
                    });
                });

                egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::symmetric(18, 14))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(RichText::new(format!("{} - {}", self.defs[self.mi].title, if self.st[self.mi].mode == "check" { "independent check" } else { "design" })).size(17.0).strong().color(TX));
                        ui.label(RichText::new(self.defs[self.mi].subtitle).color(MUTE));
                        ui.add_space(10.0);
                        if let Some(r) = &self.result {
                            for e in &r.errors { ui.colored_label(RED, e); }
                            if r.ok {
                                if r.param_status.contains("NOT") {
                                    egui::Frame::new().fill(Color32::from_rgb(255, 247, 224)).stroke(Stroke::new(1.0_f32, Color32::from_rgb(241, 211, 138))).corner_radius(CornerRadius::ZERO).inner_margin(egui::Margin::same(11)).show(ui, |ui| { ui.set_width(ui.available_width()); ui.label(RichText::new(format!("Parameter set: {}. Confirm the governing National Annex before issue for construction.", r.param_status)).size(12.5).color(Color32::from_rgb(91, 65, 8))); });
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
            Page::Project => { self.pj.ui(ctx, &self.defs, &mut self.cl); }
            Page::Account => { if let Some((id, name)) = self.acct.ui(ctx, &mut self.cl, &mut self.acct_job) { self.pj.open_cloud(&mut self.cl, id, name); self.page = Page::Project; } }
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
    let opts = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 860.0]).with_min_inner_size([1100.0, 650.0]).with_title("StruCal").with_icon(std::sync::Arc::new(egui::IconData { rgba: include_bytes!("../assets/icon.rgba").to_vec(), width: 64, height: 64 })), ..Default::default() };
    eframe::run_native("StruCal", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
