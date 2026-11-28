#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod account_page;
mod assistant_page;
mod cloud;
mod project_page;
mod update;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, StrokeKind};
use std::sync::mpsc::{channel, Receiver};
use strucal_engine::{
    module::{Drawing, ModDef, ModResult},
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
/// Card header row: title and an optional muted note.
fn card_head(ui: &mut egui::Ui, title: &str, note: &str) {
    egui::Frame::new().inner_margin(egui::Margin::symmetric(16, 10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).strong().size(13.5).color(TX));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(note).size(11.5).color(MUTE)));
        });
    });
    rule(ui);
}
/// Full-width hairline.
fn rule(ui: &mut egui::Ui) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(r, 0.0, LINE);
}
/// Pane title: small uppercase label over a hairline.
fn panel_header(ui: &mut egui::Ui, t: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(t.to_uppercase()).size(10.5).strong().color(MUTE));
    ui.add_space(6.0);
    rule(ui);
    ui.add_space(6.0);
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

    /// Labelled numeric input with the unit set apart in a fixed-width column.
    fn field(ui: &mut egui::Ui, label: &str, unit: &str, v: &mut f64, ch: &mut bool) {
        ui.add_space(2.0);
        ui.label(RichText::new(label).size(11.5).color(MUTE));
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 52.0).max(60.0);
            *ch |= ui.add_sized([w, 26.0], egui::DragValue::new(v).speed(1.0).max_decimals(3)).changed();
            ui.label(RichText::new(unit).font(FontId::monospace(11.0)).color(MUTE));
        });
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

    fn draw_section(ui: &mut egui::Ui, d: &Drawing) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(220.0, 300.0), egui::Sense::hover());
        let p = ui.painter_at(rect);
        let lab =
            |p: &egui::Painter, pos: egui::Pos2, t: String| p.text(pos, egui::Align2::CENTER_CENTER, t, FontId::monospace(11.0), BLUE);
        match d {
            Drawing::Beam { b, h, cover, link_dia, bar_dia, n_bars } => {
                let sc = (150.0 / b).min(250.0 / h) as f32;
                let (w, hh) = (*b as f32 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                p.rect_stroke(
                    outer.shrink(((cover + link_dia / 2.0) as f32) * sc),
                    0.0,
                    Stroke::new(1.2_f32, Color32::from_rgb(51, 65, 85)),
                    StrokeKind::Inside,
                );
                let ed = ((cover + link_dia + bar_dia / 2.0) as f32) * sc;
                let n = (*n_bars).max(2);
                let br = ((*bar_dia as f32 * sc) / 2.0).max(2.4);
                for k in 0..n {
                    p.circle_filled(egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32 / (n - 1) as f32, o.y + hh - ed), br, TX);
                }
                for k in 0..2 {
                    p.circle_filled(
                        egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32, o.y + ed),
                        (6.0 * sc).max(2.2),
                        Color32::from_rgb(100, 116, 139),
                    );
                }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("b = {b}"));
                lab(&p, egui::pos2(o.x - 12.0, rect.center().y - 8.0), format!("h = {h}"));
            }
            Drawing::Column { b, h, cover, link_dia, bar_dia, n_face } => {
                let sc = (170.0 / b).min(250.0 / h) as f32;
                let (w, hh) = (*b as f32 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                p.rect_stroke(
                    outer.shrink(((cover + link_dia / 2.0) as f32) * sc),
                    0.0,
                    Stroke::new(1.2_f32, Color32::from_rgb(51, 65, 85)),
                    StrokeKind::Inside,
                );
                let ed = ((cover + link_dia + bar_dia / 2.0) as f32) * sc;
                let n = (*n_face).max(2);
                let br = ((*bar_dia as f32 * sc) / 2.0).max(2.4);
                for y in [o.y + ed, o.y + hh - ed] {
                    for k in 0..n {
                        p.circle_filled(egui::pos2(o.x + ed + (w - 2.0 * ed) * k as f32 / (n - 1) as f32, y), br, TX);
                    }
                }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{b} × {h}"));
            }
            Drawing::Bolts { w, n1, n2, e1, p1, e2: _, p2, d } => {
                let len = 2.0 * e1 + (*n1 as f64 - 1.0) * p1;
                let sc = (200.0 / len.max(*w)) as f32;
                let (ww, hh) = (len as f32 * sc, *w as f32 * sc);
                let o = egui::pos2(rect.center().x - ww / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(ww, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                let y0 = o.y + (((*w - (*n2 as f64 - 1.0) * p2) / 2.0) as f32) * sc;
                for i in 0..*n1 {
                    for j in 0..*n2 {
                        let c = egui::pos2(o.x + ((e1 + i as f64 * p1) as f32) * sc, y0 + (j as f64 * p2) as f32 * sc);
                        p.circle_filled(c, ((*d as f32 * sc) / 2.0).max(3.0), Color32::WHITE);
                        p.circle_stroke(c, ((*d as f32 * sc) / 2.0).max(3.0), Stroke::new(1.6_f32, TX));
                    }
                }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{n1} × {n2} bolts, plate {w} wide"));
            }
            Drawing::Weld { lw, a } => {
                let c = rect.center();
                let steel = Color32::from_rgb(51, 65, 85);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(c.x - 90.0, c.y - 20.0), egui::vec2(180.0, 16.0)), 0.0, steel);
                p.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(c.x - 90.0, c.y - 4.0), egui::vec2(180.0, 16.0)),
                    0.0,
                    Color32::from_rgb(148, 163, 184),
                );
                p.add(egui::Shape::convex_polygon(
                    vec![egui::pos2(c.x - 90.0, c.y + 12.0), egui::pos2(c.x - 90.0, c.y + 25.0), egui::pos2(c.x - 77.0, c.y + 12.0)],
                    TX,
                    Stroke::NONE,
                ));
                lab(&p, egui::pos2(c.x, c.y + 50.0), format!("a = {a} mm, L = {lw} mm"));
            }
            Drawing::Raft { l, w, n, spacing, edge } => {
                let sc = (200.0 / l.max(*w)) as f32;
                let (ww, hh) = (*l as f32 * sc, *w as f32 * sc);
                let o = egui::pos2(rect.center().x - ww / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(ww, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                for i in 0..*n {
                    p.rect_filled(
                        egui::Rect::from_center_size(
                            egui::pos2(o.x + ((edge + i as f64 * spacing) as f32) * sc, o.y + hh / 2.0),
                            egui::vec2(10.0, 10.0),
                        ),
                        0.0,
                        Color32::from_rgb(51, 65, 85),
                    );
                }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("L = {:.0} / w = {w} (plan)", l));
            }
            Drawing::Retain { hs, tb, ts, lt, b } => {
                let sc = (190.0 / (hs + tb).max(*b)) as f32;
                let (w, hh) = (*b as f32 * sc, (*hs + *tb) as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let steel = Color32::from_rgb(51, 65, 85);
                let (lts, tss, hss, tbs) = (*lt as f32 * sc, *ts as f32 * sc, *hs as f32 * sc, *tb as f32 * sc);
                p.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(o.x + lts + tss, o.y), egui::vec2(w - lts - tss, hss)),
                    0.0,
                    Color32::from_rgb(226, 232, 240),
                );
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x, o.y + hss), egui::vec2(w, tbs)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x + lts, o.y), egui::vec2(tss, hss)), 0.0, steel);
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("H = {} / B = {b}", hs + tb));
            }
            Drawing::Rect { b, h } => {
                let sc = (150.0 / b).min(250.0 / h) as f32;
                let (w, hh) = (*b as f32 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::from_rgb(232, 216, 189));
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{b} × {h}"));
            }
            Drawing::ISection { h, b, tw, tf } => {
                let sc = (160.0 / b).min(250.0 / h) as f32;
                let (w, hh, wt, ft) = (*b as f32 * sc, *h as f32 * sc, *tw as f32 * sc, *tf as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let steel = Color32::from_rgb(51, 65, 85);
                p.rect_filled(egui::Rect::from_min_size(o, egui::vec2(w, ft)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x, o.y + hh - ft), egui::vec2(w, ft)), 0.0, steel);
                p.rect_filled(egui::Rect::from_min_size(egui::pos2(o.x + (w - wt) / 2.0, o.y), egui::vec2(wt, hh)), 0.0, steel);
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{h} × {b} × {tw} / {tf}"));
            }
            Drawing::Wall { h, cover, bar_dia, spacing } => {
                let sc = (120.0 / h).min(1.0) as f32;
                let (w, hh) = (*h as f32 * sc, 260.0f32);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                let mut y = *spacing as f32 / 2.0;
                while y < 400.0 {
                    for xx in [o.x + ((cover + 10.0) as f32) * sc, o.x + w - ((cover + 10.0) as f32) * sc] {
                        p.circle_filled(egui::pos2(xx, o.y + y / 400.0 * hh), ((*bar_dia as f32 * sc) / 2.0).max(2.2), TX);
                    }
                    y += *spacing as f32;
                }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("h = {h}, T{bar_dia} @ {spacing}"));
            }
            Drawing::Strip { width, t, h } => {
                let sc = (190.0 / width) as f32;
                let (w, hh, tw) = (*width as f32 * sc, *h as f32 * sc, *t as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                p.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(o.x + (w - tw) / 2.0, o.y - 50.0), egui::vec2(tw, 50.0)),
                    0.0,
                    Color32::from_rgb(51, 65, 85),
                );
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                p.line_segment(
                    [egui::pos2(o.x + 8.0, o.y + hh - 8.0), egui::pos2(o.x + w - 8.0, o.y + hh - 8.0)],
                    Stroke::new(2.5_f32, TX),
                );
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("B = {width}, h = {h}"));
            }
            Drawing::Slab { h, cover, bar_dia, spacing } => {
                let sc = 0.5f32;
                let (w, hh) = (400.0 * sc, *h as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                let mut x = *spacing as f32 / 2.0;
                while x < 400.0 {
                    p.circle_filled(
                        egui::pos2(o.x + x * sc, o.y + hh - ((cover + bar_dia / 2.0) as f32) * sc),
                        ((*bar_dia as f32 * sc) / 2.0).max(2.2),
                        TX,
                    );
                    x += *spacing as f32;
                }
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("T{} @ {}", bar_dia, spacing));
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 32.0), format!("h = {h} (1 m strip)"));
            }
            Drawing::Pad { bx, by, cx, cy, h } => {
                let sc = (170.0 / bx.max(*by)) as f32;
                let (w, hh) = (*bx as f32 * sc, *by as f32 * sc);
                let o = egui::pos2(rect.center().x - w / 2.0, rect.center().y - hh / 2.0 - 8.0);
                let outer = egui::Rect::from_min_size(o, egui::vec2(w, hh));
                p.rect_filled(outer, 0.0, Color32::WHITE);
                p.rect_stroke(outer, 0.0, Stroke::new(2.0_f32, TX), StrokeKind::Inside);
                for i in 1..7 {
                    let y = o.y + hh * i as f32 / 7.0;
                    p.line_segment(
                        [egui::pos2(o.x + 6.0, y), egui::pos2(o.x + w - 6.0, y)],
                        Stroke::new(1.0_f32, Color32::from_rgb(148, 163, 184)),
                    );
                }
                p.rect_filled(
                    egui::Rect::from_center_size(outer.center(), egui::vec2(*cx as f32 * sc, *cy as f32 * sc)),
                    0.0,
                    Color32::from_rgb(51, 65, 85),
                );
                lab(&p, egui::pos2(rect.center().x, o.y + hh + 16.0), format!("{bx} × {by}, h = {h}"));
            }
        }
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
                egui::SidePanel::left("in")
                    .exact_width(300.0)
                    .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::same(16)))
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            panel_header(ui, if self.st[self.mi].mode == "check" { "Inputs · Check" } else { "Inputs · Design" });
                            let stt = &mut self.st[self.mi];
                            let mut last = "";
                            for (f, v) in self.defs[self.mi].fields.iter().zip(stt.vals.iter_mut()) {
                                if f.check_only && stt.mode != "check" {
                                    continue;
                                }
                                if f.group != last {
                                    heading(ui, f.group);
                                    last = f.group;
                                }
                                if let Some(opts) = &f.options {
                                    ui.label(RichText::new(f.label).size(11.5).color(MUTE));
                                    let cur = opts.iter().find(|o| (o.0 - *v).abs() < 1e-9).map(|o| o.1).unwrap_or("-");
                                    egui::ComboBox::from_id_salt(format!("opt_{}", f.key))
                                        .width(ui.available_width() - 8.0)
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
                                    Self::field(ui, f.label, f.unit, v, &mut ch);
                                }
                            }
                        });
                    });
                if ch {
                    self.run();
                }

                egui::SidePanel::right("out")
                    .exact_width(340.0)
                    .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, LINE)).inner_margin(egui::Margin::same(16)))
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            panel_header(ui, "Verification");
                            if let Some(r) = &self.result {
                                if r.ok {
                                    let (t, c, top) = if r.checks.is_empty() {
                                        ("RESULTS", TX, MUTE)
                                    } else if r.overall_pass {
                                        ("PASS", GREEN, GREEN)
                                    } else {
                                        ("FAIL", RED, RED)
                                    };
                                    let resp = egui::Frame::new()
                                        .fill(Color32::WHITE)
                                        .stroke(Stroke::new(1.0_f32, LINE))
                                        .corner_radius(CornerRadius::ZERO)
                                        .inner_margin(egui::Margin { left: 14, right: 14, top: 18, bottom: 14 })
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(t).font(FontId::monospace(30.0)).strong().color(c));
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(
                                                        RichText::new(if r.checks.is_empty() {
                                                            String::new()
                                                        } else {
                                                            format!("max {:.0}%", r.max_util * 100.0)
                                                        })
                                                        .font(FontId::monospace(12.0))
                                                        .color(MUTE),
                                                    )
                                                });
                                            });
                                            ui.label(
                                                RichText::new(if r.checks.is_empty() {
                                                    "Combined action effects. Design values only, no PASS/FAIL verdict.".to_string()
                                                } else {
                                                    format!(
                                                        "{}/{} checks satisfied. Implemented checks only; not an engineer's approval.",
                                                        r.checks.iter().filter(|k| k.pass).count(),
                                                        r.checks.len()
                                                    )
                                                })
                                                .size(11.5)
                                                .color(MUTE),
                                            );
                                        });
                                    let rr = resp.response.rect;
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_max(rr.left_top(), egui::pos2(rr.right(), rr.top() + 4.0)),
                                        0.0,
                                        top,
                                    );
                                    if !r.checks.is_empty() {
                                        heading(ui, "Utilisation  Ed / Rd");
                                    }
                                    for k in &r.checks {
                                        let col = if !k.pass {
                                            RED
                                        } else if k.utilisation > 0.85 {
                                            AMBER
                                        } else {
                                            GREEN
                                        };
                                        ui.horizontal(|ui| {
                                            ui.label(&k.name);
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(
                                                    RichText::new(if k.utilisation >= 9.99 {
                                                        "n/a".to_string()
                                                    } else {
                                                        format!("{:.2}", k.utilisation)
                                                    })
                                                    .font(FontId::monospace(13.0))
                                                    .strong()
                                                    .color(col),
                                                )
                                            });
                                        });
                                        ui.label(
                                            RichText::new(format!("{} vs {}", k.demand, k.capacity))
                                                .font(FontId::monospace(10.5))
                                                .color(MUTE),
                                        );
                                        let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
                                        ui.painter().rect_filled(bar, 0.0, HEAD);
                                        ui.painter().rect_filled(
                                            egui::Rect::from_min_size(
                                                bar.min,
                                                egui::vec2(bar.width() * k.utilisation.clamp(0.0, 1.0) as f32, 4.0),
                                            ),
                                            0.0,
                                            col,
                                        );
                                        ui.label(RichText::new(&k.reference).font(FontId::monospace(10.0)).color(MUTE));
                                        ui.add_space(6.0);
                                        rule(ui);
                                        ui.add_space(4.0);
                                    }
                                    heading(ui, &format!("Observations  {}", r.observations.len()));
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
                                        ui.add_space(6.0);
                                        rule(ui);
                                        ui.add_space(4.0);
                                    }
                                    if !self.saved.is_empty() {
                                        ui.add_space(8.0);
                                        ui.label(RichText::new(&self.saved).size(11.5).color(GREEN));
                                    }
                                }
                            }
                        });
                    });

                egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::symmetric(32, 24))).show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(RichText::new(if self.st[self.mi].mode == "check" { "INDEPENDENT CHECK" } else { "DESIGN" }).size(10.5).strong().color(MUTE));
                        ui.label(RichText::new(self.defs[self.mi].title).size(24.0).strong().color(TX));
                        ui.label(RichText::new(self.defs[self.mi].subtitle).font(FontId::monospace(11.5)).color(MUTE));
                        ui.add_space(12.0);
                        let (hr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                        ui.painter().rect_filled(hr, 0.0, TX);
                        ui.add_space(16.0);
                        if let Some(r) = &self.result {
                            for e in &r.errors { ui.colored_label(RED, e); }
                            if r.ok {
                                if r.param_status.contains("NOT") {
                                    egui::Frame::new().fill(AMBER_BG).corner_radius(CornerRadius::ZERO).inner_margin(egui::Margin::symmetric(14, 11)).show(ui, |ui| { ui.set_width(ui.available_width()); ui.label(RichText::new(format!("Parameter set: {}. Confirm the governing National Annex before issue for construction.", r.param_status)).size(12.5).color(TX)); });
                                    ui.add_space(10.0);
                                }
                                for w in &r.warnings { ui.colored_label(AMBER, w); }
                                card(ui, |ui| {
                                    card_head(ui, "Section", self.defs[self.mi].name);
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0); if let Some(d) = &r.drawing { Self::draw_section(ui, d); }
                                        ui.vertical(|ui| {
                                            ui.set_width(ui.available_width() - 12.0);
                                            for k in &r.summary {
                                                ui.horizontal(|ui| { ui.label(RichText::new(&k.label).color(MUTE)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(&k.value).font(FontId::monospace(13.0)).strong())); });
                                                ui.separator();
                                            }
                                        });
                                    });
                                });
                                ui.add_space(14.0);
                                card(ui, |ui| {
                                    card_head(ui, "Calculation", &format!("{} steps · deterministic", r.trace.steps.len()));
                                    egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                                        let calc_w = (ui.available_width() - 40.0 - 150.0 - 170.0 - 3.0 * 16.0).max(220.0);
                                        egui::Grid::new("calc").striped(false).num_columns(4).spacing([16.0, 12.0]).min_col_width(40.0).show(ui, |ui| {
                                            for h in ["#", "REFERENCE", "CALCULATION", "OUTPUT"] { ui.label(RichText::new(h).size(10.5).strong().color(MUTE)); }
                                            ui.end_row();
                                            let (mut cur, mut n) = (String::new(), 0);
                                            for s in &r.trace.steps {
                                                if s.section != cur { cur = s.section.clone(); ui.label(""); ui.label(RichText::new(&s.section).strong().color(TX)); ui.label(""); ui.label(""); ui.end_row(); }
                                                n += 1;
                                                ui.label(RichText::new(format!("{:02}", n)).font(FontId::monospace(11.0)).color(MUTE));
                                                ui.allocate_ui_with_layout(egui::vec2(150.0, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| { ui.set_width(150.0); ui.add(egui::Label::new(RichText::new(if s.reference.is_empty() { "—" } else { &s.reference }).font(FontId::monospace(11.0)).color(BLUE)).wrap()); });
                                                ui.vertical(|ui| { ui.set_width(calc_w); ui.label(RichText::new(&s.title).strong().size(12.5)); ui.add(egui::Label::new(RichText::new(&s.calculation).font(FontId::monospace(12.0))).wrap()); if !s.comment.is_empty() { ui.label(RichText::new(&s.comment).size(11.0).color(MUTE)); } });
                                                ui.allocate_ui_with_layout(egui::vec2(170.0, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| { ui.set_width(170.0); ui.add(egui::Label::new(RichText::new(&s.output).font(FontId::monospace(12.0)).strong().color(match s.output.as_str() { "PASS" => GREEN, "FAIL" => RED, _ => TX })).wrap()); });
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
