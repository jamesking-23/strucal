//! Calculation sheet: a hand-calculation style document generated from the deterministic trace.
use crate::module::{Drawing, ModDef, ModResult};
use serde_json::Value;

/// Markup (`x_{a}`, `x^{b}`, newlines) -> escaped HTML.
pub fn mk(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    let (mut o, mut i) = (String::new(), 0);
    while i < c.len() {
        if (c[i] == '_' || c[i] == '^') && i + 1 < c.len() && c[i + 1] == '{' {
            if let Some(j) = c[i + 2..].iter().position(|&x| x == '}') {
                let inner: String = c[i + 2..i + 2 + j].iter().collect();
                let tag = if c[i] == '_' { "sub" } else { "sup" };
                o.push_str(&format!("<{tag}>{}</{tag}>", esc(&inner))); i += j + 3; continue;
            }
        }
        match c[i] { '<' => o.push_str("&lt;"), '>' => o.push_str("&gt;"), '&' => o.push_str("&amp;"), '\n' => o.push_str("<br>"), ch => o.push(ch) }
        i += 1;
    }
    o
}
fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }

fn drawing_svg(d: &Drawing) -> String {
    match d {
        Drawing::Beam { b, h, cover, link_dia, bar_dia, n_bars } => {
            let sc = 150.0 / h.max(*b); let (pw, ph) = (b * sc, h * sc); let (ox, oy) = (20.0, 10.0);
            let mut s = format!("<svg width='{}' height='{}' xmlns='http://www.w3.org/2000/svg'><rect x='{ox}' y='{oy}' width='{pw}' height='{ph}' fill='#f4f5f7' stroke='#222' stroke-width='1.5'/>", pw + 40.0, ph + 30.0);
            let off = (cover + link_dia / 2.0) * sc;
            s += &format!("<rect x='{}' y='{}' width='{}' height='{}' fill='none' stroke='#222'/>", ox + off, oy + off, pw - 2.0 * off, ph - 2.0 * off);
            let n = (*n_bars).max(2) as f64; let br = (bar_dia * sc / 2.0).max(2.2);
            let ed = (cover + link_dia + bar_dia / 2.0) * sc; let (x0, x1) = (ox + ed, ox + pw - ed); let yb = oy + ph - ed;
            for k in 0..(n as u32) { let x = x0 + (x1 - x0) * k as f64 / (n - 1.0); s += &format!("<circle cx='{x}' cy='{yb}' r='{br}' fill='#111'/>"); }
            for x in [x0, x1] { s += &format!("<circle cx='{x}' cy='{}' r='{}' fill='#777'/>", oy + (cover + link_dia + 6.0) * sc, (6.0 * sc).max(2.0)); }
            s + &format!("<text x='{}' y='{}' font-size='10' text-anchor='middle'>b = {} / h = {}</text></svg>", ox + pw / 2.0, oy + ph + 18.0, b, h)
        }
        Drawing::Slab { h, cover, bar_dia, spacing } => {
            let sc = 0.6_f64; let w = 400.0 * sc; let hh = h * sc; let (ox, oy) = (20.0, 10.0);
            let mut s = format!("<svg width='{}' height='{}' xmlns='http://www.w3.org/2000/svg'><rect x='{ox}' y='{oy}' width='{w}' height='{hh}' fill='#f4f5f7' stroke='#222' stroke-width='1.5'/>", w + 40.0, hh + 30.0);
            let yb = oy + hh - (cover + bar_dia / 2.0) * sc; let mut x = ox + spacing * sc / 2.0;
            while x < ox + w { s += &format!("<circle cx='{x}' cy='{yb}' r='{}' fill='#111'/>", (bar_dia * sc / 2.0).max(2.0)); x += spacing * sc; }
            s + &format!("<text x='{}' y='{}' font-size='10' text-anchor='middle'>h = {} / T{} @ {}</text></svg>", ox + w / 2.0, oy + hh + 18.0, h, bar_dia, spacing)
        }
        Drawing::Pad { bx, by, cx, cy, h } => {
            let sc = 150.0 / bx.max(*by); let (pw, ph) = (bx * sc, by * sc); let (ox, oy) = (20.0, 10.0);
            format!("<svg width='{}' height='{}' xmlns='http://www.w3.org/2000/svg'><rect x='{ox}' y='{oy}' width='{pw}' height='{ph}' fill='#f4f5f7' stroke='#222' stroke-width='1.5'/><rect x='{}' y='{}' width='{}' height='{}' fill='#555'/><text x='{}' y='{}' font-size='10' text-anchor='middle'>{} × {} × h {}</text></svg>", pw + 40.0, ph + 30.0, ox + (pw - cx * sc) / 2.0, oy + (ph - cy * sc) / 2.0, cx * sc, cy * sc, ox + pw / 2.0, oy + ph + 18.0, bx, by, h)
        }
    }
}

pub fn generic(def: &ModDef, input: &Value, r: &ModResult) -> String {
    let mut body = String::new();
    let mode = input.get("mode").and_then(|m| m.as_str()).unwrap_or("design");
    body += &format!("<div class='top'><div><div class='brand'>STRUCAL</div><h1>Calculation sheet</h1><div class='sub'>{} – {}</div></div><div class='res {}'>{}<small>max utilisation {:.2}</small></div></div>",
        esc(def.title), if mode == "check" { "independent check" } else { "design" }, if r.overall_pass { "p" } else { "f" }, if r.overall_pass { "PASS" } else { "FAIL" }, r.max_util);
    body += "<table class='meta'><tr><td>Project</td><td></td><td>Calculation no.</td><td></td></tr><tr><td>Element</td><td></td><td>Revision</td><td></td></tr><tr><td>Prepared by</td><td></td><td>Checked by</td><td></td></tr><tr><td>Date</td><td></td><td>Approved by</td><td></td></tr></table>";
    body += &format!("<div class='basis'><b>Design basis.</b> {} Parameter set: {} <span class='tag'>{}</span></div>", esc(def.subtitle), esc(input.get("param_set").and_then(|p| p.as_str()).unwrap_or("")), esc(&r.param_status));
    body += "<h2>Input data</h2><table class='sum'><tbody>";
    for f in &def.fields { if f.check_only && mode != "check" { continue; } if let Some(x) = input.get(f.key).and_then(|x| x.as_f64()) { body += &format!("<tr><td>{}</td><td>{} {}</td></tr>", esc(f.label), x, esc(f.unit)); } }
    body += "</tbody></table>";
    let svg = r.drawing.as_ref().map(drawing_svg).unwrap_or_default();
    body += &format!("<div class='row'><div class='sec'>{}</div><div class='kv'>", svg);
    for k in &r.summary { body += &format!("<div><span>{}</span><b>{}</b></div>", esc(&k.label), esc(&k.value)); }
    body += "</div></div>";
    body += "<h2>Step-by-step calculation</h2><table class='calc'><thead><tr><th class='c1'>Reference</th><th class='c2'>Calculation</th><th class='c3'>Output</th></tr></thead><tbody>";
    let mut cur = ""; let mut k = 0;
    for s in &r.trace.steps {
        if s.section != cur { cur = &s.section; body += &format!("<tr class='sh'><td colspan='3'>{}</td></tr>", esc(cur)); }
        k += 1;
        body += &format!("<tr><td class='c1'>{}<div class='n'>{:02}</div></td><td class='c2'><div class='t'>{}</div>", esc(&s.reference), k, esc(&s.title));
        if !s.formula.is_empty() { body += &format!("<div class='f'>{}</div>", mk(&s.formula)); }
        if !s.substitution.is_empty() { body += &format!("<div class='s'>{}</div>", mk(&s.substitution)); }
        if !s.comment.is_empty() { body += &format!("<div class='cm'>{}</div>", mk(&s.comment)); }
        body += &format!("</td><td class='c3'>{}</td></tr>", mk(&s.result));
    }
    body += "</tbody></table><h2>Summary of checks</h2><table class='sum'><thead><tr><th>Check</th><th>Clause</th><th>Demand</th><th>Capacity</th><th>Utilisation</th><th>Result</th></tr></thead><tbody>";
    for c in &r.checks { body += &format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2}</td><td class='{}'>{}</td></tr>", esc(&c.name), esc(&c.reference), esc(&c.demand), esc(&c.capacity), c.utilisation, if c.pass { "p" } else { "f" }, if c.pass { "PASS" } else { "FAIL" }); }
    body += "</tbody></table>";
    if !r.observations.is_empty() { body += "<h2>Observations</h2><ul>"; for o in &r.observations { body += &format!("<li><b>{} [{}]</b> {}</li>", o.id, o.level, esc(&o.text)); } body += "</ul>"; }
    body += "<h2>Limitations</h2><ul>"; for n in &r.scope_notes { body += &format!("<li>{}</li>", esc(n)); } body += "</ul>";
    body += &format!("<div class='foot'>Engine v{} · Trace hash (SHA-256): {}<br>Input snapshot: {}<br>Generated by StruCal preview build. This sheet is not an engineer's approval and must be checked and signed by a registered engineer before issue.</div>", r.engine_version, r.trace_hash, esc(&input.to_string()));
    format!("<!DOCTYPE html><html><head><meta charset='utf-8'><title>StruCal calculation sheet</title><style>{}</style></head><body>{}</body></html>", CSS, body)
}

const CSS: &str = "@page{size:A4;margin:16mm}body{font-family:'Segoe UI',Arial,sans-serif;color:#111;font-size:11pt;max-width:900px;margin:24px auto;padding:0 20px}.top{display:flex;justify-content:space-between;align-items:flex-start;border-bottom:3px solid #0b1426;padding-bottom:10px}.brand{letter-spacing:.2em;font-weight:700;color:#2563eb;font-size:11pt}h1{margin:2px 0;font-size:22pt}.sub{color:#555}.res{padding:10px 18px;font-weight:800;font-size:18pt;text-align:center}.res small{display:block;font-size:9pt;font-weight:500}.res.p{background:#e3f4ee;color:#0f6b4f}.res.f{background:#fdeceb;color:#b42318}table{border-collapse:collapse;width:100%}table.meta{margin:12px 0}.meta td{border:1px solid #bbb;padding:5px 8px;font-size:10pt;height:22px}.meta td:nth-child(odd){background:#f1f3f6;width:14%;color:#444}.basis{background:#f1f3f6;padding:8px 10px;margin:8px 0;font-size:10pt}.tag{background:#fdf3e0;color:#8a5a00;padding:2px 6px;font-size:9pt;margin-left:6px}.row{display:flex;gap:20px;align-items:center;border:1px solid #ccc;padding:10px;margin:10px 0}.kv div{display:flex;gap:12px;padding:3px 0;font-size:10pt}.kv span{color:#555;min-width:130px}h2{font-size:13pt;margin:22px 0 8px;border-bottom:1px solid #999;padding-bottom:3px}.calc th,.sum th{background:#0b1426;color:#fff;text-align:left;padding:6px 8px;font-size:9.5pt;text-transform:uppercase;letter-spacing:.05em}.calc td{border:1px solid #ccc;vertical-align:top;padding:7px 9px}.c1{width:20%;font-size:9pt;color:#2457b8}.c3{width:22%;font-weight:700;text-align:left}.n{color:#999;font-size:8pt;margin-top:3px}.sh td{background:#e8edf5;font-weight:700;padding:6px 9px;border:1px solid #ccc}.t{font-weight:600;margin-bottom:3px}.f{font-family:Cambria,'Times New Roman',serif;font-style:italic;font-size:11.5pt;margin:2px 0}.s{font-family:Cambria,'Times New Roman',serif;font-size:11.5pt;margin:2px 0 2px 18px}.cm{font-size:9pt;color:#666;margin-top:3px}.sum td{border:1px solid #ccc;padding:5px 8px;font-size:10pt}.p{color:#0f6b4f;font-weight:700}.f.x{}td.f{color:#b42318;font-weight:700}.foot{margin-top:24px;font-size:8.5pt;color:#666;border-top:1px solid #ccc;padding-top:8px;word-break:break-all}tr{page-break-inside:avoid}";

pub fn html_json(id: &str, input: &str) -> String {
    let v: Value = match serde_json::from_str(input) { Ok(v) => v, Err(e) => return format!("<html><body>Invalid input: {}</body></html>", esc(&e.to_string())) };
    let def = match crate::modules::def(id) { Some(d) => d, None => return "<html><body>Unknown module</body></html>".into() };
    let r = crate::modules::run(id, &v);
    if r.ok { generic(&def, &v, &r) } else { format!("<html><body><h3>Cannot generate report</h3><p>{}</p></body></html>", esc(&r.errors.join("; "))) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_contains_working() {
        let h = html_json("slab", r#"{"mode":"design","param_set":"EN1992-1-1:2004/RECOMMENDED","span":4000,"h":175,"cover":25,"bar_dia":12,"fck":30,"fyk":500,"gk":1.5,"qk":3}"#);
        assert!(h.contains("Step-by-step calculation") && h.contains("<sub>cd</sub>") && h.contains("Trace hash"));
    }
}
