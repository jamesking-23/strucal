//! Loading and support diagrams (port of apps/web/app/workspace/LoadingDiagram.tsx). Each diagram states the model the
//! engine module uses and is annotated with input values only — never with computed results.
use eframe::egui::{Align2, Color32, Pos2, Rect, Shape, Stroke, StrokeKind};
use std::f32::consts::PI;

use crate::canvas::Sink;
use crate::theme::{blue, masonry, mono, mono_strong, mute, p as pal, red, surf, tx};

const W: f32 = 560.0;
const H: f32 = 300.0;

fn f(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{x:.0}")
    } else {
        let s = format!("{x:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

struct L<'a> {
    s: &'a Sink,
}
impl L<'_> {
    fn load(&self) -> Color32 {
        red()
    }
    fn line(&self, a: (f32, f32), b: (f32, f32), st: Stroke) {
        self.s.line_segment([Pos2::new(a.0, a.1), Pos2::new(b.0, b.1)], st);
    }
    fn tri(&self, pts: [(f32, f32); 3], fill: Color32) {
        self.s.add(Shape::convex_polygon(pts.iter().map(|p| Pos2::new(p.0, p.1)).collect(), fill, Stroke::NONE));
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, fill: Color32, edge: bool) {
        let r = Rect::from_min_size(Pos2::new(x, y), eframe::egui::vec2(w.max(0.0), h.max(0.0)));
        self.s.rect_filled(r, 0.0, fill);
        if edge {
            self.s.rect_stroke(r, 0.0, Stroke::new(1.3_f32, tx()), StrokeKind::Middle);
        }
    }
    /// Downward arrow with its tip at (x, y); `up` reverses it.
    fn arrow(&self, x: f32, y: f32, len: f32, up: bool) {
        let s = if up { -1.0 } else { 1.0 };
        self.line((x, y - s * len), (x, y - s * 6.0), Stroke::new(1.4_f32, self.load()));
        self.tri([(x, y), (x - 4.0, y - s * 8.0), (x + 4.0, y - s * 8.0)], self.load());
    }
    fn harrow(&self, x: f32, y: f32, len: f32, left: bool) {
        let s = if left { -1.0 } else { 1.0 };
        self.line((x - s * len, y), (x - s * 6.0, y), Stroke::new(1.4_f32, self.load()));
        self.tri([(x, y), (x - s * 8.0, y - 4.0), (x - s * 8.0, y + 4.0)], self.load());
    }
    fn txt(&self, x: f32, y: f32, t: &str, a: Align2, load: bool) {
        self.s.text(Pos2::new(x, y), a, t, if load { mono_strong(10.5) } else { mono(10.5) }, if load { self.load() } else { mute() });
    }
    fn udl(&self, x1: f32, x2: f32, y: f32, label: &str) {
        let k = ((x2 - x1) / 36.0).round().max(2.0) as i32;
        self.line((x1, y - 34.0), (x2, y - 34.0), Stroke::new(1.4_f32, self.load()));
        for i in 0..=k {
            self.arrow(x1 + (x2 - x1) * i as f32 / k as f32, y - 2.0, 32.0, false);
        }
        self.txt((x1 + x2) / 2.0, y - 46.0, label, Align2::CENTER_BOTTOM, true);
    }
    fn pin(&self, x: f32, y: f32) {
        let st = Stroke::new(1.3_f32, tx());
        self.s.add(Shape::convex_polygon(vec![Pos2::new(x, y), Pos2::new(x - 10.0, y + 16.0), Pos2::new(x + 10.0, y + 16.0)], surf(), st));
        self.line((x - 16.0, y + 16.0), (x + 16.0, y + 16.0), st);
        for d in [-12.0, -4.0, 4.0, 12.0] {
            self.line((x + d, y + 16.0), (x + d - 5.0, y + 22.0), st);
        }
    }
    fn roller(&self, x: f32, y: f32) {
        let st = Stroke::new(1.3_f32, tx());
        self.s.add(Shape::convex_polygon(vec![Pos2::new(x, y), Pos2::new(x - 10.0, y + 13.0), Pos2::new(x + 10.0, y + 13.0)], surf(), st));
        self.s.circle_stroke(Pos2::new(x - 5.0, y + 16.0), 3.0, st);
        self.s.circle_stroke(Pos2::new(x + 5.0, y + 16.0), 3.0, st);
        self.line((x - 16.0, y + 20.0), (x + 16.0, y + 20.0), st);
    }
    fn dim(&self, x1: f32, x2: f32, y: f32, label: &str) {
        let st = Stroke::new(0.9_f32, blue());
        self.line((x1, y - 6.0), (x1, y + 6.0), st);
        self.line((x2, y - 6.0), (x2, y + 6.0), st);
        self.line((x1, y), (x2, y), st);
        self.line((x1 - 4.0, y + 4.0), (x1 + 4.0, y - 4.0), Stroke::new(1.3_f32, blue()));
        self.line((x2 - 4.0, y + 4.0), (x2 + 4.0, y - 4.0), Stroke::new(1.3_f32, blue()));
        self.s.text(Pos2::new((x1 + x2) / 2.0, y + 10.0), Align2::CENTER_TOP, label, mono(10.5), blue());
    }
    fn vdim(&self, x: f32, y1: f32, y2: f32, label: &str) {
        let st = Stroke::new(0.9_f32, blue());
        self.line((x - 6.0, y1), (x + 6.0, y1), st);
        self.line((x - 6.0, y2), (x + 6.0, y2), st);
        self.line((x, y1), (x, y2), st);
        self.line((x - 4.0, y1 + 4.0), (x + 4.0, y1 - 4.0), Stroke::new(1.3_f32, blue()));
        self.line((x - 4.0, y2 + 4.0), (x + 4.0, y2 - 4.0), Stroke::new(1.3_f32, blue()));
        let g = self.s.layout_no_wrap(label.to_string(), mono(10.5), blue());
        let c = Pos2::new(x - 12.0, (y1 + y2) / 2.0);
        let pos = Pos2::new(c.x - g.size().y / 2.0, c.y + g.size().x / 2.0);
        self.s.add(eframe::egui::epaint::TextShape::new(pos, g, blue()).with_angle(-PI / 2.0));
    }
    /// Moment arc centred at (x, y), clockwise when `cw`, with a label at (lx, ly).
    fn moment(&self, x: f32, y: f32, r: f32, cw: bool, label: &str, lx: f32, ly: f32) {
        let (a1, a2) = if cw { (-150.0_f32, 150.0_f32) } else { (210.0_f32, -30.0_f32) };
        let n = 24;
        let pts: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = (a1 + (a2 - a1) * i as f32 / n as f32).to_radians();
                Pos2::new(x + r * a.cos(), y + r * a.sin())
            })
            .collect();
        let end = *pts.last().unwrap_or(&Pos2::new(x, y));
        self.s.add(Shape::line(pts, Stroke::new(1.4_f32, self.load())));
        self.s.circle_filled(end, 2.5, self.load());
        if !label.is_empty() {
            self.txt(lx, ly, label, Align2::LEFT_CENTER, true);
        }
    }
    fn note(&self, x: f32, y: f32, t: &str) {
        self.txt(x, y, t, Align2::LEFT_CENTER, false);
    }
    fn ground(&self, x1: f32, x2: f32, y: f32) {
        let st = Stroke::new(1.0_f32, pal().ink3);
        self.line((x1, y), (x2, y), st);
        let mut x = x1;
        while x + 12.0 <= x2 {
            self.line((x + 4.0, y), (x - 4.0, y + 8.0), st);
            x += 12.0;
        }
    }
    fn member(&self, a: (f32, f32), b: (f32, f32)) {
        self.line(a, b, Stroke::new(3.0_f32, tx()));
    }
    fn dashed(&self, a: (f32, f32), b: (f32, f32), c: Color32) {
        self.s.extend(Shape::dashed_line(&[Pos2::new(a.0, a.1), Pos2::new(b.0, b.1)], Stroke::new(1.3_f32, c), 6.0, 4.0));
    }
}

fn simple_span(l: &L, v: &dyn Fn(&str) -> f64, id: &str) {
    let (len, g, q) = (v("span"), v("gk"), v("qk"));
    let per = if id == "slab" || id == "stairs" { "kN/m²" } else { "kN/m" };
    let (x1, x2, y) = (70.0, 490.0, 160.0);
    l.udl(x1, x2, y, &format!("gk {} + qk {} {}", f(g), f(q), per));
    l.member((x1, y), (x2, y));
    l.pin(x1, y + 4.0);
    l.roller(x2, y + 4.0);
    if id == "steelbeam" {
        let lcr = v("lcr");
        let nr = if lcr > 0.0 && len > 0.0 { (len / lcr).floor().min(30.0) as i32 } else { 0 };
        for i in 1..nr {
            let x = x1 + (x2 - x1) * i as f32 / nr as f32;
            let st = Stroke::new(1.3_f32, blue());
            l.line((x, y - 8.0), (x, y + 8.0), st);
            l.line((x - 5.0, y - 8.0), (x + 5.0, y - 8.0), st);
        }
    }
    l.dim(x1, x2, y + 52.0, &format!("L = {} mm", f(len)));
    let note = match id {
        "slab" => "Simply supported single span · 1 m design strip".to_string(),
        "stairs" => "Simply supported flight · horizontal span · 1 m width".to_string(),
        "timber" => "Simply supported · rectangular section".to_string(),
        _ => format!("Simply supported · lateral restraints at Lcr = {} mm", f(v("lcr"))),
    };
    l.note(x1, y + 96.0, &note);
}

fn vertical(l: &L, v: &dyn Fn(&str) -> f64, id: &str) {
    let (x, y1, y2) = (280.0, 60.0, 240.0);
    let per = if id == "wall" { " kN/m" } else { " kN" };
    let len = if id == "timbercol" { v("lef_y") } else { v("len") };
    l.arrow(x, y1 - 4.0, 40.0, false);
    l.txt(x + 10.0, y1 - 30.0, &format!("N Ed = {}{}", f(v("n_ed")), per), Align2::LEFT_CENTER, true);
    l.member((x, y1), (x, y2));
    l.pin(x, y2);
    let st = Stroke::new(1.3_f32, tx());
    l.line((x - 14.0, y1 - 2.0), (x + 14.0, y1 - 2.0), st);
    l.s.circle_stroke(Pos2::new(x, y1), 3.0, st);
    l.line((x - 22.0, y1), (x - 8.0, y1), Stroke::new(1.3_f32, pal().ink3));
    l.line((x - 22.0, y2), (x - 8.0, y2), Stroke::new(1.3_f32, pal().ink3));
    match id {
        "column" | "wall" => {
            l.moment(x, y1 + 18.0, 16.0, true, &format!("M02 = {}", f(v("m02"))), x + 26.0, y1 + 22.0);
            l.moment(x, y2 - 18.0, 16.0, false, &format!("M01 = {}", f(v("m01"))), x + 26.0, y2 - 14.0);
        }
        "beamcol" => {
            l.moment(x, y1 + 18.0, 16.0, true, &format!("My,Ed = {} kNm", f(v("my_ed"))), x + 26.0, y1 + 22.0);
            l.moment(x, y2 - 18.0, 16.0, v("psi") >= 0.0, &format!("ψ·My,Ed (ψ = {})", f(v("psi"))), x + 26.0, y2 - 14.0);
        }
        "timbercol" => l.note(x + 26.0, (y1 + y2) / 2.0, &format!("My,d {} · Mz,d {} kNm", f(v("my_ed")), f(v("mz_ed")))),
        _ => {}
    }
    l.vdim(x - 70.0, y1, y2, &format!("{} = {} mm", if id == "timbercol" { "lef,y" } else { "l" }, f(len)));
    let note = match id {
        "steelcol" => format!("Effective-length factors ky = {}, kz = {}", f(v("ky")), f(v("kz"))),
        "beamcol" => format!("Effective-length factors ky = {}, kz = {}, kLT = {} · linear moment", f(v("ky")), f(v("kz")), f(v("klt"))),
        "timbercol" => "Buckling about both axes".to_string(),
        _ => format!("Braced · l0 = β·l with β = {}{}", f(v("beta")), if id == "wall" { " · per metre run" } else { "" }),
    };
    l.note(40.0, H - 16.0, &note);
}

fn footing(l: &L, v: &dyn Fn(&str) -> f64, id: &str) {
    let (y, x1, x2, depth) = (150.0, 120.0, 440.0, 40.0);
    let strip = id == "strip";
    let cw = if strip { 44.0 } else { 36.0 };
    l.rect(280.0 - cw / 2.0, y - 80.0, cw, 80.0, pal().m_concrete, true);
    l.arrow(280.0, y - 84.0, 40.0, false);
    l.txt(
        292.0,
        y - 108.0,
        &format!("N Ed = {} {} · N k = {}", f(v("n_ed")), if strip { "kN/m" } else { "kN" }, f(v("n_k"))),
        Align2::LEFT_CENTER,
        true,
    );
    l.rect(x1, y, x2 - x1, depth, pal().m_concrete, true);
    for i in 0..9 {
        l.arrow(x1 + 12.0 + (x2 - x1 - 24.0) * i as f32 / 8.0, y + depth + 2.0, 26.0, true);
    }
    l.txt(
        280.0,
        y + depth + 46.0,
        &format!("uniform bearing pressure · allowable q = {} kPa", f(v("q_allow"))),
        Align2::CENTER_CENTER,
        false,
    );
    l.ground(60.0, 500.0, y);
    l.note(
        40.0,
        H - 12.0,
        &format!(
            "{} · allowable bearing pressure from the geotechnical engineer",
            if strip { "Concentric wall line load per metre" } else { "Concentric axial load" }
        ),
    );
}

fn raft(l: &L, v: &dyn Fn(&str) -> f64) {
    let nc = (v("n_cols").round() as i32).clamp(2, 12);
    let (x1, x2, y) = (50.0, 510.0, 160.0);
    for i in 0..nc {
        let x = x1 + 30.0 + (x2 - x1 - 60.0) * i as f32 / (nc - 1) as f32;
        l.rect(x - 8.0, y - 50.0, 16.0, 50.0, pal().m_concrete, true);
        l.arrow(x, y - 54.0, 30.0, false);
    }
    l.txt(280.0, y - 92.0, &format!("{} × N Ed = {} kN at {} mm", nc, f(v("n_ed")), f(v("spacing"))), Align2::CENTER_CENTER, true);
    l.rect(x1, y, x2 - x1, 30.0, pal().m_concrete, true);
    for i in 0..12 {
        l.arrow(x1 + 10.0 + (x2 - x1 - 20.0) * i as f32 / 11.0, y + 32.0, 24.0, true);
    }
    l.txt(
        280.0,
        y + 74.0,
        &format!("uniform ground pressure · allowable q = {} kPa · strip width {} mm", f(v("q_allow")), f(v("width"))),
        Align2::CENTER_CENTER,
        false,
    );
    l.note(40.0, H - 12.0, "One column-line strip · equal symmetric column loads · rigid method");
}

fn retaining(l: &L, v: &dyn Fn(&str) -> f64) {
    let (hs, tb, bw, lt, ts, hw) = (v("hs") as f32, v("tb") as f32, v("bw") as f32, v("lt") as f32, v("ts") as f32, v("hw") as f32);
    let sc = (200.0 / (hs + tb).max(1.0)).min(300.0 / bw.max(1.0));
    let (ox, base) = (90.0, 250.0);
    let top = base - (hs + tb) * sc;
    let (stem_x, heel_x, end_x) = (ox + lt * sc, ox + (lt + ts) * sc, ox + bw * sc);
    let tri = 70.0;
    l.rect(heel_x, top, end_x - heel_x, hs * sc, pal().m_soil, false);
    l.rect(ox, base - tb * sc, bw * sc, tb * sc, pal().m_concrete, true);
    l.rect(stem_x, top, ts * sc, hs * sc, pal().m_concrete, true);
    if hw > 0.0 {
        l.dashed((heel_x, base - hw * sc), (end_x + tri + 20.0, base - hw * sc), blue());
    }
    l.s.add(Shape::convex_polygon(
        vec![Pos2::new(end_x + 6.0, top), Pos2::new(end_x + 6.0, base), Pos2::new(end_x + 6.0 + tri, base)],
        pal().fail_bg,
        Stroke::new(1.0_f32, l.load()),
    ));
    for i in 0..5 {
        let y = top + (base - top) * (i + 1) as f32 / 6.0;
        l.harrow(end_x + 6.0, y, tri * (i + 1) as f32 / 6.0 + 6.0, true);
    }
    if v("q_sur") > 0.0 {
        for i in 0..4 {
            l.arrow(heel_x + 10.0 + (end_x - heel_x - 20.0) * i as f32 / 3.0, top - 2.0, 22.0, false);
        }
    }
    l.txt(end_x + 10.0, top - 30.0, &format!("q = {} kPa", f(v("q_sur"))), Align2::LEFT_CENTER, true);
    l.txt(end_x + tri + 14.0, base - 8.0, "Ka·γ·H", Align2::LEFT_CENTER, true);
    l.ground(40.0, ox, base - tb * sc);
    l.note(
        40.0,
        H - 12.0,
        &format!(
            "Rankine active pressure · γ = {} kN/m³ · φ′ = {}° · horizontal backfill{}",
            f(v("gamma")),
            f(v("phi")),
            if hw > 0.0 { format!(" · water {} mm above base", f(hw as f64)) } else { String::new() }
        ),
    );
}

fn bolted(l: &L, v: &dyn Fn(&str) -> f64) {
    let y = 130.0;
    l.rect(60.0, y - 14.0, 300.0, 14.0, surf(), true);
    l.rect(200.0, y, 300.0, 14.0, pal().m_concrete, true);
    let n1 = (v("n1").round() as i32).clamp(1, 12);
    for i in 0..n1 {
        l.rect(226.0 + i as f32 * 34.0, y - 22.0, 8.0, 44.0, pal().ink2, false);
    }
    l.harrow(60.0, y - 7.0, 44.0, true);
    l.harrow(500.0, y + 7.0, 44.0, false);
    l.txt(24.0, y - 26.0, &format!("N Ed = {} kN", f(v("n_ed"))), Align2::LEFT_CENTER, true);
    l.txt(430.0, y + 40.0, "N Ed", Align2::LEFT_CENTER, true);
    l.note(
        40.0,
        H - 36.0,
        &format!(
            "{} × {} M{} grade {} · {} shear plane(s)",
            v("n1").round(),
            v("n2").round(),
            f(v("d")),
            f(v("grade")),
            v("planes").round()
        ),
    );
    l.note(40.0, H - 14.0, "Concentric tension · bolts in shear and bearing · Category A (non-preloaded)");
}

fn weld(l: &L, v: &dyn Fn(&str) -> f64) {
    let y = 150.0;
    l.rect(100.0, y - 16.0, 360.0, 16.0, surf(), true);
    l.rect(100.0, y, 360.0, 16.0, pal().m_concrete, true);
    l.tri([(100.0, y + 16.0), (100.0, y + 30.0), (114.0, y + 16.0)], tx());
    l.arrow(280.0, y - 18.0, 50.0, false);
    l.txt(290.0, y - 54.0, &format!("F⊥ = {} kN", f(v("f_perp"))), Align2::LEFT_CENTER, true);
    l.harrow(470.0, y - 8.0, 50.0, false);
    l.txt(380.0, y - 24.0, &format!("F∥ = {} kN", f(v("f_par"))), Align2::LEFT_CENTER, true);
    l.dim(100.0, 460.0, y + 52.0, &format!("Lw = {} mm · a = {} mm", f(v("lw")), f(v("a"))));
    l.note(40.0, H - 12.0, "Equal-leg fillet welds · directional and simplified methods");
}

fn actions(l: &L, v: &dyn Fn(&str) -> f64) {
    let items = [("Gk", v("gk")), ("Qk,1", v("q1")), ("Qk,2", v("q2")), ("Qk,3", v("q3"))];
    let max = items.iter().map(|x| x.1.abs()).fold(1.0, f64::max);
    for (i, (k, val)) in items.iter().enumerate() {
        let h = (val.abs() / max) as f32 * 170.0;
        let x = 90.0 + i as f32 * 110.0;
        l.rect(x, 230.0 - h, 56.0, h, if i == 0 { pal().ink3 } else { l.load().gamma_multiply(0.75) }, false);
        l.txt(x + 28.0, 250.0, k, Align2::CENTER_CENTER, false);
        l.txt(x + 28.0, 222.0 - h, &f(*val), Align2::CENTER_BOTTOM, true);
    }
    l.line((60.0, 230.0), (520.0, 230.0), Stroke::new(1.0_f32, tx()));
    l.note(40.0, H - 14.0, "Characteristic action effects in a common unit · combined by EN 1990 Eq 6.10, 6.10a/b and SLS");
}

fn section_actions(l: &L, v: &dyn Fn(&str) -> f64) {
    let y = 140.0;
    l.member((140.0, y), (420.0, y));
    l.moment(140.0, y, 22.0, false, &format!("M Ed = {} kNm (sagging)", f(v("m_ed"))), 120.0, y - 40.0);
    l.moment(420.0, y, 22.0, true, "", 0.0, 0.0);
    l.arrow(140.0, y + 4.0, 36.0, true);
    l.txt(150.0, y + 46.0, &format!("V Ed = {} kN", f(v("v_ed"))), Align2::LEFT_CENTER, true);
    l.note(40.0, H - 54.0, "The module verifies a section for design actions supplied from the frame analysis.");
    l.note(
        40.0,
        H - 34.0,
        &format!(
            "Serviceability: effective span l = {} mm, K = {} (Table 7.4N), M qp = {} kNm.",
            f(v("span")),
            f(v("k_sys")),
            f(v("m_qp"))
        ),
    );
    l.note(40.0, H - 14.0, "Moments and shear are not derived from the span: enter them from the analysis.");
}

fn tension(l: &L, v: &dyn Fn(&str) -> f64, id: &str) {
    let (y, x1, x2) = (140.0, 120.0, 440.0);
    let timber = id == "timbertension";
    let hh = if timber { 28.0 } else { 16.0 };
    l.rect(x1, y - hh / 2.0, x2 - x1, hh, if timber { pal().m_timber } else { surf() }, true);
    l.harrow(x1 - 48.0, y, 40.0, true);
    l.harrow(x2 + 48.0, y, 40.0, false);
    let nm = if timber { "N t,d" } else { "N Ed" };
    l.txt(x1 - 90.0, y - 18.0, &format!("{} = {} kN", nm, f(v("n_ed"))), Align2::LEFT_CENTER, true);
    l.txt(x2 + 6.0, y - 18.0, nm, Align2::LEFT_CENTER, true);
    if timber && v("my_ed") > 0.0 {
        l.moment(280.0, y, 26.0, true, &format!("My,d = {} kNm", f(v("my_ed"))), 300.0, y - 34.0);
    }
    if !timber {
        let nh = (v("n_holes").round() as i32).clamp(0, 12);
        for i in 0..nh {
            l.s.circle_filled(Pos2::new(x2 - 30.0, y - 6.0 + 12.0 * (i as f32 + 0.5) / nh.max(1) as f32), 2.0, pal().ink2);
        }
        l.dim(x1, x2, y + 46.0, &format!("L = {} mm", f(v("len"))));
        l.note(40.0, H - 34.0, &format!("Critical section through {} hole(s) of d0 = {} mm", v("n_holes").round(), f(v("d0"))));
        l.note(40.0, H - 14.0, "Concentric axial tension · connection checked separately");
    } else {
        l.note(40.0, H - 34.0, &format!("Net section · holes or notches remove {} mm²", f(v("a_hole"))));
        l.note(40.0, H - 14.0, "Concentric axial tension with optional major-axis bending");
    }
}

fn masonry_wall(l: &L, v: &dyn Fn(&str) -> f64) {
    let (x, y1, y2, t) = (280.0, 70.0, 240.0, 30.0);
    let (e, tw) = (v("e_load"), v("t").max(1.0));
    let ex = x + (e / tw) as f32 * t;
    l.rect(x - 70.0, y1 - 12.0, 140.0, 12.0, pal().m_concrete, true);
    l.rect(x - 70.0, y2, 140.0, 12.0, pal().m_concrete, true);
    l.rect(x - t / 2.0, y1, t, y2 - y1, masonry(), true);
    l.dashed((x, y1 - 30.0), (x, y2 + 30.0), pal().ink4);
    l.arrow(ex, y1 - 14.0, 40.0, false);
    l.txt(ex + 10.0, y1 - 40.0, &format!("N Ed = {} kN/m · e = {} mm", f(v("n_ed")), f(e)), Align2::LEFT_CENTER, true);
    l.vdim(x - 110.0, y1, y2, &format!("h = {} mm", f(v("h"))));
    l.note(x + 30.0, (y1 + y2) / 2.0, &format!("hef = ρn·h, ρn = {}", f(v("rho_n"))));
    l.note(40.0, H - 14.0, "Single-leaf wall · restrained top and bottom by floors · capacity reduction at top and mid-height");
}

fn punch(l: &L, v: &dyn Fn(&str) -> f64) {
    let (y, x, cw, h) = (130.0, 280.0, 60.0, 26.0);
    let d = (v("h") - v("cover") - v("bar_y")).max(1.0);
    let two = ((2.0 * d / v("cx").max(1.0)) as f32 * cw).min(120.0);
    let pos = v("position").round() as i32;
    let left = if pos == 1 { 60.0 } else { x - cw / 2.0 };
    l.rect(left, y, 500.0 - left, h, pal().m_concrete, true);
    l.rect(x - cw / 2.0, y + h, cw, 80.0, pal().m_concrete, true);
    l.arrow(x, y + h + 82.0, 30.0, true);
    l.txt(
        x + 12.0,
        y + h + 104.0,
        &format!(
            "V Ed = {} kN · β = {}",
            f(v("v_ed")),
            match pos {
                2 => "1.4",
                3 => "1.5",
                _ => "1.15",
            }
        ),
        Align2::LEFT_CENTER,
        true,
    );
    l.dashed((x + cw / 2.0 + two, y - 8.0), (x + cw / 2.0 + two, y + h + 8.0), blue());
    if pos == 1 {
        l.dashed((x - cw / 2.0 - two, y - 8.0), (x - cw / 2.0 - two, y + h + 8.0), blue());
    }
    l.txt(x + cw / 2.0 + two + 6.0, y - 10.0, "u1 at 2d", Align2::LEFT_CENTER, false);
    l.note(
        40.0,
        H - 14.0,
        &format!(
            "{} column · flat slab of uniform depth · β from Figure 6.21N",
            match pos {
                2 => "Edge",
                3 => "Corner",
                _ => "Internal",
            }
        ),
    );
}

/// Records the loading diagram for module `id`; `v` reads an input value by key. Returns the drawing extent.
pub fn draw(sink: &Sink, id: &str, v: &dyn Fn(&str) -> f64) -> Option<(f32, f32)> {
    let l = L { s: sink };
    match id {
        "steelbeam" | "timber" | "slab" | "stairs" => simple_span(&l, v, id),
        "column" | "steelcol" | "timbercol" | "wall" | "beamcol" => vertical(&l, v, id),
        "pad" | "strip" => footing(&l, v, id),
        "raft" => raft(&l, v),
        "retaining" => retaining(&l, v),
        "bolted" => bolted(&l, v),
        "weld" => weld(&l, v),
        "combos" => actions(&l, v),
        "beam" => section_actions(&l, v),
        "steeltension" | "timbertension" => tension(&l, v, id),
        "masonry" => masonry_wall(&l, v),
        "punching" => punch(&l, v),
        _ => return None,
    }
    Some((W, H))
}
