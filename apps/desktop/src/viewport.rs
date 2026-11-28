//! Section viewport: drafting-grid canvas with a dimensioned, proportional schematic of the member.
//! Geometry mirrors apps/web/app/workspace/SectionDrawing.tsx so both editions draw the same thing.
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind};
use strucal_engine::module::Drawing;

use crate::{BLUE, MUTE, TX};

const CONCRETE: Color32 = Color32::from_rgb(217, 217, 211);
const STEEL: Color32 = Color32::from_rgb(58, 58, 56);
const TIMBER: Color32 = Color32::from_rgb(231, 213, 179);
const GRAIN: Color32 = Color32::from_rgb(160, 125, 76);
const SOIL: Color32 = Color32::from_rgb(236, 231, 220);
const GRID_MINOR: Color32 = Color32::from_rgb(236, 236, 232);
const GRID_MAJOR: Color32 = Color32::from_rgb(220, 220, 214);
const M: f32 = 56.0;

fn fit(w: f64, h: f64, max_w: f64, max_h: f64) -> f32 {
    (max_w / w.max(1.0)).min(max_h / h.max(1.0)) as f32
}
fn fmt(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

/// Painter in drawing coordinates (like the web SVG viewBox), offset to the canvas origin.
struct Pen<'a> {
    p: &'a egui::Painter,
    o: Pos2,
    dims: bool,
}
impl Pen<'_> {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.o.x + x, self.o.y + y)
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, fill: Color32, edge: bool) {
        let r = Rect::from_min_size(self.at(x, y), egui::vec2(w.max(0.0), h.max(0.0)));
        self.p.rect_filled(r, 0.0, fill);
        if edge {
            self.p.rect_stroke(r, 0.0, Stroke::new(1.4_f32, TX), StrokeKind::Middle);
        }
    }
    fn dashed_rect(&self, x: f32, y: f32, w: f32, h: f32) {
        let pts = [self.at(x, y), self.at(x + w, y), self.at(x + w, y + h), self.at(x, y + h), self.at(x, y)];
        for s in pts.windows(2) {
            self.p.extend(Shape::dashed_line(&[s[0], s[1]], Stroke::new(1.0_f32, MUTE), 3.0, 2.0));
        }
    }
    fn dot(&self, x: f32, y: f32, r: f32, fill: Color32) {
        self.p.circle_filled(self.at(x, y), r, fill);
    }
    fn ring(&self, x: f32, y: f32, r: f32) {
        self.p.circle_filled(self.at(x, y), r, Color32::WHITE);
        self.p.circle_stroke(self.at(x, y), r, Stroke::new(1.3_f32, TX));
    }
    fn axis(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.p.extend(Shape::dashed_line(&[self.at(x1, y1), self.at(x2, y2)], Stroke::new(0.8_f32, MUTE), 10.0, 4.0));
    }
    fn note(&self, x: f32, y: f32, t: &str, a: Align2) {
        self.p.text(self.at(x, y), a, t, FontId::monospace(10.5), MUTE);
    }
    /// Horizontal dimension between x1 and x2 at member edge y, offset below (off > 0) or above (off < 0).
    fn dim_h(&self, x1: f32, x2: f32, y: f32, off: f32, label: &str) {
        if !self.dims {
            return;
        }
        let s = Stroke::new(0.9_f32, BLUE);
        let yy = y + off;
        let dir = off.signum();
        for x in [x1, x2] {
            self.p.line_segment([self.at(x, y + dir * 4.0), self.at(x, yy + dir * 4.0)], Stroke::new(0.7_f32, BLUE.gamma_multiply(0.7)));
            self.p.line_segment([self.at(x - 4.0, yy + 4.0), self.at(x + 4.0, yy - 4.0)], Stroke::new(1.3_f32, BLUE));
        }
        self.p.line_segment([self.at(x1, yy), self.at(x2, yy)], s);
        let ty = if off > 0.0 { yy + 9.0 } else { yy - 9.0 };
        self.p.text(self.at((x1 + x2) / 2.0, ty), Align2::CENTER_CENTER, label, FontId::monospace(10.5), BLUE);
    }
    /// Vertical dimension between y1 and y2 at member edge x, offset left (off < 0) or right (off > 0).
    fn dim_v(&self, y1: f32, y2: f32, x: f32, off: f32, label: &str) {
        if !self.dims {
            return;
        }
        let xx = x + off;
        let dir = off.signum();
        for y in [y1, y2] {
            self.p.line_segment([self.at(x + dir * 4.0, y), self.at(xx + dir * 4.0, y)], Stroke::new(0.7_f32, BLUE.gamma_multiply(0.7)));
            self.p.line_segment([self.at(xx - 4.0, y + 4.0), self.at(xx + 4.0, y - 4.0)], Stroke::new(1.3_f32, BLUE));
        }
        self.p.line_segment([self.at(xx, y1), self.at(xx, y2)], Stroke::new(0.9_f32, BLUE));
        let galley = self.p.layout_no_wrap(label.to_string(), FontId::monospace(10.5), BLUE);
        let c = self.at(xx + if off > 0.0 { 9.0 } else { -9.0 }, (y1 + y2) / 2.0);
        let pos = Pos2::new(c.x - galley.size().y / 2.0, c.y + galley.size().x / 2.0);
        self.p.add(egui::epaint::TextShape::new(pos, galley, BLUE).with_angle(-std::f32::consts::FRAC_PI_2));
    }
}

/// Size of the drawing in drawing units (px), matching the web viewBox.
fn extent(d: &Drawing) -> (f32, f32) {
    match d {
        Drawing::Beam { b, h, .. } | Drawing::Column { b, h, .. } => {
            let sc = fit(*b, *h, 220.0, 260.0);
            (*b as f32 * sc + 2.0 * M, *h as f32 * sc + 24.0 + M)
        }
        Drawing::Slab { h, .. } => {
            let sc = fit(1000.0, *h, 300.0, 140.0);
            (1000.0 * sc + 2.0 * M, *h as f32 * sc + 24.0 + M)
        }
        Drawing::Pad { bx, by, .. } => {
            let sc = fit(*bx, *by, 220.0, 220.0);
            (*bx as f32 * sc + 2.0 * M, *by as f32 * sc + 24.0 + M)
        }
        Drawing::Bolts { w, n1, e1, p1, .. } => {
            let len = 2.0 * e1 + (*n1 as f64 - 1.0) * p1;
            let sc = fit(len, *w, 280.0, 180.0);
            (len as f32 * sc + 2.0 * M, *w as f32 * sc + 40.0 + M)
        }
        Drawing::Weld { .. } => (260.0 + 2.0 * M, 150.0),
        Drawing::Raft { l, w, .. } => {
            let sc = fit(*l, *w, 300.0, 160.0);
            (*l as f32 * sc + 2.0 * M, *w as f32 * sc + 24.0 + M)
        }
        Drawing::Retain { hs, tb, b, .. } => {
            let sc = fit(*b, hs + tb, 240.0, 240.0);
            (*b as f32 * sc + 2.0 * M, (*hs + *tb) as f32 * sc + 24.0 + M)
        }
        Drawing::ISection { h, b, .. } | Drawing::Rect { b, h } => {
            let sc = fit(*b, *h, 200.0, 260.0);
            (*b as f32 * sc + 2.0 * M, *h as f32 * sc + 24.0 + M)
        }
        Drawing::Wall { h, .. } => {
            let sc = fit(*h, 1000.0, 160.0, 260.0);
            (*h as f32 * sc + 2.0 * M + 80.0, 1000.0 * sc + 24.0 + M)
        }
        Drawing::Strip { width, h, .. } => {
            let sc = fit(*width, h + 600.0, 280.0, 220.0);
            (*width as f32 * sc + 2.0 * M, (600.0 + *h as f32) * sc + 24.0 + M)
        }
    }
}

fn valid(d: &Drawing) -> bool {
    let v: Vec<f64> = match d {
        Drawing::Beam { b, h, cover, link_dia, bar_dia, .. } => vec![*b, *h, *cover, *link_dia, *bar_dia],
        Drawing::Column { b, h, cover, link_dia, bar_dia, .. } => vec![*b, *h, *cover, *link_dia, *bar_dia],
        Drawing::Slab { h, cover, bar_dia, spacing } | Drawing::Wall { h, cover, bar_dia, spacing } => vec![*h, *cover, *bar_dia, *spacing],
        Drawing::Pad { bx, by, cx, cy, h } => vec![*bx, *by, *cx, *cy, *h],
        Drawing::Bolts { w, e1, p1, e2, p2, d, .. } => vec![*w, *e1, *p1, *e2, *p2, *d],
        Drawing::Weld { lw, a } => vec![*lw, *a],
        Drawing::Raft { l, w, spacing, edge, .. } => vec![*l, *w, *spacing, *edge],
        Drawing::Retain { hs, tb, ts, lt, b } => vec![*hs, *tb, *ts, *lt, *b],
        Drawing::ISection { h, b, tw, tf } => vec![*h, *b, *tw, *tf],
        Drawing::Rect { b, h } => vec![*b, *h],
        Drawing::Strip { width, t, h } => vec![*width, *t, *h],
    };
    v.iter().all(|x| x.is_finite() && *x >= 0.0)
}

fn paint_grid(p: &egui::Painter, r: Rect) {
    p.rect_filled(r, 0.0, Color32::from_rgb(251, 251, 250));
    let mut x = r.left();
    let mut i = 0;
    while x <= r.right() {
        p.line_segment(
            [Pos2::new(x, r.top()), Pos2::new(x, r.bottom())],
            Stroke::new(1.0_f32, if i % 5 == 0 { GRID_MAJOR } else { GRID_MINOR }),
        );
        x += 16.0;
        i += 1;
    }
    let mut y = r.top();
    i = 0;
    while y <= r.bottom() {
        p.line_segment(
            [Pos2::new(r.left(), y), Pos2::new(r.right(), y)],
            Stroke::new(1.0_f32, if i % 5 == 0 { GRID_MAJOR } else { GRID_MINOR }),
        );
        y += 16.0;
        i += 1;
    }
}

/// Paints the viewport (grid + drawing) into a canvas of the given height. `None` shows an empty-state message.
pub fn paint(ui: &mut egui::Ui, d: Option<&Drawing>, empty: &str, dims: bool, height: f32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::hover());
    let p = ui.painter_at(r);
    paint_grid(&p, r);
    let Some(d) = d.filter(|d| valid(d)) else {
        p.text(r.center(), Align2::CENTER_CENTER, empty, FontId::proportional(13.0), MUTE);
        return;
    };
    let (w, h) = extent(d);
    let o = Pos2::new(r.center().x - w / 2.0, r.center().y - h / 2.0);
    let pen = Pen { p: &p, o, dims };
    draw(&pen, d);
    p.text(
        Pos2::new(r.left() + 10.0, r.bottom() - 10.0),
        Align2::LEFT_BOTTOM,
        "schematic · proportional · mm",
        FontId::monospace(10.0),
        MUTE,
    );
}

fn draw(pen: &Pen, d: &Drawing) {
    match d {
        Drawing::Beam { b, h, cover, link_dia, bar_dia, n_bars } | Drawing::Column { b, h, cover, link_dia, bar_dia, n_face: n_bars } => {
            let is_beam = matches!(d, Drawing::Beam { .. });
            let sc = fit(*b, *h, 220.0, 260.0);
            let (w, hh, ox, oy) = (*b as f32 * sc, *h as f32 * sc, M, 24.0);
            let off = (cover + link_dia / 2.0) as f32 * sc;
            let ed = (cover + link_dia + bar_dia / 2.0) as f32 * sc;
            let br = (*bar_dia as f32 * sc / 2.0).max(2.4);
            let n = (*n_bars).max(2);
            pen.rect(ox, oy, w, hh, CONCRETE, true);
            pen.dashed_rect(ox + off, oy + off, w - 2.0 * off, hh - 2.0 * off);
            let rows: Vec<f32> = if is_beam { vec![oy + hh - ed] } else { vec![oy + ed, oy + hh - ed] };
            for y in &rows {
                for i in 0..n {
                    pen.dot(ox + ed + (w - 2.0 * ed) * i as f32 / (n - 1) as f32, *y, br, TX);
                }
            }
            if is_beam {
                for i in 0..2 {
                    pen.dot(ox + ed + (w - 2.0 * ed) * i as f32, oy + ed, (6.0 * sc).max(2.2), MUTE);
                }
            }
            pen.dim_h(ox, ox + w, oy + hh, 22.0, &format!("b = {}", fmt(*b)));
            pen.dim_v(oy, oy + hh, ox, -22.0, &format!("h = {}", fmt(*h)));
            if pen.dims {
                pen.note(
                    ox + w + 8.0,
                    rows[rows.len() - 1],
                    &format!("{n}φ{}{}", fmt(*bar_dia), if is_beam { "" } else { "/face" }),
                    Align2::LEFT_CENTER,
                );
            }
        }
        Drawing::Slab { h, cover, bar_dia, spacing } => {
            let sc = fit(1000.0, *h, 300.0, 140.0);
            let (w, hh, ox, oy) = (1000.0 * sc, *h as f32 * sc, M, 24.0);
            pen.rect(ox, oy, w, hh, CONCRETE, true);
            if *spacing > 0.0 {
                let mut x = spacing / 2.0;
                let mut k = 0;
                while x < 1000.0 && k < 200 {
                    pen.dot(ox + x as f32 * sc, oy + hh - (cover + bar_dia / 2.0) as f32 * sc, (*bar_dia as f32 * sc / 2.0).max(2.2), TX);
                    x += spacing;
                    k += 1;
                }
            }
            pen.dim_v(oy, oy + hh, ox, -22.0, &format!("h = {}", fmt(*h)));
            pen.dim_h(ox, ox + w, oy + hh, 22.0, "1000 (design strip)");
            if pen.dims {
                pen.note(ox + w / 2.0, oy - 8.0, &format!("φ{} @ {} bottom", fmt(*bar_dia), fmt(*spacing)), Align2::CENTER_BOTTOM);
            }
        }
        Drawing::Pad { bx, by, cx, cy, h } => {
            let sc = fit(*bx, *by, 220.0, 220.0);
            let (w, hh, ox, oy) = (*bx as f32 * sc, *by as f32 * sc, M, 24.0);
            pen.rect(ox, oy, w, hh, CONCRETE, true);
            pen.rect(ox + (w - *cx as f32 * sc) / 2.0, oy + (hh - *cy as f32 * sc) / 2.0, *cx as f32 * sc, *cy as f32 * sc, STEEL, false);
            pen.axis(ox + w / 2.0, oy - 8.0, ox + w / 2.0, oy + hh + 8.0);
            pen.axis(ox - 8.0, oy + hh / 2.0, ox + w + 8.0, oy + hh / 2.0);
            pen.dim_h(ox, ox + w, oy + hh, 22.0, &format!("B = {}", fmt(*bx)));
            pen.dim_v(oy, oy + hh, ox + w, 22.0, &format!("L = {}", fmt(*by)));
            if pen.dims {
                pen.note(ox + 4.0, oy - 8.0, &format!("plan · h = {}", fmt(*h)), Align2::LEFT_BOTTOM);
            }
        }
        Drawing::Bolts { w, n1, n2, e1, p1, e2: _, p2, d } => {
            let len = 2.0 * e1 + (*n1 as f64 - 1.0) * p1;
            let sc = fit(len, *w, 280.0, 180.0);
            let (ww, hh, ox, oy) = (len as f32 * sc, *w as f32 * sc, M, 40.0);
            let y0 = oy + ((w - (*n2 as f64 - 1.0) * p2) / 2.0) as f32 * sc;
            pen.rect(ox, oy, ww, hh, Color32::WHITE, true);
            for i in 0..(*n1).min(200) {
                for j in 0..(*n2).min(200) {
                    pen.ring(ox + (e1 + i as f64 * p1) as f32 * sc, y0 + (j as f64 * p2) as f32 * sc, (*d as f32 * sc / 2.0).max(3.0));
                }
            }
            pen.dim_h(ox, ox + *e1 as f32 * sc, oy, -14.0, &fmt(*e1));
            if *n1 > 1 {
                pen.dim_h(ox + *e1 as f32 * sc, ox + (e1 + p1) as f32 * sc, oy, -14.0, &fmt(*p1));
            }
            pen.dim_h(ox, ox + ww, oy + hh, 22.0, &fmt(len));
            pen.dim_v(oy, oy + hh, ox, -22.0, &format!("w = {}", fmt(*w)));
            if pen.dims {
                pen.note(ox + ww + 8.0, oy + hh / 2.0, &format!("{n1}×{n2} M{}", fmt(*d)), Align2::LEFT_CENTER);
            }
        }
        Drawing::Weld { lw, a } => {
            let (w, ox, oy, t) = (260.0, M, 40.0, 18.0);
            pen.rect(ox, oy, w, t, Color32::WHITE, true);
            pen.rect(ox, oy + t, w, t, CONCRETE, true);
            pen.p.add(Shape::convex_polygon(
                vec![pen.at(ox, oy + 2.0 * t), pen.at(ox, oy + 2.0 * t + 14.0), pen.at(ox + 14.0, oy + 2.0 * t)],
                TX,
                Stroke::NONE,
            ));
            pen.dim_h(ox, ox + w, oy + 2.0 * t, 30.0, &format!("L = {}", fmt(*lw)));
            if pen.dims {
                pen.note(ox + 22.0, oy + 2.0 * t + 10.0, &format!("a = {} (throat)", fmt(*a)), Align2::LEFT_CENTER);
            }
        }
        Drawing::Raft { l, w, n, spacing, edge } => {
            let sc = fit(*l, *w, 300.0, 160.0);
            let (ww, hh, ox, oy) = (*l as f32 * sc, *w as f32 * sc, M, 24.0);
            pen.rect(ox, oy, ww, hh, CONCRETE, true);
            pen.axis(ox - 8.0, oy + hh / 2.0, ox + ww + 8.0, oy + hh / 2.0);
            for i in 0..(*n).min(200) {
                pen.rect(ox + (edge + i as f64 * spacing) as f32 * sc - 5.0, oy + hh / 2.0 - 5.0, 10.0, 10.0, STEEL, false);
            }
            pen.dim_h(ox, ox + ww, oy + hh, 22.0, &format!("L = {}", fmt(l.round())));
            pen.dim_v(oy, oy + hh, ox, -22.0, &format!("w = {}", fmt(*w)));
            if *n > 1 {
                pen.dim_h(ox + *edge as f32 * sc, ox + (edge + spacing) as f32 * sc, oy, -12.0, &fmt(*spacing));
            }
        }
        Drawing::Retain { hs, tb, ts, lt, b } => {
            let sc = fit(*b, hs + tb, 240.0, 240.0);
            let (w, hh, ox, oy) = (*b as f32 * sc, (*hs + *tb) as f32 * sc, M, 24.0);
            let stem_x = ox + *lt as f32 * sc;
            let heel_x = stem_x + *ts as f32 * sc;
            let hs_px = *hs as f32 * sc;
            pen.rect(heel_x, oy, (ox + w - heel_x).max(0.0), hs_px, SOIL, false);
            pen.rect(ox, oy + hs_px, w, *tb as f32 * sc, CONCRETE, true);
            pen.rect(stem_x, oy, *ts as f32 * sc, hs_px, CONCRETE, true);
            pen.dim_h(ox, ox + w, oy + hh, 22.0, &format!("B = {}", fmt(*b)));
            pen.dim_v(oy + hs_px, oy + hh, ox, -14.0, &fmt(*tb));
            pen.dim_v(oy, oy + hs_px, stem_x, -(stem_x - ox + 30.0), &format!("Hs = {}", fmt(*hs)));
            pen.dim_h(stem_x, heel_x, oy, -12.0, &fmt(*ts));
            if pen.dims {
                pen.note(heel_x + 6.0, oy + 12.0, "retained soil", Align2::LEFT_CENTER);
            }
        }
        Drawing::ISection { h, b, tw, tf } => {
            let sc = fit(*b, *h, 200.0, 260.0);
            let (w, hh, ox, oy) = (*b as f32 * sc, *h as f32 * sc, M, 24.0);
            let (twp, tfp) = ((*tw as f32 * sc).max(1.5), (*tf as f32 * sc).max(1.5));
            pen.rect(ox, oy, w, tfp, STEEL, false);
            pen.rect(ox, oy + hh - tfp, w, tfp, STEEL, false);
            pen.rect(ox + (w - twp) / 2.0, oy, twp, hh, STEEL, false);
            pen.axis(ox - 8.0, oy + hh / 2.0, ox + w + 8.0, oy + hh / 2.0);
            pen.dim_h(ox, ox + w, oy + hh, 22.0, &format!("b = {}", fmt(*b)));
            pen.dim_v(oy, oy + hh, ox, -22.0, &format!("h = {}", fmt(*h)));
            if pen.dims {
                pen.note(
                    ox + w / 2.0 + twp / 2.0 + 6.0,
                    oy + hh / 2.0 - 8.0,
                    &format!("tw {} · tf {}", fmt(*tw), fmt(*tf)),
                    Align2::LEFT_CENTER,
                );
            }
        }
        Drawing::Rect { b, h } => {
            let sc = fit(*b, *h, 200.0, 260.0);
            let (w, hh, ox, oy) = (*b as f32 * sc, *h as f32 * sc, M, 24.0);
            pen.rect(ox, oy, w, hh, TIMBER, true);
            for t in [0.3_f32, 0.5, 0.7] {
                let y = oy + hh * t;
                let pts: Vec<Pos2> = (0..=12)
                    .map(|k| {
                        let x = ox + 6.0 + (w - 12.0) * k as f32 / 12.0;
                        pen.at(x, y + 4.0 * ((k as f32 / 12.0) * std::f32::consts::TAU).sin())
                    })
                    .collect();
                pen.p.add(Shape::line(pts, Stroke::new(0.9_f32, GRAIN)));
            }
            pen.dim_h(ox, ox + w, oy + hh, 22.0, &format!("b = {}", fmt(*b)));
            pen.dim_v(oy, oy + hh, ox, -22.0, &format!("h = {}", fmt(*h)));
        }
        Drawing::Wall { h, cover, bar_dia, spacing } => {
            let sc = fit(*h, 1000.0, 160.0, 260.0);
            let (w, hh, ox, oy) = (*h as f32 * sc, 1000.0 * sc, M + 20.0, 24.0);
            pen.rect(ox, oy, w, hh, CONCRETE, true);
            let inset = (cover + bar_dia / 2.0) as f32 * sc;
            if *spacing > 0.0 {
                let mut y = spacing / 2.0;
                let mut k = 0;
                while y < 1000.0 && k < 200 {
                    for x in [ox + inset, ox + w - inset] {
                        pen.dot(x, oy + y as f32 * sc, (*bar_dia as f32 * sc / 2.0).max(2.2), TX);
                    }
                    y += spacing;
                    k += 1;
                }
            }
            pen.dim_h(ox, ox + w, oy + hh, 22.0, &format!("h = {}", fmt(*h)));
            pen.dim_v(oy, oy + hh, ox, -22.0, "1000 (per m)");
            if pen.dims {
                pen.note(ox + w + 8.0, oy + 20.0, &format!("φ{} @ {} EF", fmt(*bar_dia), fmt(*spacing)), Align2::LEFT_CENTER);
            }
        }
        Drawing::Strip { width, t, h } => {
            let sc = fit(*width, h + 600.0, 280.0, 220.0);
            let (w, hh, tw, ox, oy, wall_h) = (*width as f32 * sc, *h as f32 * sc, *t as f32 * sc, M, 24.0, 600.0 * sc);
            pen.rect(ox + (w - tw) / 2.0, oy, tw, wall_h, CONCRETE, true);
            pen.rect(ox, oy + wall_h, w, hh, CONCRETE, true);
            pen.p.line_segment(
                [pen.at(ox + 8.0, oy + wall_h + hh - 8.0), pen.at(ox + w - 8.0, oy + wall_h + hh - 8.0)],
                Stroke::new(2.4_f32, TX),
            );
            pen.dim_h(ox, ox + w, oy + wall_h + hh, 22.0, &format!("B = {}", fmt(*width)));
            pen.dim_v(oy + wall_h, oy + wall_h + hh, ox + w, 22.0, &format!("h = {}", fmt(*h)));
            pen.dim_h(ox + (w - tw) / 2.0, ox + (w + tw) / 2.0, oy, -10.0, &format!("t = {}", fmt(*t)));
        }
    }
}
