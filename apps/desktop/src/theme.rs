//! Design tokens shared with the web design system (apps/web/app/globals.css).
//! Two themes: "drafting" (off-white drafting surface) and "obsidian" (workstation dark).
//! Square corners, 1px hairlines, no gradients; emerald = pass, amber = utilisation ≥ 0.85, crimson = fail.
use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Utilisation at or above which a passing check is flagged for review (amber).
pub const REVIEW: f64 = 0.85;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Drafting,
    Obsidian,
}

impl Theme {
    pub fn id(self) -> &'static str {
        match self {
            Theme::Drafting => "drafting",
            Theme::Obsidian => "obsidian",
        }
    }
    pub fn from_id(s: &str) -> Theme {
        if s == "obsidian" {
            Theme::Obsidian
        } else {
            Theme::Drafting
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Theme::Drafting => "Drafting (light)",
            Theme::Obsidian => "Obsidian (dark)",
        }
    }
}

pub struct Pal {
    pub bg: Color32,
    pub surface: Color32,
    pub surface2: Color32,
    pub surface3: Color32,
    pub sunken: Color32,
    pub line: Color32,
    pub line_strong: Color32,
    pub ink: Color32,
    pub ink2: Color32,
    pub ink3: Color32,
    pub ink4: Color32,
    pub signal: Color32,
    pub signal_bg: Color32,
    pub grid_minor: Color32,
    pub grid_major: Color32,
    pub pass: Color32,
    pub pass_bg: Color32,
    pub warn: Color32,
    pub warn_bg: Color32,
    pub fail: Color32,
    pub fail_bg: Color32,
    pub track: Color32,
    pub m_concrete: Color32,
    pub m_steel: Color32,
    pub m_timber: Color32,
    pub m_grain: Color32,
    pub m_soil: Color32,
}

const fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

static DRAFTING: Pal = Pal {
    bg: hex(0xf6f6f3),
    surface: hex(0xffffff),
    surface2: hex(0xf0f0eb),
    surface3: hex(0xe7e7e0),
    sunken: hex(0xfafaf7),
    line: hex(0xdcdcd5),
    line_strong: hex(0xb8b8af),
    ink: hex(0x0a0c0f),
    ink2: hex(0x2b2f35),
    ink3: hex(0x5b6068),
    ink4: hex(0x8e939a),
    signal: hex(0x1f4fe0),
    signal_bg: hex(0xe9eefc),
    grid_minor: hex(0xecece6),
    grid_major: hex(0xdadad2),
    pass: hex(0x047857),
    pass_bg: hex(0xe3f3ec),
    warn: hex(0xb45309),
    warn_bg: hex(0xfbf0dc),
    fail: hex(0xb91c1c),
    fail_bg: hex(0xfbe8e6),
    track: hex(0xe6e6df),
    m_concrete: hex(0xd8d8d1),
    m_steel: hex(0x2b2f35),
    m_timber: hex(0xe5d2ae),
    m_grain: hex(0x9d7a49),
    m_soil: hex(0xebe5d8),
};

static OBSIDIAN: Pal = Pal {
    bg: hex(0x090a0c),
    surface: hex(0x101215),
    surface2: hex(0x15181c),
    surface3: hex(0x1d2126),
    sunken: hex(0x0c0e11),
    line: hex(0x22262c),
    line_strong: hex(0x343a42),
    ink: hex(0xeceef1),
    ink2: hex(0xc7cbd1),
    ink3: hex(0x8b919a),
    ink4: hex(0x5d636c),
    signal: hex(0x6f96ff),
    signal_bg: hex(0x141d38),
    grid_minor: hex(0x14171b),
    grid_major: hex(0x1e2227),
    pass: hex(0x34d399),
    pass_bg: hex(0x0c2219),
    warn: hex(0xfbbf24),
    warn_bg: hex(0x2a200a),
    fail: hex(0xf2545b),
    fail_bg: hex(0x2e1214),
    track: hex(0x22262c),
    m_concrete: hex(0x353a40),
    m_steel: hex(0xc7cbd1),
    m_timber: hex(0x6b5634),
    m_grain: hex(0xb8955f),
    m_soil: hex(0x2a2620),
};

static DARK: AtomicBool = AtomicBool::new(false);

/// The active palette.
pub fn p() -> &'static Pal {
    if DARK.load(Ordering::Relaxed) {
        &OBSIDIAN
    } else {
        &DRAFTING
    }
}

pub fn current() -> Theme {
    if DARK.load(Ordering::Relaxed) {
        Theme::Obsidian
    } else {
        Theme::Drafting
    }
}

// Short accessors used throughout the UI.
pub fn bg() -> Color32 {
    p().bg
}
pub fn surf() -> Color32 {
    p().surface
}
pub fn head() -> Color32 {
    p().surface2
}
pub fn line() -> Color32 {
    p().line
}
pub fn tx() -> Color32 {
    p().ink
}
/// Text on an ink-filled control (inverse of the ink).
pub fn on_ink() -> Color32 {
    p().bg
}
pub fn mute() -> Color32 {
    p().ink3
}
pub fn blue() -> Color32 {
    p().signal
}
pub fn sel() -> Color32 {
    p().signal_bg
}
pub fn green() -> Color32 {
    p().pass
}
pub fn amber() -> Color32 {
    p().warn
}
pub fn amber_bg() -> Color32 {
    p().warn_bg
}
pub fn red() -> Color32 {
    p().fail
}
pub fn red_bg() -> Color32 {
    p().fail_bg
}

/// Masonry unit and mortar colours (material tokens --m-masonry / --m-mortar on the web).
pub fn masonry() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        hex(0x4a3a2e)
    } else {
        hex(0xe6cfb8)
    }
}
pub fn mortar() -> Color32 {
    if DARK.load(Ordering::Relaxed) {
        hex(0x2a221b)
    } else {
        hex(0xb8a48f)
    }
}

/// Status of a check: crimson when failing, amber at or above the review threshold, emerald otherwise.
pub fn status_color(pass: bool, util: f64) -> Color32 {
    if !pass {
        red()
    } else if util >= REVIEW {
        amber()
    } else {
        green()
    }
}

/// Embedded typefaces: Geist (neo-grotesque UI) and JetBrains Mono (tabular figures), both SIL OFL 1.1.
pub fn install_fonts(ctx: &egui::Context) {
    let mut f = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 6] = [
        ("geist", include_bytes!("../assets/fonts/Geist-Regular.ttf")),
        ("geist-medium", include_bytes!("../assets/fonts/Geist-Medium.ttf")),
        ("geist-semibold", include_bytes!("../assets/fonts/Geist-SemiBold.ttf")),
        ("jbm", include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")),
        ("jbm-medium", include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf")),
        ("jbm-semibold", include_bytes!("../assets/fonts/JetBrainsMono-SemiBold.ttf")),
    ];
    for (n, b) in faces {
        f.font_data.insert(n.into(), Arc::new(FontData::from_static(b)));
    }
    // Keep egui's bundled fonts as fallbacks for symbols (arrows, Greek, emoji) the primary faces lack.
    f.families.entry(FontFamily::Proportional).or_default().insert(0, "geist".into());
    f.families.entry(FontFamily::Monospace).or_default().insert(0, "jbm".into());
    f.families.insert(FontFamily::Name("strong".into()), vec!["geist-semibold".into(), "geist".into()]);
    f.families.insert(FontFamily::Name("medium".into()), vec!["geist-medium".into(), "geist".into()]);
    f.families.insert(FontFamily::Name("mono-strong".into()), vec!["jbm-semibold".into(), "jbm".into()]);
    f.families.insert(FontFamily::Name("mono-medium".into()), vec!["jbm-medium".into(), "jbm".into()]);
    // The named families need the fallbacks too.
    let fallback: Vec<String> = FontDefinitions::default().families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for name in ["strong", "medium", "mono-strong", "mono-medium"] {
        if let Some(v) = f.families.get_mut(&FontFamily::Name(name.into())) {
            v.extend(fallback.iter().cloned());
        }
    }
    ctx.set_fonts(f);
}

/// Semibold UI font.
pub fn strong(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("strong".into()))
}
/// Medium UI font.
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}
/// Monospace, tabular figures.
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}
/// Semibold monospace.
pub fn mono_strong(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("mono-strong".into()))
}

/// Switches the theme and rebuilds egui's visuals from the tokens.
pub fn apply(ctx: &egui::Context, t: Theme) {
    DARK.store(t == Theme::Obsidian, Ordering::Relaxed);
    let c = p();
    let mut v = if t == Theme::Obsidian { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = c.bg;
    v.window_fill = c.surface;
    v.extreme_bg_color = c.sunken;
    v.faint_bg_color = c.surface2;
    v.code_bg_color = c.surface2;
    v.window_stroke = Stroke::new(1.0_f32, c.line_strong);
    let sh = if t == Theme::Obsidian { 110 } else { 40 };
    v.window_shadow = egui::epaint::Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_black_alpha(sh) };
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 6], blur: 18, spread: 0, color: Color32::from_black_alpha(sh) };
    v.interact_cursor = Some(egui::CursorIcon::PointingHand);
    v.selection.bg_fill = c.signal_bg;
    v.selection.stroke = Stroke::new(1.0_f32, c.signal);
    v.hyperlink_color = c.signal;
    v.warn_fg_color = c.warn;
    v.error_fg_color = c.fail;
    // Structural panes stay square; controls, menus and windows take soft radii (enterprise finish).
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.override_text_color = None;
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(6);
        w.expansion = 0.0;
    }
    v.widgets.noninteractive.bg_fill = c.surface;
    v.widgets.noninteractive.weak_bg_fill = c.surface;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, c.line);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, c.ink2);
    v.widgets.inactive.bg_fill = c.sunken;
    v.widgets.inactive.weak_bg_fill = c.surface;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, c.line_strong);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, c.ink);
    v.widgets.hovered.bg_fill = c.surface2;
    v.widgets.hovered.weak_bg_fill = c.surface2;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, c.ink4);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, c.ink);
    v.widgets.active.bg_fill = c.surface3;
    v.widgets.active.weak_bg_fill = c.surface3;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, c.signal);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, c.ink);
    v.widgets.open.bg_fill = c.surface2;
    v.widgets.open.weak_bg_fill = c.surface2;
    v.widgets.open.bg_stroke = Stroke::new(1.0_f32, c.ink4);
    v.widgets.open.fg_stroke = Stroke::new(1.0_f32, c.ink);
    ctx.set_visuals(v);

    let mut st = (*ctx.style()).clone();
    st.spacing.item_spacing = egui::vec2(8.0, 6.0);
    st.spacing.button_padding = egui::vec2(10.0, 4.0);
    st.spacing.interact_size.y = 26.0;
    st.spacing.scroll = egui::style::ScrollStyle::thin();
    st.spacing.scroll.floating = true;
    st.spacing.scroll.bar_width = 8.0;
    // 160 ms state transitions for hover/active/open (egui animates widget visuals over this time).
    st.animation_time = 0.16;
    for (ts, sz) in [
        (egui::TextStyle::Body, 13.0),
        (egui::TextStyle::Button, 12.5),
        (egui::TextStyle::Small, 11.0),
        (egui::TextStyle::Monospace, 12.0),
        (egui::TextStyle::Heading, 20.0),
    ] {
        if let Some(f) = st.text_styles.get_mut(&ts) {
            f.size = sz;
        }
    }
    ctx.set_style(st);
}
