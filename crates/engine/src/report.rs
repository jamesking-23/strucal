//! Calculation sheet: a hand-calculation style document generated from the deterministic trace.
use crate::rc_beam::{BeamInput, BeamResult};

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

pub fn html(i: &BeamInput, r: &BeamResult) -> String {
    let mut body = String::new();
    // Section drawing (SVG)
    let (w, h) = (i.b, i.h);
    let sc = 150.0 / h.max(w);
    let (pw, ph) = (w * sc, h * sc);
    let ox = 20.0; let oy = 10.0;
    let mut svg = format!("<svg width='{}' height='{}' xmlns='http://www.w3.org/2000/svg'>", pw + 40.0, ph + 30.0);
    svg += &format!("<rect x='{ox}' y='{oy}' width='{pw}' height='{ph}' fill='#f4f5f7' stroke='#222' stroke-width='1.5'/>");
    let off = (i.cover + i.link_dia / 2.0) * sc;
    svg += &format!("<rect x='{}' y='{}' width='{}' height='{}' fill='none' stroke='#222' stroke-width='1'/>", ox + off, oy + off, pw - 2.0 * off, ph - 2.0 * off);
    let n = r.bar_count.max(2) as f64;
    let br = (i.bar_dia * sc / 2.0).max(2.2);
    let x0 = ox + (i.cover + i.link_dia + i.bar_dia / 2.0) * sc; let x1 = ox + pw - (i.cover + i.link_dia + i.bar_dia / 2.0) * sc;
    let yb = oy + ph - (i.cover + i.link_dia + i.bar_dia / 2.0) * sc;
    for k in 0..(n as u32) { let x = x0 + (x1 - x0) * k as f64 / (n - 1.0); svg += &format!("<circle cx='{x}' cy='{yb}' r='{br}' fill='#111'/>"); }
    let yt = oy + (i.cover + i.link_dia + 6.0) * sc;
    for x in [x0, x1] { svg += &format!("<circle cx='{x}' cy='{yt}' r='{}' fill='#777'/>", (6.0 * sc).max(2.0)); }
    svg += &format!("<text x='{}' y='{}' font-size='10' text-anchor='middle'>b = {}</text></svg>", ox + pw / 2.0, oy + ph + 18.0, i.b);

    body += &format!("<div class='top'><div><div class='brand'>STRUCAL</div><h1>Calculation sheet</h1><div class='sub'>Reinforced concrete beam – {} to EN 1992-1-1:2004</div></div><div class='res {}'>{}<small>max utilisation {:.2}</small></div></div>",
        if i.mode == "check" { "independent check" } else { "design" }, if r.overall_pass { "p" } else { "f" }, if r.overall_pass { "PASS" } else { "FAIL" }, r.max_util);
    body += "<table class='meta'><tr><td>Project</td><td></td><td>Calculation no.</td><td></td></tr><tr><td>Element</td><td></td><td>Revision</td><td></td></tr><tr><td>Prepared by</td><td></td><td>Checked by</td><td></td></tr><tr><td>Date</td><td></td><td>Approved by</td><td></td></tr></table>";
    body += &format!("<div class='basis'><b>Design basis.</b> EN 1992-1-1:2004. Parameter set: {} <span class='tag'>{}</span></div>", esc(&i.param_set), esc(&r.param_status));
    body += &format!("<div class='row'><div class='sec'>{}</div><div class='kv'><div><span>Section</span><b>{} × {} mm</b></div><div><span>Effective depth d</span><b>{:.1} mm</b></div><div><span>Tension steel</span><b>{} (A<sub>s,prov</sub> = {:.0} mm<sup>2</sup>)</b></div><div><span>Top steel</span><b>{}</b></div><div><span>Links</span><b>{}</b></div></div></div>", svg, i.b, i.h, r.d, esc(&r.bars), r.as_prov, esc(&r.top_steel), esc(&r.links));

    body += "<h2>Step-by-step calculation</h2><table class='calc'><thead><tr><th class='c1'>Reference</th><th class='c2'>Calculation</th><th class='c3'>Output</th></tr></thead><tbody>";
    let mut cur = ""; let mut k = 0;
    for s in &r.trace.steps {
        if s.section != cur { cur = &s.section; body += &format!("<tr class='sh'><td colspan='3'>{}</td></tr>", esc(cur)); }
        k += 1;
        body += &format!("<tr><td class='c1'>{}<div class='n'>{:02}</div></td><td class='c2'>", esc(&s.reference), k);
        body += &format!("<div class='t'>{}</div>", esc(&s.title));
        if !s.formula.is_empty() { body += &format!("<div class='f'>{}</div>", mk(&s.formula)); }
        if !s.substitution.is_empty() { body += &format!("<div class='s'>{}</div>", mk(&s.substitution)); }
        if !s.comment.is_empty() { body += &format!("<div class='cm'>{}</div>", mk(&s.comment)); }
        body += &format!("</td><td class='c3'>{}</td></tr>", mk(&s.result));
    }
    body += "</tbody></table>";

    body += "<h2>Summary of checks</h2><table class='sum'><thead><tr><th>Check</th><th>Clause</th><th>Demand</th><th>Capacity</th><th>Utilisation</th><th>Result</th></tr></thead><tbody>";
    for c in &r.checks { body += &format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2}</td><td class='{}'>{}</td></tr>", esc(&c.name), esc(&c.reference), mk(&c.demand), mk(&c.capacity), c.utilisation, if c.pass { "p" } else { "f" }, if c.pass { "PASS" } else { "FAIL" }); }
    body += "</tbody></table>";
    if !r.observations.is_empty() { body += "<h2>Observations</h2><ul>"; for o in &r.observations { body += &format!("<li><b>{} [{}]</b> {}</li>", o.id, o.level, esc(&o.text)); } body += "</ul>"; }
    body += "<h2>Limitations</h2><ul>"; for n in &r.scope_notes { body += &format!("<li>{}</li>", esc(n)); } body += "</ul>";
    body += &format!("<div class='foot'>Engine v{} · Trace hash (SHA-256): {}<br>Input snapshot: {}<br>Generated by StruCal preview build. This sheet is not an engineer's approval and must be checked and signed by a registered engineer before issue.</div>",
        r.engine_version, r.trace_hash, esc(&serde_json::to_string(i).unwrap_or_default()));

    format!("<!DOCTYPE html><html><head><meta charset='utf-8'><title>StruCal calculation sheet</title><style>{}</style></head><body>{}</body></html>", CSS, body)
}

const CSS: &str = "@page{size:A4;margin:16mm}body{font-family:'Segoe UI',Arial,sans-serif;color:#111;font-size:11pt;max-width:900px;margin:24px auto;padding:0 20px}.top{display:flex;justify-content:space-between;align-items:flex-start;border-bottom:3px solid #0b1426;padding-bottom:10px}.brand{letter-spacing:.2em;font-weight:700;color:#2563eb;font-size:11pt}h1{margin:2px 0;font-size:22pt}.sub{color:#555}.res{padding:10px 18px;font-weight:800;font-size:18pt;text-align:center}.res small{display:block;font-size:9pt;font-weight:500}.res.p{background:#e3f4ee;color:#0f6b4f}.res.f{background:#fdeceb;color:#b42318}table{border-collapse:collapse;width:100%}table.meta{margin:12px 0}.meta td{border:1px solid #bbb;padding:5px 8px;font-size:10pt;height:22px}.meta td:nth-child(odd){background:#f1f3f6;width:14%;color:#444}.basis{background:#f1f3f6;padding:8px 10px;margin:8px 0;font-size:10pt}.tag{background:#fdf3e0;color:#8a5a00;padding:2px 6px;font-size:9pt;margin-left:6px}.row{display:flex;gap:20px;align-items:center;border:1px solid #ccc;padding:10px;margin:10px 0}.kv div{display:flex;gap:12px;padding:3px 0;font-size:10pt}.kv span{color:#555;min-width:130px}h2{font-size:13pt;margin:22px 0 8px;border-bottom:1px solid #999;padding-bottom:3px}.calc th,.sum th{background:#0b1426;color:#fff;text-align:left;padding:6px 8px;font-size:9.5pt;text-transform:uppercase;letter-spacing:.05em}.calc td{border:1px solid #ccc;vertical-align:top;padding:7px 9px}.c1{width:20%;font-size:9pt;color:#2457b8}.c3{width:22%;font-weight:700;text-align:left}.n{color:#999;font-size:8pt;margin-top:3px}.sh td{background:#e8edf5;font-weight:700;padding:6px 9px;border:1px solid #ccc}.t{font-weight:600;margin-bottom:3px}.f{font-family:Cambria,'Times New Roman',serif;font-style:italic;font-size:11.5pt;margin:2px 0}.s{font-family:Cambria,'Times New Roman',serif;font-size:11.5pt;margin:2px 0 2px 18px}.cm{font-size:9pt;color:#666;margin-top:3px}.sum td{border:1px solid #ccc;padding:5px 8px;font-size:10pt}.p{color:#0f6b4f;font-weight:700}.f.x{}td.f{color:#b42318;font-weight:700}.foot{margin-top:24px;font-size:8.5pt;color:#666;border-top:1px solid #ccc;padding-top:8px;word-break:break-all}tr{page-break-inside:avoid}";

pub fn html_json(input: &str) -> String {
    match serde_json::from_str::<BeamInput>(input) {
        Ok(i) => { let r = crate::rc_beam::run(&i); if r.ok { html(&i, &r) } else { format!("<html><body><h3>Cannot generate report</h3><p>{}</p></body></html>", esc(&r.errors.join("; "))) } }
        Err(e) => format!("<html><body>Invalid input: {}</body></html>", esc(&e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_contains_working() {
        let i = BeamInput { mode: "design".into(), param_set: "EN1992-1-1:2004/RECOMMENDED".into(), b: 300.0, h: 500.0, cover: 30.0, link_dia: 8.0, bar_dia: 20.0, fck: 30.0, fyk: 500.0, m_ed: 200.0, v_ed: 120.0, n_bars: None, link_spacing: None };
        let h = html(&i, &crate::rc_beam::run(&i));
        assert!(h.contains("Step-by-step calculation") && h.contains("<sub>cd</sub>") && h.contains("Trace hash"));
    }
}
