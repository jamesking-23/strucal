#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod account_page;
mod assistant_page;
mod canvas;
mod cloud;
mod loading;
mod presets;
mod project_page;
mod secure_store;
mod theme;
mod update;
mod viewport;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, StrokeKind};
use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use strucal_engine::{
    module::{ModDef, ModResult},
    modules, params, registry, report, trace,
};
pub(crate) use theme::*;

#[derive(PartialEq, Clone, Copy)]
enum Page {
    Workspace,
    Project,
    Account,
    Standards,
    Settings,
}
impl Page {
    fn id(self) -> &'static str {
        match self {
            Page::Workspace => "workbench",
            Page::Project => "project",
            Page::Account => "account",
            Page::Standards => "standards",
            Page::Settings => "settings",
        }
    }
    fn from_id(s: &str) -> Page {
        match s {
            "project" => Page::Project,
            "account" => Page::Account,
            "standards" => Page::Standards,
            "settings" => Page::Settings,
            _ => Page::Workspace,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Page::Workspace => "Workbench",
            Page::Project => "Project",
            Page::Account => "Account",
            Page::Standards => "Standards",
            Page::Settings => "Settings",
        }
    }
}
enum Upd {
    Idle,
    Checking,
    UpToDate,
    Available(update::Available),
    Installing,
    Failed(String),
}

/// Launch-time session state shown in the status bar.
#[derive(Clone, PartialEq)]
enum SessionUi {
    /// Validating a stored session in the background.
    Restoring,
    SignedOut,
    SignedIn,
    /// A stored session exists but the server could not be reached; retried periodically.
    Offline(String),
    /// The stored session was rejected and removed.
    Expired,
}

#[derive(PartialEq, Clone, Copy)]
enum VpTab {
    Section,
    Loading,
}

#[derive(PartialEq, Clone, Copy)]
enum ExportKind {
    Html,
    Doc,
    Print,
}

#[derive(PartialEq, Clone, Copy)]
enum LedgerTab {
    Ledger,
    Checks,
    Summary,
    Scope,
}

struct ModState {
    mode: String,
    param: String,
    vals: Vec<f64>,
    /// Text as typed in each numeric input; committed to `vals` only when it parses.
    text: Vec<String>,
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
    ledger: LedgerTab,
    palette: bool,
    pq: String,
    session: SessionUi,
    srx: Option<Receiver<cloud::Restore>>,
    next_retry: Option<Instant>,
    prefs_dirty: bool,
    vp_tab: VpTab,
    vp_views: [canvas::View; 2],
    /// One-frame override for every ledger section (Expand all / Collapse all).
    ledger_force: Option<bool>,
    /// Verification rail order: by utilisation (true) or as on the calculation sheet.
    eta_order: bool,
    show_insp: bool,
    show_rail: bool,
    confirm_reset: bool,
}

// ---------------------------------------------------------------- primitives shared by every page

/// Bordered surface block (1px hairline, square corners).
fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())).corner_radius(CornerRadius::ZERO).show(ui, add).inner
}
/// Full-width hairline.
fn rule(ui: &mut egui::Ui) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, line());
}
/// Toolbar button; `on` renders the selected (ink) state of a segmented control.
fn tbtn(ui: &mut egui::Ui, t: &str, on: bool) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(t).font(medium(12.5)).color(if on { on_ink() } else { tx() }))
            .fill(if on { tx() } else { surf() })
            .stroke(Stroke::new(1.0_f32, if on { tx() } else { p().line_strong }))
            .corner_radius(CornerRadius::same(6))
            .min_size(egui::vec2(0.0, 28.0)),
    )
}
/// Primary (ink) action button.
fn primary(ui: &mut egui::Ui, t: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(t).font(strong(12.5)).color(on_ink()))
            .fill(tx())
            .stroke(Stroke::new(1.0_f32, tx()))
            .corner_radius(CornerRadius::same(6))
            .min_size(egui::vec2(0.0, 28.0)),
    )
}
/// Layout toggle chip: quiet surface when off, raised tinted surface when on.
fn chip(ui: &mut egui::Ui, t: &str, on: bool) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(t).font(medium(12.0)).color(if on { tx() } else { mute() }))
            .fill(if on { head() } else { surf() })
            .stroke(Stroke::new(1.0_f32, if on { p().line_strong } else { line() }))
            .corner_radius(CornerRadius::same(6))
            .min_size(egui::vec2(0.0, 28.0)),
    )
}
/// Ghost button: text only, hairline on hover.
fn ghost(ui: &mut egui::Ui, t: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(t).font(medium(12.5)).color(tx())).frame(false).min_size(egui::vec2(0.0, 28.0)))
}
/// Section header strip (SectionHeader): uppercase title and a muted monospace meta value.
fn pane_header(ui: &mut egui::Ui, title: &str, meta: &str) {
    egui::Frame::new().fill(head()).inner_margin(egui::Margin::symmetric(16, 0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(34.0);
        ui.horizontal_centered(|ui| {
            ui.label(RichText::new(title.to_uppercase()).font(strong(10.5)).extra_letter_spacing(0.9).color(p().ink2));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(meta).font(mono(10.5)).color(mute()))
            });
        });
    });
    rule(ui);
}
/// Lettered calculation-sheet heading with a rule below.
fn sheet_heading(ui: &mut egui::Ui, letter: &str, title: &str) {
    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(letter).font(mono(12.0)).color(p().ink4));
        ui.label(RichText::new(title).font(strong(14.0)).color(tx()));
    });
    ui.add_space(2.0);
    rule(ui);
    ui.add_space(8.0);
}
/// Utilisation as shown to engineers; the engine saturates unbounded ratios at 9.99.
fn util_text(u: f64) -> String {
    if u >= 9.99 {
        "n/a".into()
    } else {
        format!("{u:.2}")
    }
}
/// Horizontal utilisation bar with the 0.85 review tick and the 1.00 limit.
fn util_bar(ui: &mut egui::Ui, u: f64, col: Color32, h: f32) {
    let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), Sense::hover());
    ui.painter().rect_filled(bar, 0.0, p().track);
    let fill = bar.width() * (u / 1.0).clamp(0.0, 1.0) as f32;
    ui.painter().rect_filled(egui::Rect::from_min_size(bar.min, egui::vec2(fill, h)), 0.0, col);
    for (x, c) in [(REVIEW as f32, p().ink4), (1.0, p().ink3)] {
        let tx_ = bar.left() + bar.width() * x - 0.5;
        ui.painter().line_segment([egui::pos2(tx_, bar.top() - 3.0), egui::pos2(tx_, bar.bottom() + 3.0)], Stroke::new(1.0_f32, c));
    }
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
    ui.label(RichText::new(t.to_uppercase()).font(strong(10.5)).extra_letter_spacing(0.9).color(mute()));
    ui.add_space(4.0);
    rule(ui);
    ui.add_space(6.0);
}
/// Status badge: monospace uppercase label in a 1px border, tinted by status.
fn badge(ui: &mut egui::Ui, t: &str, fg: Color32, bg: Color32) -> egui::Response {
    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, fg))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| ui.label(RichText::new(t.to_uppercase()).font(mono_strong(10.0)).color(fg)))
        .response
}
/// Notice strip with a 2px tone edge.
fn notice(ui: &mut egui::Ui, fg: Color32, bg: Color32, add: impl FnOnce(&mut egui::Ui)) {
    let r = egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin { left: 14, right: 12, top: 9, bottom: 9 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        });
    let rect = r.response.rect;
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
        CornerRadius { nw: 6, sw: 6, ne: 0, se: 0 },
        fg,
    );
}
/// Numbers as typed: trims, accepts a decimal comma, rejects empty and non-finite values (apps/web/lib/form.ts parseNumber).
fn parse_number(s: &str) -> Option<f64> {
    let t = s.trim().replacen(',', ".", 1);
    if t.is_empty() || t == "-" || t == "." {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}
/// Shortest faithful text for a value.
fn fmt_num(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('e') {
        format!("{v:.6}")
    } else {
        s
    }
}
fn discipline_of(id: &str) -> &'static str {
    match id {
        "combos" => "Actions",
        "steelbeam" | "steelcol" | "beamcol" | "steeltension" | "bolted" | "weld" => "Structural steel",
        "timber" | "timbercol" | "timbertension" => "Timber",
        "masonry" => "Masonry",
        "pad" | "strip" | "raft" | "retaining" => "Foundations & geotechnics",
        _ => "Reinforced concrete",
    }
}
const DISCIPLINES: [&str; 6] = ["Actions", "Reinforced concrete", "Structural steel", "Timber", "Masonry", "Foundations & geotechnics"];
fn standard_of(d: &ModDef) -> &str {
    d.subtitle.split('·').next().unwrap_or("").trim()
}

/// Rail icons, drawn as 1.4px line glyphs in an 18px box.
fn paint_icon(pt: &egui::Painter, c: egui::Pos2, page: Page, col: Color32) {
    let s = Stroke::new(1.4_f32, col);
    let r = |x0: f32, y0: f32, x1: f32, y1: f32| egui::Rect::from_min_max(c + egui::vec2(x0, y0), c + egui::vec2(x1, y1));
    match page {
        Page::Workspace => {
            pt.rect_stroke(r(-8.0, -7.0, 8.0, 7.0), 0.0, s, StrokeKind::Middle);
            pt.line_segment([c + egui::vec2(-3.0, -7.0), c + egui::vec2(-3.0, 7.0)], s);
            pt.line_segment([c + egui::vec2(-3.0, 1.0), c + egui::vec2(8.0, 1.0)], s);
        }
        Page::Project => {
            pt.rect_stroke(r(-8.0, -7.0, -2.0, -2.0), 0.0, s, StrokeKind::Middle);
            pt.rect_stroke(r(2.0, 2.0, 8.0, 7.0), 0.0, s, StrokeKind::Middle);
            pt.line_segment([c + egui::vec2(-5.0, -2.0), c + egui::vec2(-5.0, 4.5)], s);
            pt.line_segment([c + egui::vec2(-5.0, 4.5), c + egui::vec2(2.0, 4.5)], s);
        }
        Page::Standards => {
            pt.rect_stroke(r(-6.0, -8.0, 6.0, 8.0), 0.0, s, StrokeKind::Middle);
            for y in [-4.0, -0.5, 3.0] {
                pt.line_segment([c + egui::vec2(-3.0, y), c + egui::vec2(3.0, y)], s);
            }
        }
        Page::Account => {
            pt.circle_stroke(c + egui::vec2(0.0, -3.5), 3.5, s);
            pt.line_segment([c + egui::vec2(-7.0, 8.0), c + egui::vec2(-5.0, 2.5)], s);
            pt.line_segment([c + egui::vec2(-5.0, 2.5), c + egui::vec2(5.0, 2.5)], s);
            pt.line_segment([c + egui::vec2(5.0, 2.5), c + egui::vec2(7.0, 8.0)], s);
        }
        Page::Settings => {
            pt.circle_stroke(c, 3.0, s);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                let (sn, cs) = a.sin_cos();
                pt.line_segment([c + egui::vec2(cs * 5.5, sn * 5.5), c + egui::vec2(cs * 8.0, sn * 8.0)], s);
            }
        }
    }
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        let prefs = secure_store::load_prefs("desktop.json").unwrap_or(Value::Null);
        theme::apply(&cc.egui_ctx, Theme::from_id(prefs["theme"].as_str().unwrap_or("drafting")));
        update::cleanup_old();
        let defs = modules::defs();
        let ps_ids: Vec<String> = params::available().into_iter().map(|p| p.id).collect();
        let st: Vec<ModState> = defs
            .iter()
            .map(|d| {
                let saved = &prefs["modules"][d.id];
                let vals: Vec<f64> =
                    d.fields.iter().map(|f| saved["vals"][f.key].as_f64().filter(|v| v.is_finite()).unwrap_or(f.default)).collect();
                let mode = if saved["mode"] == "check" && d.has_check_mode { "check" } else { "design" };
                let param = saved["param"].as_str().filter(|p| ps_ids.iter().any(|x| x == p)).unwrap_or("EN1992-1-1:2004/RECOMMENDED");
                ModState { mode: mode.into(), param: param.into(), text: vals.iter().map(|v| fmt_num(*v)).collect(), vals }
            })
            .collect();
        let mi = prefs["module"].as_str().or(Some("beam")).and_then(|m| defs.iter().position(|d| d.id == m)).unwrap_or(0);
        let page = Page::from_id(prefs["page"].as_str().unwrap_or("workbench"));

        // Restore a stored session in the background: the window opens immediately and never blocks on the network.
        let cl = cloud::Client::default();
        let (session, srx) = if cl.load_stored() {
            let (tx_, rx_) = channel();
            let c = cl.clone();
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let _ = tx_.send(c.restore());
                ctx.request_repaint();
            });
            (SessionUi::Restoring, Some(rx_))
        } else {
            (SessionUi::SignedOut, None)
        };

        let mut a = App {
            page,
            defs,
            mi,
            st,
            result: None,
            saved: String::new(),
            upd: Upd::Idle,
            rx: None,
            irx: None,
            pj: project_page::Pj::new(),
            cl,
            acct: account_page::Acct::new(),
            acct_job: account_page::AcctJob::default(),
            asst: assistant_page::Asst::default(),
            dims: prefs["dims"].as_bool().unwrap_or(true),
            ledger: LedgerTab::Ledger,
            palette: false,
            pq: String::new(),
            session,
            srx,
            next_retry: None,
            prefs_dirty: false,
            vp_tab: VpTab::Section,
            vp_views: [canvas::View::default(); 2],
            ledger_force: None,
            eta_order: true,
            show_insp: prefs["show_insp"].as_bool().unwrap_or(true),
            show_rail: prefs["show_rail"].as_bool().unwrap_or(true),
            confirm_reset: false,
        };
        a.run();
        a
    }

    fn json(&self) -> Value {
        let (d, s) = (&self.defs[self.mi], &self.st[self.mi]);
        let mut m = serde_json::Map::new();
        m.insert("mode".into(), s.mode.clone().into());
        m.insert("param_set".into(), s.param.clone().into());
        for (f, v) in d.fields.iter().zip(&s.vals) {
            m.insert(f.key.into(), (*v).into());
        }
        Value::Object(m)
    }
    fn run(&mut self) {
        let id = self.defs[self.mi].id;
        self.result = Some(modules::run(id, &self.json()));
        self.prefs_dirty = true;
    }
    /// Rewrites the typed text of every input of a module from its committed values.
    fn sync_text(&mut self, mi: usize) {
        let s = &mut self.st[mi];
        s.text = s.vals.iter().map(|v| fmt_num(*v)).collect();
    }
    /// Inputs of the current module whose typed text is not a number.
    fn input_errors(&self) -> Vec<(&'static str, &'static str)> {
        let (d, s) = (&self.defs[self.mi], &self.st[self.mi]);
        d.fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.options.is_none() && (!f.check_only || s.mode == "check"))
            .filter_map(|(i, f)| match parse_number(&s.text[i]) {
                Some(_) => None,
                None => Some((f.label, if s.text[i].trim().is_empty() { "Required" } else { "Not a number" })),
            })
            .collect()
    }
    fn select_module(&mut self, i: usize) {
        if i != self.mi {
            self.mi = i;
            self.saved.clear();
            self.vp_views = [canvas::View::default(); 2];
            self.run();
            // Modules without a cross-section open on their loading diagram.
            if self.result.as_ref().is_some_and(|r| r.ok && r.drawing.is_none()) {
                self.vp_tab = VpTab::Loading;
            }
        }
    }
    /// Engine validation messages assigned to the field they name (by label or key); the rest stay general.
    fn split_engine_errors(&self) -> (std::collections::HashMap<&'static str, String>, Vec<String>) {
        let mut by = std::collections::HashMap::new();
        let mut general = vec![];
        let Some(r) = self.result.as_ref().filter(|r| !r.ok) else { return (by, general) };
        let mut fields: Vec<&strucal_engine::module::Field> = self.defs[self.mi].fields.iter().collect();
        fields.sort_by(|a, b| b.label.len().cmp(&a.label.len()));
        for e in &r.errors {
            match fields.iter().find(|f| e.starts_with(&format!("{} ", f.label)) || e.starts_with(&format!("{} ", f.key))) {
                Some(f) if !by.contains_key(f.key) => {
                    by.insert(f.key, e.clone());
                }
                _ => general.push(e.clone()),
            }
        }
        (by, general)
    }
    fn set_theme(&mut self, ctx: &egui::Context, t: Theme) {
        theme::apply(ctx, t);
        self.prefs_dirty = true;
    }
    fn save_prefs(&mut self) {
        let modules: serde_json::Map<String, Value> = self
            .defs
            .iter()
            .zip(&self.st)
            .map(|(d, s)| {
                let vals: serde_json::Map<String, Value> =
                    d.fields.iter().zip(&s.vals).map(|(f, v)| (f.key.to_string(), json!(v))).collect();
                (d.id.to_string(), json!({ "mode": s.mode, "param": s.param, "vals": vals }))
            })
            .collect();
        secure_store::save_prefs(
            "desktop.json",
            &json!({
                "theme": theme::current().id(),
                "page": self.page.id(),
                "module": self.defs[self.mi].id,
                "dims": self.dims,
                "show_insp": self.show_insp,
                "show_rail": self.show_rail,
                "modules": modules,
            }),
        );
        self.prefs_dirty = false;
    }

    fn export(&mut self, kind: ExportKind) {
        let Some(r) = &self.result else { return };
        if !r.ok || !self.input_errors().is_empty() {
            return;
        }
        let dir = user_dir(&["Downloads"]);
        let def = &self.defs[self.mi];
        let html = report::generic(def, &self.json(), r);
        let stem = format!("StruCal-{}-{}", def.id, &r.trace_hash[..8]);
        let (file, what) = match kind {
            ExportKind::Html => (dir.join(format!("{stem}.html")), "calculation sheet (.html)"),
            ExportKind::Doc => (dir.join(format!("{stem}.doc")), "Word document (.doc)"),
            ExportKind::Print => (dir.join(format!("{stem}-print.html")), "print sheet: use the browser's Print / Save as PDF"),
        };
        self.saved = match std::fs::write(&file, &html) {
            Ok(_) => {
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("cmd").args(["/C", "start", "", &file.to_string_lossy()]).spawn();
                }
                format!("Saved to Downloads: {} — {}", file.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default(), what)
            }
            Err(e) => format!("Could not write to the Downloads folder: {e}"),
        };
    }

    /// Polls the background session restore and schedules retries while offline.
    fn poll_session(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.srx {
            if let Ok(r) = rx.try_recv() {
                self.srx = None;
                self.session = match r {
                    cloud::Restore::Restored => SessionUi::SignedIn,
                    cloud::Restore::None => SessionUi::SignedOut,
                    cloud::Restore::Expired => SessionUi::Expired,
                    cloud::Restore::Offline(e) => {
                        self.next_retry = Some(Instant::now() + Duration::from_secs(30));
                        SessionUi::Offline(e)
                    }
                };
            }
        }
        if let (SessionUi::Offline(_), Some(at), None) = (&self.session, self.next_retry, &self.srx) {
            if Instant::now() >= at {
                let (tx_, rx_) = channel();
                let c = self.cl.clone();
                let ctx2 = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx_.send(c.restore());
                    ctx2.request_repaint();
                });
                self.srx = Some(rx_);
                self.next_retry = None;
            } else {
                ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
            }
        }
        // Sign-in and sign-out happen on the Account page; keep the shell's state in step with the client.
        if self.srx.is_none() {
            let live = self.cl.signed_in();
            match (&self.session, live) {
                (SessionUi::SignedIn, false) if !self.cl.has_pending_session() => self.session = SessionUi::SignedOut,
                (SessionUi::SignedOut | SessionUi::Expired | SessionUi::Offline(_), true) => {
                    self.session = SessionUi::SignedIn;
                    self.next_retry = None;
                }
                _ => {}
            }
        }
    }

    fn session_text(&self) -> (String, Color32) {
        match &self.session {
            SessionUi::Restoring => ("restoring session…".into(), mute()),
            SessionUi::SignedOut => ("signed out".into(), p().ink4),
            SessionUi::SignedIn => (format!("signed in · {}", self.cl.email()), green()),
            SessionUi::Offline(_) => ("offline · session kept, retrying".into(), amber()),
            SessionUi::Expired => ("session expired · sign in again".into(), red()),
        }
    }

    // ------------------------------------------------------------ shell

    fn rail(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("rail")
            .exact_width(56.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let (b, br) = ui.allocate_exact_size(egui::vec2(56.0, 44.0), Sense::click());
                let m = egui::Rect::from_center_size(b.center(), egui::vec2(26.0, 26.0));
                ui.painter().rect_filled(m, 7.0, tx());
                ui.painter().text(m.center(), egui::Align2::CENTER_CENTER, "S", strong(15.0), on_ink());
                ui.painter().line_segment([b.left_bottom(), b.right_bottom()], Stroke::new(1.0_f32, line()));
                if br.on_hover_text("StruCal · open command palette (Ctrl+K)").clicked() {
                    self.palette = true;
                }
                ui.add_space(6.0);
                for pg in [Page::Workspace, Page::Project, Page::Standards, Page::Account] {
                    self.rail_item(ui, pg);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(56.0, 40.0), Sense::click());
                    let signed = self.session == SessionUi::SignedIn;
                    let av = egui::Rect::from_center_size(r.center(), egui::vec2(26.0, 26.0));
                    let initials: String = if signed {
                        let n = if self.cl.name().is_empty() { self.cl.email() } else { self.cl.name() };
                        n.split(|c: char| c.is_whitespace() || c == '.' || c == '@')
                            .filter(|w| !w.is_empty())
                            .take(2)
                            .filter_map(|w| w.chars().next())
                            .collect::<String>()
                            .to_uppercase()
                    } else {
                        "—".into()
                    };
                    ui.painter().rect_filled(av, 13.0, if signed { tx() } else { head() });
                    ui.painter().rect_stroke(
                        av,
                        0.0,
                        Stroke::new(1.0_f32, if signed { tx() } else { p().line_strong }),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        av.center(),
                        egui::Align2::CENTER_CENTER,
                        initials,
                        mono_strong(10.5),
                        if signed { on_ink() } else { mute() },
                    );
                    if resp.on_hover_text(self.session_text().0).clicked() {
                        self.page = Page::Account;
                        self.prefs_dirty = true;
                    }
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(56.0, 40.0), Sense::click());
                    let dark = theme::current() == Theme::Obsidian;
                    let col = if resp.hovered() { tx() } else { mute() };
                    if dark {
                        ui.painter().circle_filled(r.center(), 3.5, col);
                        for k in 0..8 {
                            let (sn, cs) = (k as f32 * std::f32::consts::FRAC_PI_4).sin_cos();
                            ui.painter().line_segment(
                                [r.center() + egui::vec2(cs * 6.0, sn * 6.0), r.center() + egui::vec2(cs * 7.5, sn * 7.5)],
                                Stroke::new(1.4_f32, col),
                            );
                        }
                    } else {
                        ui.painter().circle_stroke(r.center(), 6.5, Stroke::new(1.4_f32, col));
                        ui.painter().circle_filled(r.center() + egui::vec2(3.5, -2.5), 5.5, surf());
                    }
                    if resp.on_hover_text(if dark { "Drafting theme" } else { "Obsidian theme" }).clicked() {
                        let t = if dark { Theme::Drafting } else { Theme::Obsidian };
                        self.set_theme(ctx, t);
                    }
                    self.rail_item(ui, Page::Settings);
                });
            });
    }

    fn rail_item(&mut self, ui: &mut egui::Ui, pg: Page) {
        let (r, resp) = ui.allocate_exact_size(egui::vec2(56.0, 46.0), Sense::click());
        let on = self.page == pg;
        let pill = r.shrink2(egui::vec2(8.0, 3.0));
        if on {
            ui.painter().rect_filled(pill, 8.0, head());
            ui.painter().rect_stroke(pill, 8.0, Stroke::new(1.0_f32, line()), StrokeKind::Inside);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(r.left(), pill.top() + 6.0), egui::vec2(2.0, pill.height() - 12.0)),
                CornerRadius { nw: 0, sw: 0, ne: 2, se: 2 },
                tx(),
            );
        } else if resp.hovered() {
            ui.painter().rect_filled(pill, 8.0, head());
        }
        paint_icon(ui.painter(), r.center(), pg, if on || resp.hovered() { tx() } else { mute() });
        if resp.on_hover_text(pg.label()).clicked() {
            self.page = pg;
            self.prefs_dirty = true;
        }
    }

    fn command_bar(&mut self, ctx: &egui::Context) -> (Option<ExportKind>, bool) {
        let (mut export, mut reset) = (None, false);
        egui::TopBottomPanel::top("bar")
            .exact_height(44.0)
            .frame(egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())).inner_margin(egui::Margin::symmetric(16, 0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let d = &self.defs[self.mi];
                    let crumbs: Vec<String> = match self.page {
                        Page::Workspace => vec!["Workbench".into(), discipline_of(d.id).into(), d.name.into()],
                        p => vec![p.label().into()],
                    };
                    ui.label(RichText::new("StruCal").color(mute()));
                    for (i, c) in crumbs.iter().enumerate() {
                        ui.label(RichText::new("/").color(p().ink4));
                        let last = i + 1 == crumbs.len();
                        ui.label(RichText::new(c).font(if last { strong(13.0) } else { FontId::proportional(13.0) }).color(if last {
                            tx()
                        } else {
                            mute()
                        }));
                    }
                    ui.add_space(14.0);
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(260.0, 28.0), Sense::click());
                    ui.painter().rect_filled(r, 6.0, p().sunken);
                    ui.painter().rect_stroke(
                        r,
                        6.0,
                        Stroke::new(1.0_f32, if resp.hovered() { p().ink4 } else { line() }),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        r.left_center() + egui::vec2(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        "Modules, pages, actions",
                        FontId::proportional(12.5),
                        p().ink4,
                    );
                    ui.painter().text(r.right_center() - egui::vec2(10.0, 0.0), egui::Align2::RIGHT_CENTER, "Ctrl K", mono(10.5), mute());
                    if resp.clicked() {
                        self.palette = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.page == Page::Workspace {
                            let can = self.result.as_ref().is_some_and(|r| r.ok) && self.input_errors().is_empty();
                            ui.add_enabled_ui(can, |ui| {
                                let mb = egui::Button::new(RichText::new("Export…").font(strong(12.5)).color(on_ink()))
                                    .fill(tx())
                                    .stroke(Stroke::new(1.0_f32, tx()))
                                    .corner_radius(CornerRadius::same(6))
                                    .min_size(egui::vec2(0.0, 28.0));
                                egui::menu::menu_custom_button(ui, mb, |ui| {
                                    ui.set_min_width(260.0);
                                    for (k, t, h) in [
                                        (ExportKind::Html, "Calculation sheet", ".html · full working, A4 print layout"),
                                        (ExportKind::Doc, "Word document", ".doc · editable in Microsoft Word"),
                                        (ExportKind::Print, "Print / PDF", "opens the sheet in the browser to print"),
                                    ] {
                                        let resp = ui.add(
                                            egui::Button::new(RichText::new(format!("{t}\n")).font(strong(12.5)))
                                                .frame(false)
                                                .min_size(egui::vec2(250.0, 34.0)),
                                        );
                                        ui.painter().text(
                                            resp.rect.left_bottom() + egui::vec2(8.0, -7.0),
                                            egui::Align2::LEFT_BOTTOM,
                                            h,
                                            mono(10.0),
                                            mute(),
                                        );
                                        if resp.clicked() {
                                            export = Some(k);
                                            ui.close_menu();
                                        }
                                    }
                                });
                            });
                            if self
                                .assistant_button(ui)
                                .on_hover_text("StruCal Assistant · explains this calculation (needs a signed-in account)")
                                .clicked()
                            {
                                self.asst.open = !self.asst.open;
                            }
                            if chip(ui, "Verification ]", self.show_rail).on_hover_text("Show or hide the verification rail").clicked() {
                                self.show_rail = !self.show_rail;
                                self.prefs_dirty = true;
                            }
                            if chip(ui, "[ Inspector", self.show_insp).on_hover_text("Show or hide the inspector").clicked() {
                                self.show_insp = !self.show_insp;
                                self.prefs_dirty = true;
                            }
                            if ghost(ui, "Reset").on_hover_text("Reset this module's inputs to their defaults").clicked() {
                                reset = true;
                            }
                        } else {
                            ui.label(RichText::new(format!("v{}", update::CURRENT)).font(mono(11.0)).color(mute()));
                        }
                    });
                });
            });
        (export, reset)
    }

    /// StruCal Assistant button: rounded icon container whose edge carries a traveling light on hover and while the
    /// assistant is open. The light is painted under the icon and does not intercept the pointer.
    fn assistant_button(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (r, resp) = ui.allocate_exact_size(egui::vec2(112.0, 28.0), Sense::click());
        let active = self.asst.open;
        let lit = active || resp.hovered() || resp.has_focus();
        let pt = ui.painter();
        pt.rect_filled(r, 6.0, if resp.hovered() { head() } else { surf() });
        pt.rect_stroke(r, 6.0, Stroke::new(1.0_f32, if lit { p().line_strong } else { line() }), StrokeKind::Inside);
        let ico = egui::Rect::from_min_size(r.left_top() + egui::vec2(4.0, 3.0), egui::vec2(22.0, 22.0));
        let t = ui.input(|i| i.time) as f32;
        if lit {
            // soft halo
            for (k, a) in [(5.0_f32, 18u8), (3.0, 30), (1.5, 46)] {
                pt.rect_filled(ico.expand(k), 7.0 + k, blue().gamma_multiply(a as f32 / 255.0));
            }
        }
        pt.rect_filled(ico, 7.0, head());
        pt.rect_stroke(ico, 7.0, Stroke::new(1.0_f32, if lit { blue().gamma_multiply(0.35) } else { p().line_strong }), StrokeKind::Inside);
        if lit {
            // traveling light: a gradient tail running clockwise around the rounded container edge
            let rr = 7.0_f32;
            let e = ico.shrink(0.75);
            let (w, h) = (e.width() - 2.0 * rr, e.height() - 2.0 * rr);
            let per = 2.0 * (w + h) + 2.0 * std::f32::consts::PI * rr;
            let at = |d: f32| -> egui::Pos2 {
                let mut d = d.rem_euclid(per);
                let arc = std::f32::consts::FRAC_PI_2 * rr;
                let segs: [(f32, u8); 8] = [(w, 0), (arc, 1), (h, 2), (arc, 3), (w, 4), (arc, 5), (h, 6), (arc, 7)];
                for (len, k) in segs {
                    if d <= len {
                        let f = if len > 0.0 { d / len } else { 0.0 };
                        let c = |x: f32, y: f32| egui::pos2(x, y);
                        let ang = |a0: f32| a0 + f * std::f32::consts::FRAC_PI_2;
                        return match k {
                            0 => c(e.left() + rr + f * w, e.top()),
                            1 => {
                                let a = ang(-std::f32::consts::FRAC_PI_2);
                                c(e.right() - rr + rr * a.cos(), e.top() + rr + rr * a.sin())
                            }
                            2 => c(e.right(), e.top() + rr + f * h),
                            3 => {
                                let a = ang(0.0);
                                c(e.right() - rr + rr * a.cos(), e.bottom() - rr + rr * a.sin())
                            }
                            4 => c(e.right() - rr - f * w, e.bottom()),
                            5 => {
                                let a = ang(std::f32::consts::FRAC_PI_2);
                                c(e.left() + rr + rr * a.cos(), e.bottom() - rr + rr * a.sin())
                            }
                            6 => c(e.left(), e.bottom() - rr - f * h),
                            _ => {
                                let a = ang(std::f32::consts::PI);
                                c(e.left() + rr + rr * a.cos(), e.top() + rr + rr * a.sin())
                            }
                        };
                    }
                    d -= len;
                }
                e.left_top()
            };
            let speed = if active { per / 1.6 } else { per / 2.6 };
            let head_d = t * speed;
            let tail = per * 0.42;
            let n = 28;
            for k in 0..n {
                let d0 = head_d - tail * (k as f32 / n as f32);
                let d1 = head_d - tail * ((k + 1) as f32 / n as f32);
                let fade = 1.0 - k as f32 / n as f32;
                let col = if k < 3 { egui::Color32::from_rgb(127, 224, 196) } else { blue() };
                pt.line_segment([at(d0), at(d1)], Stroke::new(1.6_f32, col.gamma_multiply(fade * fade)));
            }
            ui.ctx().request_repaint();
        }
        // spark glyph
        let c = ico.center();
        let gcol = if lit { blue() } else { tx() };
        for (a, l0, l1) in [
            (0.0_f32, 1.5_f32, 6.0_f32),
            (90.0, 1.5, 6.0),
            (180.0, 1.5, 6.0),
            (270.0, 1.5, 6.0),
            (45.0, 2.5, 4.6),
            (135.0, 2.5, 4.6),
            (225.0, 2.5, 4.6),
            (315.0, 2.5, 4.6),
        ] {
            let (sn, cs) = a.to_radians().sin_cos();
            pt.line_segment([c + egui::vec2(cs * l0, sn * l0), c + egui::vec2(cs * l1, sn * l1)], Stroke::new(1.3_f32, gcol));
        }
        pt.text(egui::pos2(ico.right() + 8.0, r.center().y), egui::Align2::LEFT_CENTER, "Assistant", medium(12.5), tx());
        resp
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status")
            .exact_height(24.0)
            .frame(egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())).inner_margin(egui::Margin::symmetric(14, 0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let t = |s: String, c: Color32| RichText::new(s).font(mono(10.5)).color(c);
                    if self.page == Page::Workspace {
                        let errs = self.input_errors().len();
                        let (state, dot) = match &self.result {
                            _ if errs > 0 => (format!("{errs} input{} to fix", if errs == 1 { "" } else { "s" }), red()),
                            Some(r) if r.ok && r.checks.is_empty() => ("calculated · design values".into(), mute()),
                            Some(r) if r.ok && !r.overall_pass => ("FAIL".into(), red()),
                            Some(r) if r.ok && r.max_util >= REVIEW => {
                                (format!("PASS · review · η max {}", util_text(r.max_util)), amber())
                            }
                            Some(r) if r.ok => (format!("PASS · η max {}", util_text(r.max_util)), green()),
                            _ => ("not calculated".into(), mute()),
                        };
                        let (d, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
                        ui.painter().rect_filled(d.shrink(0.5), 0.0, dot);
                        ui.label(t(state, tx()));
                        if let Some(g) = self
                            .result
                            .as_ref()
                            .filter(|r| r.ok && errs == 0)
                            .and_then(|r| r.checks.iter().max_by(|a, b| a.utilisation.total_cmp(&b.utilisation)))
                        {
                            ui.add_space(12.0);
                            ui.label(t(format!("η max {} — {}", util_text(g.utilisation), g.name), mute()));
                        }
                        let d = &self.defs[self.mi];
                        let changed = d
                            .fields
                            .iter()
                            .zip(&self.st[self.mi].vals)
                            .filter(|(f, v)| (!f.check_only || self.st[self.mi].mode == "check") && (**v - f.default).abs() > 1e-12)
                            .count();
                        ui.add_space(12.0);
                        ui.label(t(format!("{changed} changed"), mute()));
                        ui.add_space(12.0);
                        ui.label(t(self.st[self.mi].param.clone(), mute()));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(r) = self.result.as_ref().filter(|r| r.ok) {
                            ui.label(t(format!("engine v{}", r.engine_version), mute()));
                            if self.page == Page::Workspace {
                                ui.add_space(12.0);
                                ui.label(t(format!("trace {}", &r.trace_hash[..8]), mute()));
                            }
                        }
                        ui.add_space(12.0);
                        let (s, c) = self.session_text();
                        ui.label(t(s, c));
                    });
                });
            });
    }

    fn command_palette(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::K)) {
            self.palette = !self.palette;
            self.pq.clear();
        }
        if !self.palette {
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.palette = false;
            return;
        }
        enum Cmd {
            Module(usize),
            Go(Page),
            Theme(Theme),
            SignOut,
            Export(ExportKind),
            Reset,
            Assistant,
            ToggleInsp,
            ToggleRail,
            View(VpTab),
        }
        let q = self.pq.trim().to_lowercase();
        let mut items: Vec<(String, String, String, Cmd)> = vec![];
        for (i, d) in self.defs.iter().enumerate() {
            items.push(("Modules".into(), d.name.into(), standard_of(d).into(), Cmd::Module(i)));
        }
        for (k, t) in [
            (ExportKind::Html, "Export calculation sheet (.html)"),
            (ExportKind::Doc, "Export Word document (.doc)"),
            (ExportKind::Print, "Print or save as PDF"),
        ] {
            items.push(("Calculation".into(), t.into(), String::new(), Cmd::Export(k)));
        }
        items.push(("Calculation".into(), "Reset inputs to defaults".into(), String::new(), Cmd::Reset));
        items.push(("Calculation".into(), "Open StruCal Assistant".into(), String::new(), Cmd::Assistant));
        items.push(("Layout".into(), if self.show_insp { "Hide inspector" } else { "Show inspector" }.into(), "[".into(), Cmd::ToggleInsp));
        items.push((
            "Layout".into(),
            if self.show_rail { "Hide verification rail" } else { "Show verification rail" }.into(),
            "]".into(),
            Cmd::ToggleRail,
        ));
        items.push(("Layout".into(), "Viewport: section".into(), String::new(), Cmd::View(VpTab::Section)));
        items.push(("Layout".into(), "Viewport: loading".into(), String::new(), Cmd::View(VpTab::Loading)));
        for pg in [Page::Workspace, Page::Project, Page::Standards, Page::Account, Page::Settings] {
            items.push(("Go to".into(), pg.label().into(), String::new(), Cmd::Go(pg)));
        }
        let other = if theme::current() == Theme::Obsidian { Theme::Drafting } else { Theme::Obsidian };
        items.push(("Preferences".into(), format!("Switch to {} theme", other.label()), "theme".into(), Cmd::Theme(other)));
        if self.cl.signed_in() {
            items.push(("Account".into(), "Sign out".into(), self.cl.email(), Cmd::SignOut));
        }
        let shown: Vec<&(String, String, String, Cmd)> =
            items.iter().filter(|(g, t, h, _)| q.is_empty() || format!("{g} {t} {h}").to_lowercase().contains(&q)).collect();
        let mut run: Option<usize> = None;
        let mut close = false;
        let screen = ctx.screen_rect();
        egui::Area::new("palette-scrim".into()).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            let (r, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(r, 0.0, Color32::from_black_alpha(if theme::current() == Theme::Obsidian { 150 } else { 90 }));
            if resp.clicked() {
                close = true;
            }
        });
        egui::Window::new("palette")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .order(egui::Order::Foreground)
            .min_width(560.0)
            .max_width(560.0)
            .default_width(560.0)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 90.0])
            .frame(
                egui::Frame::new()
                    .fill(surf())
                    .stroke(Stroke::new(1.0_f32, p().line_strong))
                    .corner_radius(CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_black_alpha(60) }),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                egui::Frame::new().inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
                    let te = ui.add(
                        egui::TextEdit::singleline(&mut self.pq)
                            .hint_text("Type a module, page or action")
                            .frame(false)
                            .font(FontId::proportional(15.0))
                            .desired_width(f32::INFINITY),
                    );
                    te.request_focus();
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !shown.is_empty() {
                        run = Some(0);
                    }
                });
                rule(ui);
                egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                    let mut group = "";
                    for (k, (g, t, h, _)) in shown.iter().enumerate() {
                        if g != group {
                            group = g;
                            egui::Frame::new().inner_margin(egui::Margin { left: 14, right: 14, top: 10, bottom: 4 }).show(ui, |ui| {
                                ui.label(RichText::new(g.to_uppercase()).font(strong(10.0)).extra_letter_spacing(0.9).color(p().ink4));
                            });
                        }
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
                        if resp.hovered() || k == 0 {
                            ui.painter().rect_filled(r.shrink2(egui::vec2(6.0, 1.0)), 6.0, head());
                        }
                        ui.painter().text(
                            r.left_center() + egui::vec2(14.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            t,
                            FontId::proportional(13.0),
                            tx(),
                        );
                        ui.painter().text(r.right_center() - egui::vec2(14.0, 0.0), egui::Align2::RIGHT_CENTER, h, mono(10.5), mute());
                        if resp.clicked() {
                            run = Some(k);
                        }
                    }
                    if shown.is_empty() {
                        egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                            ui.label(RichText::new("No matches.").color(mute()));
                        });
                    }
                });
            });
        if let Some(k) = run {
            match &shown[k].3 {
                Cmd::Module(i) => {
                    let i = *i;
                    self.select_module(i);
                    self.page = Page::Workspace;
                }
                Cmd::Go(pg) => self.page = *pg,
                Cmd::Theme(t) => {
                    let t = *t;
                    self.set_theme(ctx, t);
                }
                Cmd::SignOut => {
                    self.cl.sign_out();
                    self.session = SessionUi::SignedOut;
                }
                Cmd::Export(k) => {
                    let k = *k;
                    self.page = Page::Workspace;
                    self.export(k);
                }
                Cmd::Reset => {
                    self.page = Page::Workspace;
                    self.confirm_reset = true;
                }
                Cmd::Assistant => {
                    self.page = Page::Workspace;
                    self.asst.open = true;
                }
                Cmd::ToggleInsp => self.show_insp = !self.show_insp,
                Cmd::ToggleRail => self.show_rail = !self.show_rail,
                Cmd::View(v) => {
                    self.vp_tab = *v;
                    self.page = Page::Workspace;
                }
            }
            self.prefs_dirty = true;
            close = true;
        }
        if close {
            self.palette = false;
            self.pq.clear();
        }
    }

    // ------------------------------------------------------------ workbench

    fn inspector(&mut self, ctx: &egui::Context) -> bool {
        let mut ch = false;
        let mut pick: Option<usize> = None;
        let mi = self.mi;
        egui::SidePanel::left("inspector")
            .exact_width(340.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.set_max_width(340.0);
                let def = &self.defs[mi];
                // module header
                egui::Frame::new().inner_margin(egui::Margin { left: 16, right: 16, top: 14, bottom: 12 }).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(discipline_of(def.id).to_uppercase()).font(strong(10.5)).extra_letter_spacing(1.0).color(mute()),
                    );
                    egui::ComboBox::from_id_salt("module")
                        .width(ui.available_width() - 24.0)
                        .height(520.0)
                        .selected_text(RichText::new(def.name).font(strong(17.0)).color(tx()))
                        .show_ui(ui, |ui| {
                            for g in DISCIPLINES {
                                ui.label(RichText::new(g.to_uppercase()).font(strong(10.0)).color(p().ink4));
                                for (i, d) in self.defs.iter().enumerate().filter(|(_, d)| discipline_of(d.id) == g) {
                                    if ui.selectable_label(i == mi, d.name).clicked() {
                                        pick = Some(i);
                                    }
                                }
                            }
                            for u in modules::upcoming() {
                                ui.add_enabled(false, egui::Label::new(format!("{} · {}", u.name, u.note)));
                            }
                        });
                    ui.label(RichText::new(standard_of(def)).font(mono(11.0)).color(mute()));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let w = ui.available_width() / 2.0;
                        for (m, n) in [("design", "Design"), ("check", "Check")] {
                            let enabled = m == "design" || def.has_check_mode;
                            let on = self.st[mi].mode == m;
                            let b = egui::Button::new(RichText::new(n).font(medium(12.5)).color(if on { on_ink() } else { tx() }))
                                .fill(if on { tx() } else { surf() })
                                .stroke(Stroke::new(1.0_f32, if on { tx() } else { p().line_strong }))
                                .corner_radius(if m == "design" {
                                    CornerRadius { nw: 6, sw: 6, ne: 0, se: 0 }
                                } else {
                                    CornerRadius { nw: 0, sw: 0, ne: 6, se: 6 }
                                })
                                .min_size(egui::vec2(w, 28.0));
                            if ui.add_enabled(enabled, b).on_disabled_hover_text("This module has no Check Mode").clicked() && !on {
                                self.st[mi].mode = m.into();
                                ch = true;
                            }
                        }
                    });
                    let stt = &mut self.st[mi];
                    let sets = params::available();
                    egui::ComboBox::from_id_salt("ps")
                        .width(ui.available_width() - 24.0)
                        .truncate()
                        .selected_text(sets.iter().find(|p| p.id == stt.param).map(|p| p.label.clone()).unwrap_or_default())
                        .show_ui(ui, |ui| {
                            for ps in &sets {
                                if ui.selectable_value(&mut stt.param, ps.id.clone(), ps.label.clone()).changed() {
                                    ch = true;
                                }
                            }
                        });
                    if let Some(ps) = sets.iter().find(|p| p.id == stt.param) {
                        if ps.status.contains("NOT") {
                            ui.horizontal(|ui| {
                                let (d, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), Sense::hover());
                                ui.painter().rect_filled(d, 0.0, amber());
                                ui.label(RichText::new(ps.status.to_lowercase()).size(11.0).color(amber()));
                            });
                        }
                    }
                });
                rule(ui);

                let (field_errs, errs) = self.split_engine_errors();
                let groups = presets::presets_for(def);
                let n_changed = def
                    .fields
                    .iter()
                    .zip(&self.st[mi].vals)
                    .filter(|(f, v)| (!f.check_only || self.st[mi].mode == "check") && (**v - f.default).abs() > 1e-12)
                    .count();
                let n_shown = def.fields.iter().filter(|f| !f.check_only || self.st[mi].mode == "check").count();

                // footer: changed count and reset
                egui::TopBottomPanel::bottom("insp-foot")
                    .exact_height(34.0)
                    .frame(
                        egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())).inner_margin(egui::Margin::symmetric(16, 0)),
                    )
                    .show_inside(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(RichText::new(format!("{n_changed} of {n_shown} changed")).font(mono(10.5)).color(if n_changed > 0 {
                                blue()
                            } else {
                                mute()
                            }));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .add_enabled(n_changed > 0, egui::Button::new(RichText::new("Reset all").size(12.0)).frame(false))
                                    .clicked()
                                {
                                    let d = &self.defs[mi];
                                    self.st[mi].vals = d.fields.iter().map(|f| f.default).collect();
                                    ch = true;
                                    self.sync_text_now(mi);
                                }
                            });
                        });
                    });

                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    let def = &self.defs[mi];
                    if !errs.is_empty() {
                        egui::Frame::new().inner_margin(egui::Margin::symmetric(12, 10)).show(ui, |ui| {
                            notice(ui, red(), red_bg(), |ui| {
                                ui.label(RichText::new("The engine rejected these inputs").font(strong(12.5)).color(red()));
                                for e in &errs {
                                    ui.add(egui::Label::new(RichText::new(format!("· {e}")).size(12.0).color(tx())).wrap());
                                }
                            });
                        });
                    }
                    if !groups.is_empty() {
                        let open = egui::CollapsingHeader::new(
                            RichText::new("PRESETS").font(strong(10.5)).extra_letter_spacing(0.9).color(p().ink2),
                        )
                        .id_salt(("presets", def.id))
                        .default_open(true)
                        .show(ui, |ui| {
                            for g in &groups {
                                let cur = presets::matching(g, def, &self.st[mi].vals);
                                ui.horizontal(|ui| {
                                    ui.add_space(6.0);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(128.0, 30.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.set_width(128.0);
                                            ui.add(egui::Label::new(RichText::new(&g.title).size(12.0).color(p().ink2)).truncate())
                                                .on_hover_text(g.source);
                                        },
                                    );
                                    egui::ComboBox::from_id_salt(("preset", def.id, &g.id))
                                        .width(ui.available_width() - 36.0)
                                        .truncate()
                                        .selected_text(
                                            cur.map(|i| format!("{} · {}", g.options[i].label, g.options[i].hint))
                                                .unwrap_or_else(|| "Custom".into()),
                                        )
                                        .show_ui(ui, |ui| {
                                            for o in &g.options {
                                                if ui.selectable_label(false, format!("{}   {}", o.label, o.hint)).clicked() {
                                                    for (k, v) in &o.values {
                                                        if let Some(i) = def.fields.iter().position(|f| f.key == k) {
                                                            self.st[mi].vals[i] = *v;
                                                            self.st[mi].text[i] = fmt_num(*v);
                                                        }
                                                    }
                                                    ch = true;
                                                }
                                            }
                                        });
                                });
                            }
                            ui.add_space(4.0);
                        });
                        let _ = open;
                        rule(ui);
                    }
                    let stt = &mut self.st[mi];
                    let mut groups_seen: Vec<&str> = vec![];
                    for f in &def.fields {
                        if (!f.check_only || stt.mode == "check") && !groups_seen.contains(&f.group) {
                            groups_seen.push(f.group);
                        }
                    }
                    for g in groups_seen {
                        let idx: Vec<usize> = def
                            .fields
                            .iter()
                            .enumerate()
                            .filter(|(_, f)| f.group == g && (!f.check_only || stt.mode == "check"))
                            .map(|(i, _)| i)
                            .collect();
                        let n_mod = idx.iter().filter(|&&i| (stt.vals[i] - def.fields[i].default).abs() > 1e-12).count();
                        let title = RichText::new(format!(
                            "{}   {}",
                            g.to_uppercase(),
                            if n_mod > 0 { format!("· {n_mod} changed") } else { idx.len().to_string() }
                        ))
                        .font(strong(10.5))
                        .extra_letter_spacing(0.9)
                        .color(if n_mod > 0 { blue() } else { p().ink2 });
                        egui::CollapsingHeader::new(title).id_salt((def.id, g)).default_open(true).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            for &i in &idx {
                                let f = &def.fields[i];
                                let modified = (stt.vals[i] - f.default).abs() > 1e-12;
                                let bad = f.options.is_none() && parse_number(&stt.text[i]).is_none();
                                let row = ui.horizontal(|ui| {
                                    ui.set_min_height(36.0);
                                    ui.add_space(4.0);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(150.0, 36.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.set_width(150.0);
                                            ui.add(egui::Label::new(RichText::new(f.label).size(12.5).color(p().ink2)).wrap());
                                        },
                                    );
                                    let w = (ui.available_width() - 6.0).max(80.0);
                                    if let Some(opts) = &f.options {
                                        let cur = opts.iter().find(|o| (o.0 - stt.vals[i]).abs() < 1e-9).map(|o| o.1).unwrap_or("-");
                                        egui::ComboBox::from_id_salt(("opt", def.id, f.key))
                                            .width(w - 24.0)
                                            .truncate()
                                            .selected_text(cur)
                                            .show_ui(ui, |ui| {
                                                for (val, lab) in opts {
                                                    if ui.selectable_label((*val - stt.vals[i]).abs() < 1e-9, *lab).clicked() {
                                                        stt.vals[i] = *val;
                                                        stt.text[i] = fmt_num(*val);
                                                        ch = true;
                                                    }
                                                }
                                            });
                                    } else {
                                        // UnitInput: right-aligned tabular value with the unit inside the field.
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 28.0), Sense::hover());
                                        let unit = if f.unit == "-" { "" } else { f.unit };
                                        let unit_w = if unit.is_empty() { 0.0 } else { 8.0 + unit.chars().count() as f32 * 6.4 };
                                        let edit = egui::Rect::from_min_max(
                                            rect.min + egui::vec2(6.0, 0.0),
                                            rect.max - egui::vec2(unit_w + 8.0, 0.0),
                                        );
                                        let id = ui.make_persistent_id(("in", def.id, f.key));
                                        let focus = ui.memory(|m| m.has_focus(id));
                                        let hovered = ui.rect_contains_pointer(rect);
                                        let fill = if bad {
                                            red_bg()
                                        } else if focus {
                                            surf()
                                        } else {
                                            p().sunken
                                        };
                                        let stroke = if bad {
                                            red()
                                        } else if focus {
                                            blue()
                                        } else if hovered {
                                            p().ink4
                                        } else {
                                            line()
                                        };
                                        ui.painter().rect_filled(rect, 6.0, fill);
                                        ui.painter().rect_stroke(
                                            rect,
                                            6.0,
                                            Stroke::new(if focus { 1.5_f32 } else { 1.0 }, stroke),
                                            StrokeKind::Inside,
                                        );
                                        if !unit.is_empty() {
                                            ui.painter().text(
                                                rect.right_center() - egui::vec2(8.0, 0.0),
                                                egui::Align2::RIGHT_CENTER,
                                                unit,
                                                mono(10.5),
                                                p().ink4,
                                            );
                                        }
                                        let te = ui.put(
                                            edit,
                                            egui::TextEdit::singleline(&mut stt.text[i])
                                                .id(id)
                                                .desired_width(edit.width())
                                                .frame(false)
                                                .horizontal_align(egui::Align::RIGHT)
                                                .vertical_align(egui::Align::Center)
                                                .font(mono(13.0))
                                                .text_color(if bad { red() } else { tx() })
                                                .margin(egui::Margin::ZERO),
                                        );
                                        if te.changed() {
                                            if let Some(v) = parse_number(&stt.text[i]) {
                                                stt.vals[i] = v;
                                                ch = true;
                                            } else {
                                                ch = true;
                                            }
                                        }
                                        let hover = if bad {
                                            format!(
                                                "{} · default {} {}",
                                                if stt.text[i].trim().is_empty() { "Required" } else { "Not a number" },
                                                fmt_num(f.default),
                                                f.unit
                                            )
                                        } else {
                                            format!("Default {} {}", fmt_num(f.default), f.unit)
                                        };
                                        te.on_hover_text(hover);
                                    }
                                });
                                let r = row.response.rect;
                                if modified && !bad {
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(
                                            egui::pos2(r.left() - 12.0, r.top() + 8.0),
                                            egui::vec2(2.0, r.height() - 16.0),
                                        ),
                                        0.0,
                                        blue(),
                                    );
                                }
                                if bad {
                                    ui.horizontal(|ui| {
                                        ui.add_space(158.0);
                                        ui.label(
                                            RichText::new(if stt.text[i].trim().is_empty() { "Required" } else { "Not a number" })
                                                .size(11.0)
                                                .color(red()),
                                        );
                                    });
                                } else if let Some(msg) = field_errs.get(f.key) {
                                    ui.horizontal(|ui| {
                                        ui.add_space(12.0);
                                        ui.add(egui::Label::new(RichText::new(msg).size(11.0).color(red())).wrap());
                                    });
                                    ui.add_space(4.0);
                                }
                                ui.painter().line_segment(
                                    [egui::pos2(r.left() - 16.0, r.bottom()), egui::pos2(r.right() + 16.0, r.bottom())],
                                    Stroke::new(1.0_f32, line()),
                                );
                            }
                            if idx.iter().any(|&i| def.fields[i].check_only) {
                                ui.add_space(6.0);
                                ui.label(RichText::new("Values as shown on the drawing (Check Mode).").size(11.0).color(mute()));
                                ui.add_space(6.0);
                            }
                        });
                    }
                    ui.add_space(16.0);
                });
            });
        if let Some(i) = pick {
            self.select_module(i);
            return false;
        }
        ch
    }

    /// Text sync helper usable while `self` is mutably borrowed through a closure capture of fields.
    fn sync_text_now(&mut self, mi: usize) {
        self.sync_text(mi);
    }

    fn verification_rail(&mut self, ctx: &egui::Context) {
        let errs = self.input_errors();
        let mut open_checks = false;
        let mut eta_order = self.eta_order;
        egui::SidePanel::right("verify")
            .exact_width(320.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let meta =
                    self.result.as_ref().filter(|r| r.ok).map(|r| format!("{} checks", r.checks.len())).unwrap_or_else(|| "—".into());
                pane_header(ui, "Verification", &meta);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    egui::Frame::new().inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;
                        if !errs.is_empty() {
                            let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 3.0), Sense::hover());
                            ui.painter().rect_filled(bar, 0.0, red());
                            ui.label(RichText::new("INPUTS").font(mono_strong(26.0)).color(red()));
                            ui.label(
                                RichText::new(format!(
                                    "{} input{} must be numbers before the engine runs:",
                                    errs.len(),
                                    if errs.len() == 1 { "" } else { "s" }
                                ))
                                .size(12.0)
                                .color(mute()),
                            );
                            for (l, m) in &errs {
                                ui.label(RichText::new(format!("· {l} — {m}")).size(12.0).color(tx()));
                            }
                            return;
                        }
                        let Some(r) = self.result.as_ref().filter(|r| r.ok) else {
                            ui.add_space(24.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("No verification").font(strong(13.0)));
                                ui.label(RichText::new("Results appear when every input is valid.").color(mute()));
                            });
                            return;
                        };
                        let gov = r.checks.iter().max_by(|a, b| a.utilisation.total_cmp(&b.utilisation));
                        let (t, c) = if r.checks.is_empty() {
                            ("RESULTS", tx())
                        } else if !r.overall_pass {
                            ("FAIL", red())
                        } else {
                            ("PASS", status_color(true, r.max_util))
                        };
                        let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 3.0), Sense::hover());
                        ui.painter().rect_filled(bar, 0.0, if r.checks.is_empty() { p().line_strong } else { c });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(t).font(mono_strong(34.0)).color(c));
                            if let Some(g) = gov {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                                    ui.label(RichText::new(util_text(g.utilisation)).font(mono_strong(13.0)).color(tx()));
                                    ui.label(RichText::new("η max").font(mono(11.0)).color(mute()));
                                });
                            }
                        });
                        let review = r.checks.iter().any(|k| k.utilisation >= REVIEW);
                        ui.label(
                            RichText::new(if r.checks.is_empty() {
                                "Design values only; no PASS/FAIL verdict.".to_string()
                            } else {
                                format!(
                                    "{} of {} checks satisfied{}.",
                                    r.checks.iter().filter(|k| k.pass).count(),
                                    r.checks.len(),
                                    if review && r.overall_pass { "; at least one at or above the 0.85 review threshold" } else { "" }
                                )
                            })
                            .size(12.0)
                            .color(mute()),
                        );
                        ui.label(RichText::new("Implemented checks only · not an engineer's approval").size(11.0).color(p().ink4));
                        if let Some(g) = gov {
                            heading(ui, "Governing limit state");
                            let card = egui::Frame::new()
                                .fill(p().sunken)
                                .stroke(Stroke::new(1.0_f32, p().line_strong))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(egui::Margin::same(12))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(RichText::new(&g.name).font(strong(13.0)));
                                    ui.label(RichText::new(&g.reference).font(mono(10.5)).color(blue()));
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("Ed {}", g.demand)).font(mono(11.0)));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(format!("Rd {}", g.capacity)).font(mono(11.0)))
                                        });
                                    });
                                    let col = status_color(g.pass, g.utilisation);
                                    ui.label(
                                        RichText::new(if g.utilisation >= 9.99 {
                                            "n/a".into()
                                        } else {
                                            format!("{:.0}%", g.utilisation * 100.0)
                                        })
                                        .font(mono_strong(26.0))
                                        .color(col),
                                    );
                                    util_bar(ui, g.utilisation, col, 6.0);
                                });
                            let click = ui
                                .interact(card.response.rect, ui.id().with("gov-card"), Sense::click())
                                .on_hover_text("Open the itemised checks in the ledger");
                            if click.hovered() {
                                ui.painter().rect_stroke(card.response.rect, 8.0, Stroke::new(1.0_f32, tx()), StrokeKind::Inside);
                            }
                            if click.clicked() {
                                open_checks = true;
                            }
                        }
                        if !r.checks.is_empty() {
                            ui.add_space(14.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("UTILISATION · ED / RD").font(strong(10.5)).extra_letter_spacing(0.9).color(mute()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    if tbtn(ui, "Sheet", !eta_order).clicked() {
                                        eta_order = false;
                                    }
                                    if tbtn(ui, "η", eta_order).clicked() {
                                        eta_order = true;
                                    }
                                });
                            });
                            ui.add_space(4.0);
                            rule(ui);
                            ui.add_space(6.0);
                        }
                        let mut sorted: Vec<_> = r.checks.iter().collect();
                        if eta_order {
                            sorted.sort_by(|a, b| b.utilisation.total_cmp(&a.utilisation));
                        }
                        for k in sorted {
                            let col = status_color(k.pass, k.utilisation);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&k.name).size(12.5));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(util_text(k.utilisation)).font(mono_strong(12.5)).color(col))
                                });
                            });
                            util_bar(ui, k.utilisation, col, 4.0);
                            ui.add_space(6.0);
                        }
                        heading(ui, &format!("Observations  {}", r.observations.len()));
                        if r.observations.is_empty() {
                            ui.label(RichText::new("None.").color(mute()));
                        }
                        for o in &r.observations {
                            let (fg, bg) = match o.level.as_str() {
                                "RED" => (red(), red_bg()),
                                "AMBER" => (amber(), amber_bg()),
                                _ => (blue(), sel()),
                            };
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&o.id).font(mono(10.5)).color(mute()));
                                badge(ui, &o.level, fg, bg);
                            });
                            ui.add(egui::Label::new(RichText::new(&o.text).size(12.0).color(tx())).wrap());
                            ui.add_space(4.0);
                            rule(ui);
                            ui.add_space(4.0);
                        }
                        if !self.saved.is_empty() {
                            ui.add_space(8.0);
                            notice(ui, green(), p().pass_bg, |ui| {
                                ui.add(egui::Label::new(RichText::new(&self.saved).size(11.5).color(tx())).wrap());
                            });
                        }
                    });
                });
            });
        self.eta_order = eta_order;
        if open_checks {
            self.ledger = LedgerTab::Checks;
        }
    }

    fn center(&mut self, ctx: &egui::Context) {
        let mi = self.mi;
        let errs_n = self.input_errors().len();
        egui::CentralPanel::default().frame(egui::Frame::new().fill(bg())).show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            // ---------- auditable calculation ledger (resizable) ----------
            egui::TopBottomPanel::bottom("ledger")
                .resizable(true)
                .default_height((ui.available_height() * 0.55).max(260.0))
                .height_range(160.0..=(ui.available_height() - 140.0).max(200.0))
                .frame(egui::Frame::new().fill(surf()).stroke(Stroke::new(1.0_f32, line())))
                .show_inside(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let r = self.result.as_ref();
                    let ok = r.is_some_and(|r| r.ok) && errs_n == 0;
                    // tabs
                    egui::Frame::new().fill(surf()).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let counts = r.filter(|_| ok).map(|r| (r.trace.steps.len(), r.checks.len())).unwrap_or((0, 0));
                            for (t, n, c) in [
                                (LedgerTab::Ledger, "LEDGER", Some(counts.0)),
                                (LedgerTab::Checks, "CHECKS", Some(counts.1)),
                                (LedgerTab::Summary, "SUMMARY", None),
                                (LedgerTab::Scope, "SCOPE", None),
                            ] {
                                let on = self.ledger == t;
                                let label = match c {
                                    Some(c) if ok => format!("{n}  {c}"),
                                    _ => n.to_string(),
                                };
                                let w = 28.0 + label.chars().count() as f32 * 7.0;
                                let (rr, resp) = ui.allocate_exact_size(egui::vec2(w, 36.0), Sense::click());
                                if on {
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_max(egui::pos2(rr.left(), rr.bottom() - 2.0), rr.right_bottom()),
                                        0.0,
                                        tx(),
                                    );
                                }
                                ui.painter().text(
                                    rr.center(),
                                    egui::Align2::CENTER_CENTER,
                                    label,
                                    strong(11.0),
                                    if on || resp.hovered() { tx() } else { mute() },
                                );
                                ui.painter().line_segment([rr.right_top(), rr.right_bottom()], Stroke::new(1.0_f32, line()));
                                if resp.clicked() {
                                    self.ledger = t;
                                }
                            }
                            if self.ledger == LedgerTab::Ledger && ok {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.spacing_mut().item_spacing.x = 14.0;
                                    ui.add_space(4.0);
                                    if ghost(ui, "Collapse all").clicked() {
                                        self.ledger_force = Some(false);
                                    }
                                    if ghost(ui, "Expand all").clicked() {
                                        self.ledger_force = Some(true);
                                    }
                                });
                            }
                        });
                    });
                    rule(ui);
                    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("ledger-scroll").show(ui, |ui| {
                        egui::Frame::new().inner_margin(egui::Margin::symmetric(20, 14)).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            let w = ui.available_width();
                            ui.set_width(w);
                            let Some(r) = r else { return };
                            if errs_n > 0 {
                                notice(ui, red(), red_bg(), |ui| {
                                    ui.label(RichText::new("Calculation paused: fix the highlighted inputs in the inspector.").color(tx()));
                                });
                                return;
                            }
                            if !r.ok {
                                notice(ui, red(), red_bg(), |ui| {
                                    ui.label(RichText::new("The calculation cannot run.").font(strong(13.0)).color(tx()));
                                    for e in &r.errors {
                                        ui.add(egui::Label::new(RichText::new(format!("· {e}")).color(tx())).wrap());
                                    }
                                });
                                return;
                            }
                            if r.param_status.contains("NOT") {
                                notice(ui, amber(), amber_bg(), |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "Parameter set {}. Confirm the governing National Annex before issue for construction.",
                                                r.param_status
                                            ))
                                            .size(12.5)
                                            .color(tx()),
                                        )
                                        .wrap(),
                                    );
                                });
                                ui.add_space(6.0);
                            }
                            for wn in &r.warnings {
                                ui.colored_label(amber(), wn);
                            }
                            match self.ledger {
                                LedgerTab::Ledger => Self::ledger_steps(ui, &self.defs[mi], r, w, self.ledger_force),
                                LedgerTab::Checks => Self::ledger_checks(ui, r),
                                LedgerTab::Summary => {
                                    egui::Grid::new("sum").num_columns(2).spacing([28.0, 8.0]).show(ui, |ui| {
                                        for k in &r.summary {
                                            ui.label(RichText::new(&k.label).color(mute()));
                                            ui.label(RichText::new(&k.value).font(mono_strong(12.5)));
                                            ui.end_row();
                                        }
                                    });
                                }
                                LedgerTab::Scope => {
                                    for t in &r.scope_notes {
                                        ui.add(egui::Label::new(RichText::new(format!("· {t}")).size(12.5).color(tx())).wrap());
                                    }
                                    ui.add_space(10.0);
                                    ui.label(
                                        RichText::new(format!("SHA-256 {} · engine v{}", r.trace_hash, r.engine_version))
                                            .font(mono(10.0))
                                            .color(mute()),
                                    );
                                }
                            }
                        });
                    });
                });

            // ---------- viewport: Section and Loading on a pannable, zoomable drafting grid ----------
            egui::CentralPanel::default().frame(egui::Frame::new().fill(p().sunken)).show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let def_id = self.defs[mi].id;
                let has_section = errs_n == 0 && self.result.as_ref().is_some_and(|r| r.ok && r.drawing.is_some());
                let vi = if self.vp_tab == VpTab::Section { 0 } else { 1 };
                egui::Frame::new().fill(surf()).inner_margin(egui::Margin { left: 0, right: 12, top: 0, bottom: 0 }).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(36.0);
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (t, n) in [(VpTab::Section, "SECTION"), (VpTab::Loading, "LOADING")] {
                            let on = self.vp_tab == t;
                            let (rr, resp) = ui.allocate_exact_size(egui::vec2(96.0, 36.0), Sense::click());
                            if on {
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_max(egui::pos2(rr.left(), rr.bottom() - 2.0), rr.right_bottom()),
                                    0.0,
                                    tx(),
                                );
                            }
                            ui.painter().text(
                                rr.center(),
                                egui::Align2::CENTER_CENTER,
                                n,
                                strong(11.0),
                                if on || resp.hovered() { tx() } else { mute() },
                            );
                            ui.painter().line_segment([rr.right_top(), rr.right_bottom()], Stroke::new(1.0_f32, line()));
                            if resp.clicked() {
                                self.vp_tab = t;
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            if tbtn(ui, "Fit", false).on_hover_text("Fit to view (double-click the canvas)").clicked() {
                                self.vp_views[vi].fit();
                            }
                            if tbtn(ui, "+", false).on_hover_text("Zoom in").clicked() {
                                self.vp_views[vi].zoom_by(1.2);
                            }
                            ui.label(RichText::new(format!("{:>3.0}%", self.vp_views[vi].zoom * 100.0)).font(mono(11.0)).color(mute()));
                            if tbtn(ui, "−", false).on_hover_text("Zoom out").clicked() {
                                self.vp_views[vi].zoom_by(1.0 / 1.2);
                            }
                            let can_dims = self.vp_tab == VpTab::Section && has_section;
                            if ui
                                .add_enabled_ui(can_dims, |ui| tbtn(ui, "Dims", self.dims))
                                .inner
                                .on_hover_text("Show or hide dimensions")
                                .clicked()
                            {
                                self.dims = !self.dims;
                                self.prefs_dirty = true;
                            }
                        });
                    });
                });
                rule(ui);
                let h = ui.available_height().max(120.0);
                let dims = self.dims;
                match self.vp_tab {
                    VpTab::Section => {
                        let drawing = self.result.as_ref().filter(|r| r.ok && errs_n == 0).and_then(|r| r.drawing.clone());
                        let empty = if errs_n > 0 || !self.result.as_ref().is_some_and(|r| r.ok) {
                            "Geometry appears when every input is valid."
                        } else {
                            "This module works on action effects and has no cross-section. See Loading."
                        };
                        canvas::show(ui, h, &mut self.vp_views[0], "SECTION · proportional · mm", empty, |sink| {
                            viewport::section(sink, drawing.as_ref(), dims)
                        });
                    }
                    VpTab::Loading => {
                        let vals: Vec<(&'static str, f64)> =
                            self.defs[mi].fields.iter().map(|f| f.key).zip(self.st[mi].vals.iter().copied()).collect();
                        let read = move |k: &str| vals.iter().find(|(key, _)| *key == k).map(|(_, v)| *v).unwrap_or(0.0);
                        canvas::show(
                            ui,
                            h,
                            &mut self.vp_views[1],
                            "LOADING · schematic · inputs only",
                            "No loading diagram for this module.",
                            |sink| loading::draw(sink, def_id, &read),
                        );
                    }
                }
            });
        });
    }

    fn ledger_steps(ui: &mut egui::Ui, def: &ModDef, r: &ModResult, w: f32, force: Option<bool>) {
        let ref_w = 150.0;
        let num_w = 30.0;
        let out_w = 170.0;
        let calc_w = (w - num_w - ref_w - out_w - 3.0 * 14.0 - 24.0).max(200.0);
        // column header (indented like the collapsible sections below)
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().indent);
            for (t, wd) in [("#", num_w), ("REFERENCE", ref_w), ("EQUATION · SUBSTITUTION", calc_w), ("RESULT", out_w)] {
                ui.allocate_ui_with_layout(egui::vec2(wd, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_width(wd);
                    ui.label(RichText::new(t).font(strong(10.0)).extra_letter_spacing(0.8).color(mute()));
                });
            }
        });
        rule(ui);
        let mut n = 0usize;
        let mut i = 0usize;
        while i < r.trace.steps.len() {
            let sec = &r.trace.steps[i].section;
            let j = r.trace.steps[i..].iter().position(|s| &s.section != sec).map(|p| i + p).unwrap_or(r.trace.steps.len());
            egui::CollapsingHeader::new(
                RichText::new(format!("{sec}   · {} step{}", j - i, if j - i == 1 { "" } else { "s" })).font(strong(13.0)).color(tx()),
            )
            .id_salt(("sec", def.id, i))
            .default_open(true)
            .open(force)
            .show(ui, |ui| {
                for s in &r.trace.steps[i..j] {
                    n += 1;
                    let row = ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(egui::vec2(num_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.set_width(num_w);
                            ui.label(RichText::new(format!("{n:02}")).font(mono(11.0)).color(p().ink4));
                        });
                        ui.allocate_ui_with_layout(egui::vec2(ref_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.set_width(ref_w);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(if s.reference.is_empty() { "—" } else { &s.reference }).font(mono(11.0)).color(blue()),
                                )
                                .wrap(),
                            );
                        });
                        ui.vertical(|ui| {
                            ui.set_width(calc_w);
                            ui.label(RichText::new(&s.title).font(strong(12.5)));
                            let f = trace::plain(&s.formula);
                            if !f.is_empty() {
                                ui.add(egui::Label::new(RichText::new(f).font(mono(12.0)).color(p().ink2)).wrap());
                            }
                            if !s.calculation.is_empty() {
                                ui.add(egui::Label::new(RichText::new(&s.calculation).font(mono(11.5)).color(tx())).wrap());
                            }
                            if !s.comment.is_empty() {
                                ui.add(egui::Label::new(RichText::new(&s.comment).size(11.0).color(mute())).wrap());
                            }
                        });
                        ui.allocate_ui_with_layout(egui::vec2(out_w, 0.0), egui::Layout::top_down(egui::Align::Max), |ui| {
                            ui.set_width(out_w);
                            let col = match s.output.as_str() {
                                "PASS" => green(),
                                "FAIL" => red(),
                                _ => tx(),
                            };
                            ui.add(egui::Label::new(RichText::new(&s.output).font(mono_strong(12.0)).color(col)).wrap());
                        });
                    });
                    let rr = row.response.rect;
                    ui.add_space(4.0);
                    ui.painter().line_segment(
                        [egui::pos2(rr.left(), rr.bottom() + 2.0), egui::pos2(rr.right(), rr.bottom() + 2.0)],
                        Stroke::new(1.0_f32, line()),
                    );
                    ui.add_space(4.0);
                }
            });
            i = j;
        }
    }

    fn ledger_checks(ui: &mut egui::Ui, r: &ModResult) {
        if r.checks.is_empty() {
            ui.label(
                RichText::new("This module combines action effects. It gives design values, not a PASS/FAIL verdict.")
                    .size(12.0)
                    .color(mute()),
            );
            return;
        }
        let gov_i = r.checks.iter().enumerate().max_by(|a, b| a.1.utilisation.total_cmp(&b.1.utilisation)).map(|x| x.0);
        egui::Grid::new("ls").num_columns(6).spacing([16.0, 10.0]).show(ui, |ui| {
            for h in ["#", "CHECK · CLAUSE", "Ed", "Rd", "η", "STATUS"] {
                ui.label(RichText::new(h).font(strong(10.0)).color(mute()));
            }
            ui.end_row();
            for (i, k) in r.checks.iter().enumerate() {
                let col = status_color(k.pass, k.utilisation);
                ui.label(RichText::new(format!("{:02}", i + 1)).font(mono(11.0)).color(p().ink4));
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&k.name).font(strong(12.5)));
                        if Some(i) == gov_i {
                            badge(ui, "governing", on_ink(), tx());
                        }
                    });
                    ui.label(RichText::new(&k.reference).font(mono(10.5)).color(blue()));
                });
                ui.label(RichText::new(&k.demand).font(mono(11.5)));
                ui.label(RichText::new(&k.capacity).font(mono(11.5)));
                ui.vertical(|ui| {
                    ui.set_width(110.0);
                    ui.label(RichText::new(util_text(k.utilisation)).font(mono_strong(12.0)).color(col));
                    util_bar(ui, k.utilisation, col, 3.0);
                });
                let (txt, bg_) = if !k.pass {
                    ("fail", red_bg())
                } else if k.utilisation >= REVIEW {
                    ("review", amber_bg())
                } else {
                    ("pass", p().pass_bg)
                };
                ui.horizontal(|ui| badge(ui, txt, col, bg_));
                ui.end_row();
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new("η = Ed / Rd. Emerald below 0.85 · amber at or above 0.85 · crimson when overstressed. Implemented checks only.")
                .size(11.0)
                .color(mute()),
        );
    }

    // ------------------------------------------------------------ other pages

    fn standards(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::new().fill(bg()).inner_margin(egui::Margin::symmetric(32, 24))).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.set_max_width(1200.0);
                ui.label(RichText::new("STANDARDS").font(strong(10.5)).extra_letter_spacing(1.2).color(mute()));
                ui.label(RichText::new("Formula registry and parameter sets").font(strong(26.0)));
                ui.label(RichText::new("Every implemented formula with its source clause and verification state, and the partial factors each parameter set applies.").color(mute()));
                ui.add_space(10.0);
                let (hr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), Sense::hover());
                ui.painter().rect_filled(hr, 0.0, tx());

                sheet_heading(ui, "A", "Parameter sets · partial factors compared");
                let sets = params::available();
                card(ui, |ui| {
                    egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                        egui::Grid::new("ps").striped(false).spacing([28.0, 9.0]).show(ui, |ui| {
                            ui.label(RichText::new("FACTOR").font(strong(10.0)).color(mute()));
                            for s in &sets {
                                ui.label(RichText::new(&s.label).font(strong(11.5)));
                            }
                            ui.end_row();
                            let rows: [(&str, fn(&params::ParamSet) -> f64); 8] = [
                                ("γc · concrete", |s| s.gamma_c),
                                ("γs · reinforcement", |s| s.gamma_s),
                                ("αcc · long-term compression", |s| s.alpha_cc),
                                ("αcw · strut", |s| s.alpha_cw),
                                ("γG · permanent actions", |s| s.gamma_g),
                                ("γQ · variable actions", |s| s.gamma_q),
                                ("γM0 · steel cross-section", |s| s.gamma_m0),
                                ("γM1 · steel member stability", |s| s.gamma_m1),
                            ];
                            for (l, f) in rows {
                                let vals: Vec<f64> = sets.iter().map(f).collect();
                                let differs = vals.iter().any(|v| (v - vals[0]).abs() > 1e-12);
                                ui.label(RichText::new(l).color(if differs { amber() } else { tx() }));
                                for v in vals {
                                    ui.label(RichText::new(format!("{v:.2}")).font(mono(12.0)).color(if differs { amber() } else { tx() }));
                                }
                                ui.end_row();
                            }
                            ui.label(RichText::new("Status").color(mute()));
                            for s in &sets {
                                let warn = s.status.contains("NOT");
                                ui.horizontal(|ui| badge(ui, &s.status, if warn { amber() } else { green() }, if warn { amber_bg() } else { p().pass_bg }));
                            }
                            ui.end_row();
                        });
                    });
                });
                ui.add_space(6.0);
                ui.label(RichText::new("Rows where the sets differ are amber. No set is a verified National Annex for Uganda; confirm the governing annex before issue.").size(11.0).color(mute()));

                let all = registry::formulas();
                sheet_heading(ui, "B", &format!("Formula registry · {}", all.len()));
                egui::ScrollArea::horizontal().id_salt("reg-x").show(ui, |ui| card(ui, |ui| {
                    egui::Frame::new().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                        egui::Grid::new("reg").striped(true).spacing([20.0, 9.0]).show(ui, |ui| {
                            for h in ["ID", "FORMULA", "STANDARD", "CLAUSE", "SINCE", "VERIFICATION"] {
                                ui.label(RichText::new(h).font(strong(10.0)).color(mute()));
                            }
                            ui.end_row();
                            for f in all {
                                ui.label(RichText::new(f.id).font(mono_strong(11.5)));
                                ui.label(f.name);
                                ui.label(RichText::new(f.standard).font(mono(11.0)).color(mute()));
                                ui.label(RichText::new(f.clause).font(mono(11.0)).color(blue()));
                                ui.label(RichText::new(f.module_version).font(mono(11.0)).color(p().ink4));
                                let checked = f.verification.starts_with("CHECKED");
                                ui.horizontal(|ui| badge(ui, f.verification, if checked { green() } else { amber() }, if checked { p().pass_bg } else { amber_bg() }));
                                ui.end_row();
                            }
                        });
                    });
                }));
                ui.add_space(24.0);
            });
        });
    }

    fn settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::new().fill(bg()).inner_margin(egui::Margin::symmetric(32, 24))).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.set_max_width(760.0);
                ui.label(RichText::new("SETTINGS").font(strong(10.5)).extra_letter_spacing(1.2).color(mute()));
                ui.label(RichText::new("Workstation preferences").font(strong(26.0)));
                ui.add_space(10.0);
                let (hr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), Sense::hover());
                ui.painter().rect_filled(hr, 0.0, tx());

                sheet_heading(ui, "A", "Appearance");
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for t in [Theme::Drafting, Theme::Obsidian] {
                        if tbtn(ui, t.label(), theme::current() == t).clicked() {
                            self.set_theme(ctx, t);
                        }
                    }
                });
                ui.label(RichText::new("Drafting is an off-white drafting surface; Obsidian is a dark workstation mode. The choice is remembered.").size(12.0).color(mute()));

                sheet_heading(ui, "B", "Session");
                let (s, c) = self.session_text();
                ui.horizontal(|ui| {
                    let (d, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
                    ui.painter().rect_filled(d, 0.0, c);
                    ui.label(RichText::new(s).font(mono(12.0)));
                });
                ui.label(
                    RichText::new(if cfg!(windows) {
                        "Your sign-in is kept in Windows Credential Manager and restored when StruCal starts, so you stay signed in across restarts. Signing out removes it."
                    } else {
                        "Your sign-in is kept in a private file (owner read/write only) in the StruCal configuration folder and restored on launch. Signing out removes it."
                    })
                    .size(12.0)
                    .color(mute()),
                );
                ui.horizontal(|ui| {
                    if matches!(self.session, SessionUi::Offline(_)) && self.srx.is_none() && ui.button("Retry now").clicked() {
                        self.next_retry = Some(Instant::now());
                    }
                    if self.cl.signed_in() || self.cl.has_pending_session() {
                        if tbtn(ui, "Sign out", false).clicked() {
                            self.cl.sign_out();
                            self.session = SessionUi::SignedOut;
                        }
                    } else if primary(ui, "Sign in").clicked() {
                        self.page = Page::Account;
                    }
                });
                if let SessionUi::Offline(e) = &self.session {
                    ui.label(RichText::new(format!("Last error: {e}")).font(mono(11.0)).color(amber()));
                }

                sheet_heading(ui, "C", "Software updates");
                ui.label(RichText::new(format!("Installed version {}", update::CURRENT)).font(mono(12.0)));
                let busy = matches!(self.upd, Upd::Checking | Upd::Installing);
                if ui.add_enabled_ui(!busy, |ui| primary(ui, "Check for updates")).inner.clicked() {
                    let (tx_, rx_) = channel();
                    self.rx = Some(rx_);
                    self.upd = Upd::Checking;
                    std::thread::spawn(move || {
                        let _ = tx_.send(update::check());
                    });
                }
                let mut go: Option<String> = None;
                match &self.upd {
                    Upd::Idle => {}
                    Upd::Checking => {
                        ui.label(RichText::new("Checking for updates…").color(mute()));
                    }
                    Upd::UpToDate => {
                        ui.colored_label(green(), "You are on the latest version.");
                    }
                    Upd::Available(a) => {
                        ui.colored_label(blue(), format!("Version {} is available.", a.version));
                        if !a.notes.is_empty() {
                            ui.add(egui::Label::new(RichText::new(&a.notes).size(12.0).color(mute())).wrap());
                        }
                        if tbtn(ui, "Download and install (StruCal restarts)", false).clicked() {
                            go = Some(a.url.clone());
                        }
                    }
                    Upd::Installing => {
                        ui.label("Downloading the installer. StruCal closes and reopens when it finishes…");
                    }
                    Upd::Failed(e) => {
                        ui.colored_label(red(), e);
                    }
                }
                if let Some(u) = go {
                    let (tx_, rx_) = channel();
                    self.irx = Some(rx_);
                    self.upd = Upd::Installing;
                    std::thread::spawn(move || {
                        let _ = tx_.send(update::install(&u));
                    });
                }
                ui.label(RichText::new("Updates install in place. You do not need to uninstall first.").color(mute()).size(12.0));
                sheet_heading(ui, "D", "About");
                ui.label(RichText::new("Deterministic Rust engine linked natively — no webview. Typefaces: Geist and JetBrains Mono (SIL Open Font License).").size(12.0).color(mute()));
                ui.label(RichText::new(format!("Preferences: {}", secure_store::config_dir().display())).font(mono(11.0)).color(p().ink4));
            });
        });
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
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        if let Some(rx) = &self.irx {
            if let Ok(m) = rx.try_recv() {
                if let Err(e) = m {
                    self.upd = Upd::Failed(e);
                }
                self.irx = None;
            }
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        self.poll_session(ctx);

        self.rail(ctx);
        let (export, reset) = self.command_bar(ctx);
        self.status_bar(ctx);
        if reset {
            self.confirm_reset = true;
        }
        if let Some(k) = export {
            self.export(k);
        }
        // Layout shortcuts, as on the web: [ toggles the inspector, ] the verification rail (not while typing).
        if self.page == Page::Workspace && !self.palette && !ctx.wants_keyboard_input() {
            let (l, r) = ctx.input(|i| {
                let mut l = false;
                let mut r = false;
                for e in &i.events {
                    if let egui::Event::Text(t) = e {
                        l |= t == "[";
                        r |= t == "]";
                    }
                }
                (l, r)
            });
            if l {
                self.show_insp = !self.show_insp;
                self.prefs_dirty = true;
            }
            if r {
                self.show_rail = !self.show_rail;
                self.prefs_dirty = true;
            }
        }
        if self.confirm_reset {
            let name = self.defs[self.mi].name;
            let mut close = false;
            let mut go = false;
            egui::Window::new("Reset inputs")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::new()
                        .fill(surf())
                        .stroke(Stroke::new(1.0_f32, p().line_strong))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(egui::Margin::same(18)),
                )
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("Reset all {name} inputs to their defaults?")).font(strong(14.0)));
                    ui.label(RichText::new("The current values are discarded. Saved projects are not affected.").size(12.0).color(mute()));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if primary(ui, "Reset inputs").clicked() {
                            go = true;
                        }
                        if tbtn(ui, "Cancel", false).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            close = true;
                        }
                    });
                });
            if go {
                let d = &self.defs[self.mi];
                self.st[self.mi].vals = d.fields.iter().map(|f| f.default).collect();
                self.sync_text(self.mi);
                self.saved.clear();
                self.run();
            }
            if go || close {
                self.confirm_reset = false;
            }
        }

        match self.page {
            Page::Workspace => {
                self.assistant_ui(ctx);
                if self.show_insp && self.inspector(ctx) {
                    self.saved.clear();
                    if self.input_errors().is_empty() {
                        self.run();
                    } else {
                        self.prefs_dirty = true;
                    }
                }
                if self.show_rail {
                    self.verification_rail(ctx);
                }
                self.center(ctx);
                self.ledger_force = None;
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
            Page::Standards => self.standards(ctx),
            Page::Settings => self.settings(ctx),
        }
        self.command_palette(ctx);
        if self.prefs_dirty {
            self.save_prefs();
        }
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 880.0])
            .with_min_inner_size([1100.0, 650.0])
            .with_title("StruCal")
            .with_icon(std::sync::Arc::new(egui::IconData { rgba: include_bytes!("../assets/icon.rgba").to_vec(), width: 64, height: 64 })),
        ..Default::default()
    };
    eframe::run_native("StruCal", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numbers_like_the_web_form() {
        assert_eq!(parse_number(" 12.5 "), Some(12.5));
        assert_eq!(parse_number("12,5"), Some(12.5));
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("-"), None);
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("inf"), None);
    }

    #[test]
    fn formats_round_trip() {
        for v in [0.0, 1.0, 0.85, 300.0, 12.75, -4.5] {
            assert_eq!(parse_number(&fmt_num(v)), Some(v));
        }
    }

    #[test]
    fn page_ids_round_trip() {
        for p in [Page::Workspace, Page::Project, Page::Account, Page::Standards, Page::Settings] {
            assert!(Page::from_id(p.id()) == p);
        }
    }
}
