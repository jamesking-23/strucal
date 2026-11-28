//! Technical canvas shared by the Section and Loading views: a drafting grid that pans and zooms with the drawing.
//! Drawings are recorded into a `Sink` (a painter-like recorder in drawing units), then transformed onto the
//! screen, so pan and zoom apply to geometry, strokes and text alike (mirrors apps/web/app/workspace/Viewport.tsx).
use eframe::egui::{self, emath::TSTransform, Align2, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use std::cell::RefCell;
use std::sync::Arc;

use crate::theme::{mono, mute, p as pal};

/// Records shapes in drawing coordinates with the subset of the `egui::Painter` API the drawings use.
pub struct Sink {
    ctx: egui::Context,
    shapes: RefCell<Vec<Shape>>,
}

impl Sink {
    pub fn new(ctx: &egui::Context) -> Self {
        Sink { ctx: ctx.clone(), shapes: RefCell::new(vec![]) }
    }
    pub fn add(&self, s: impl Into<Shape>) {
        self.shapes.borrow_mut().push(s.into());
    }
    pub fn extend(&self, it: impl IntoIterator<Item = Shape>) {
        self.shapes.borrow_mut().extend(it);
    }
    pub fn rect_filled(&self, r: Rect, cr: impl Into<CornerRadius>, fill: impl Into<Color32>) {
        self.add(Shape::rect_filled(r, cr, fill));
    }
    pub fn rect_stroke(&self, r: Rect, cr: impl Into<CornerRadius>, stroke: impl Into<Stroke>, kind: StrokeKind) {
        self.add(Shape::rect_stroke(r, cr, stroke, kind));
    }
    pub fn circle_filled(&self, c: Pos2, r: f32, fill: impl Into<Color32>) {
        self.add(Shape::circle_filled(c, r, fill));
    }
    pub fn circle_stroke(&self, c: Pos2, r: f32, stroke: impl Into<Stroke>) {
        self.add(Shape::circle_stroke(c, r, stroke));
    }
    pub fn line_segment(&self, pts: [Pos2; 2], stroke: impl Into<Stroke>) {
        self.add(Shape::line_segment(pts, stroke));
    }
    pub fn layout_no_wrap(&self, text: String, font: FontId, color: Color32) -> Arc<Galley> {
        self.ctx.fonts(|f| f.layout_no_wrap(text, font, color))
    }
    pub fn text(&self, pos: Pos2, anchor: Align2, text: impl ToString, font: FontId, color: Color32) -> Rect {
        let g = self.layout_no_wrap(text.to_string(), font, color);
        let r = anchor.anchor_size(pos, g.size());
        self.add(Shape::galley(r.min, g, color));
        r
    }
    fn take(&self) -> Vec<Shape> {
        std::mem::take(&mut *self.shapes.borrow_mut())
    }
}

/// Pan and zoom of one canvas.
#[derive(Clone, Copy)]
pub struct View {
    pub zoom: f32,
    pub pan: Vec2,
}
impl Default for View {
    fn default() -> Self {
        View { zoom: 1.0, pan: Vec2::ZERO }
    }
}
impl View {
    pub fn zoom_by(&mut self, f: f32) {
        self.zoom = (self.zoom * f).clamp(0.4, 4.0);
    }
    pub fn fit(&mut self) {
        *self = View::default();
    }
}

fn grid(p: &egui::Painter, r: Rect, v: &View) {
    p.rect_filled(r, 0.0, pal().sunken);
    let step = 16.0 * v.zoom;
    let major = step * 5.0;
    let draw = |spacing: f32, color: Color32| {
        if spacing < 4.0 {
            return;
        }
        let ox = (r.center().x + v.pan.x - r.left()).rem_euclid(spacing);
        let oy = (r.center().y + v.pan.y - r.top()).rem_euclid(spacing);
        let mut x = r.left() + ox;
        while x <= r.right() {
            p.line_segment([Pos2::new(x, r.top()), Pos2::new(x, r.bottom())], Stroke::new(1.0_f32, color));
            x += spacing;
        }
        let mut y = r.top() + oy;
        while y <= r.bottom() {
            p.line_segment([Pos2::new(r.left(), y), Pos2::new(r.right(), y)], Stroke::new(1.0_f32, color));
            y += spacing;
        }
    };
    draw(step, pal().grid_minor);
    draw(major, pal().grid_major);
}

/// Shows a pannable, zoomable canvas. `draw` records the drawing and returns its extent in drawing units, or
/// `None` for the empty state. Drag pans, the wheel zooms, double-click fits.
pub fn show(ui: &mut egui::Ui, height: f32, v: &mut View, legend: &str, empty: &str, draw: impl FnOnce(&Sink) -> Option<(f32, f32)>) {
    let (r, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::click_and_drag());
    if resp.dragged() {
        v.pan += resp.drag_delta();
    }
    if resp.double_clicked() {
        v.fit();
    }
    if resp.hovered() {
        let dy = ui.input(|i| i.smooth_scroll_delta.y);
        if dy.abs() > 0.5 {
            v.zoom_by(if dy > 0.0 { 1.1 } else { 1.0 / 1.1 });
        }
        if resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
    }
    let p = ui.painter_at(r);
    grid(&p, r, v);
    let sink = Sink::new(ui.ctx());
    match draw(&sink) {
        Some((w, h)) => {
            let base = ((r.width() - 24.0) / w.max(1.0)).min((r.height() - 44.0) / h.max(1.0)).clamp(0.2, 1.0);
            let z = base * v.zoom;
            let t = TSTransform { scaling: z, translation: (r.center() + v.pan - egui::vec2(w, h) * z / 2.0).to_vec2() };
            for mut s in sink.take() {
                s.transform(t);
                p.add(s);
            }
        }
        None => {
            let c = r.center();
            p.line_segment([c + egui::vec2(-8.0, -8.0), c + egui::vec2(8.0, 8.0)], Stroke::new(1.0_f32, pal().ink4));
            p.line_segment([c + egui::vec2(-8.0, 8.0), c + egui::vec2(8.0, -8.0)], Stroke::new(1.0_f32, pal().ink4));
            p.text(c + egui::vec2(0.0, 22.0), Align2::CENTER_CENTER, empty, FontId::proportional(13.0), mute());
        }
    }
    p.text(Pos2::new(r.left() + 10.0, r.bottom() - 9.0), Align2::LEFT_BOTTOM, legend, mono(10.0), mute());
    p.text(
        Pos2::new(r.right() - 10.0, r.bottom() - 9.0),
        Align2::RIGHT_BOTTOM,
        "drag to pan · wheel to zoom · double-click to fit",
        mono(10.0),
        pal().ink4,
    );
}
