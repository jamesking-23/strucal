"use client";
import { Fragment, useEffect, useState } from "react";
import Link from "next/link";
import { loadEngine } from "../../lib/engine";
import "./workspace.css";

const init: any = { mode: "design", param_set: "EN1992-1-1:2004/RECOMMENDED", b: 300, h: 500, cover: 30, link_dia: 8, bar_dia: 20, fck: 30, fyk: 500, m_ed: 200, v_ed: 120, n_bars: 4, link_spacing: 200 };

function Inp({ f, setF, k, label, sub, unit }: any) {
  return (<div className="wk-f"><label>{label} {sub && <sub>{sub}</sub>}</label><div className="wk-in"><input type="number" value={f[k]} onChange={(e) => setF({ ...f, [k]: e.target.value })} /><span>{unit}</span></div></div>);
}

function Section({ f, r }: { f: any; r: any }) {
  const b = Number(f.b), h = Number(f.h), sc = Math.min(150 / b, 250 / h), W = b * sc, H = h * sc, ox = 25, oy = 15;
  const off = (Number(f.cover) + Number(f.link_dia) / 2) * sc, ed = (Number(f.cover) + Number(f.link_dia) + Number(f.bar_dia) / 2) * sc;
  const n = Math.max(2, r.bar_count), br = Math.max(2.4, (Number(f.bar_dia) * sc) / 2);
  return (<svg viewBox={`0 0 ${W + 50} ${H + 40}`}>
    <rect x={ox} y={oy} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
    <rect x={ox + off} y={oy + off} width={W - 2 * off} height={H - 2 * off} fill="none" stroke="#334155" strokeWidth="1.2" />
    {Array.from({ length: n }).map((_, i) => <circle key={i} cx={ox + ed + ((W - 2 * ed) * i) / (n - 1)} cy={oy + H - ed} r={br} fill="#0f172a" />)}
    {[0, 1].map((i) => <circle key={i} cx={ox + ed + (W - 2 * ed) * i} cy={oy + ed} r={Math.max(2.2, 6 * sc)} fill="#64748b" />)}
    <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">b = {b}</text>
    <text x={8} y={oy + H / 2} fontSize="10" fill="#2563eb" fontFamily="IBM Plex Mono" transform={`rotate(-90 8 ${oy + H / 2})`} textAnchor="middle">h = {h}</text>
  </svg>);
}

export default function Workspace() {
  const [eng, setEng] = useState<any>(null);
  const [f, setF] = useState<any>(init);
  const [sets, setSets] = useState<any[]>([]);
  const [res, setRes] = useState<any>(null);
  const [err, setErr] = useState("");
  const [menu, setMenu] = useState(false);

  useEffect(() => { loadEngine().then((m) => { setEng(m); setSets(JSON.parse(m.param_sets())); }).catch((e) => setErr(String(e))); }, []);
  const payload = () => { const p: any = {}; for (const k of Object.keys(f)) p[k] = k === "mode" || k === "param_set" ? f[k] : Number(f[k]); if (p.mode !== "check") { delete p.n_bars; delete p.link_spacing; } return JSON.stringify(p); };
  useEffect(() => { if (eng) setRes(JSON.parse(eng.run_rc_beam(payload()))); /* eslint-disable-next-line */ }, [eng, f]);

  const save = (name: string, mime: string) => { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([eng.report_html(payload())], { type: mime })); a.download = name; a.click(); setMenu(false); };
  const tag = res?.trace_hash?.slice(0, 8) ?? "calc";
  const print = () => { const w = window.open("", "_blank"); if (w) { w.document.write(eng.report_html(payload())); w.document.close(); setTimeout(() => w.print(), 400); } setMenu(false); };
  const ok = res?.ok;
  const verified = sets.find((s) => s.id === f.param_set)?.status ?? "";
  let n = 0, cur = "";

  return (
    <div className="wk">
      <header className="wk-top">
        <Link href="/" className="wk-logo"><i>S</i>StruCal</Link>
        <span className="wk-crumb">/ <b>EN 1992-1-1:2004</b> · RC Beam</span>
        <div className="wk-seg"><button className={f.mode === "design" ? "on" : ""} onClick={() => setF({ ...f, mode: "design" })}>Design</button><button className={f.mode === "check" ? "on" : ""} onClick={() => setF({ ...f, mode: "check" })}>Check Mode</button></div>
        <div className="wk-sp" />
        <select className="wk-sel" value={f.param_set} onChange={(e) => setF({ ...f, param_set: e.target.value })}>{sets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
        <button className="wk-btn" onClick={() => setF(init)}>Reset</button>
        <button className="wk-btn w" onClick={() => setMenu(!menu)} disabled={!ok}>Export ▾</button>
        {menu && <div className="wk-menu">
          <button onClick={() => save(`StruCal-RC-Beam-${tag}.html`, "text/html")}>Calculation sheet (.html)<small>Full step-by-step working</small></button>
          <button onClick={() => save(`StruCal-RC-Beam-${tag}.doc`, "application/msword")}>Word document (.doc)<small>Opens and edits in Microsoft Word</small></button>
          <button onClick={print}>Print / save as PDF<small>Use your browser&apos;s print dialog</small></button>
        </div>}
      </header>
      <div className="wk-body">
        <aside className="wk-l">
          <div className="wk-gh">Geometry</div>
          <div className="wk-2"><Inp {...{ f, setF }} k="b" label="Width" sub="b" unit="mm" /><Inp {...{ f, setF }} k="h" label="Depth" sub="h" unit="mm" /></div>
          <div className="wk-3"><Inp {...{ f, setF }} k="cover" label="Cover" sub="c" unit="mm" /><Inp {...{ f, setF }} k="link_dia" label="Link" sub="φlink" unit="mm" /><Inp {...{ f, setF }} k="bar_dia" label="Bar" sub="φ" unit="mm" /></div>
          <div className="wk-gh">Materials</div>
          <div className="wk-2"><Inp {...{ f, setF }} k="fck" label="fck" unit="MPa" /><Inp {...{ f, setF }} k="fyk" label="fyk" unit="MPa" /></div>
          <div className="wk-gh">Actions — ULS</div>
          <div className="wk-2"><Inp {...{ f, setF }} k="m_ed" label="Moment" sub="MEd" unit="kNm" /><Inp {...{ f, setF }} k="v_ed" label="Shear" sub="VEd" unit="kN" /></div>
          {f.mode === "check" && (<>
            <div className="wk-gh"><span>Provided reinforcement</span><span>From drawing</span></div>
            <div className="wk-2"><Inp {...{ f, setF }} k="n_bars" label="Tension bars" sub="n" unit={`× Ø${f.bar_dia}`} /><Inp {...{ f, setF }} k="link_spacing" label="Link spacing" sub="s" unit="mm" /></div>
            {ok && <div className="wk-calc"><span>Area As,prov</span><b className="mono">{res.as_prov.toFixed(0)} mm²</b></div>}
          </>)}
        </aside>

        <main className="wk-c">
          <h1 className="wk-h">Reinforced concrete beam — {f.mode === "check" ? "independent check" : "design"}</h1>
          <div className="wk-st">EN 1992-1-1:2004 · Rectangular section, sagging moment and shear</div>
          {err && <div className="wk-err">{err}</div>}
          {res && !res.ok && <div className="wk-err">{res.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}</div>}
          {ok && (<>
            {/UNVERIFIED|NOT VERIFIED|NOT A UGANDAN/.test(verified) && <div className="wk-ban"><b>Parameter set:</b> {verified}. Confirm the governing National Annex before issue for construction.</div>}
            {res.warnings.map((w: string, i: number) => <div className="wk-ban" key={i}>{w}</div>)}
            <div className="wk-card">
              <div className="wk-ch">Section definition <small className="mono">{f.b} × {f.h} RC</small></div>
              <div className="wk-sec"><Section f={f} r={res} />
                <div className="wk-kv">
                  <div><span>Effective depth d</span><b>{res.d.toFixed(1)} mm</b></div>
                  <div><span>Required tension steel A<sub>s,req</sub></span><b>{res.as_req.toFixed(0)} mm²</b></div>
                  <div><span>Provided tension steel A<sub>s,prov</sub></span><b>{res.as_prov.toFixed(0)} mm²<em>{res.bars}</em></b></div>
                  <div><span>Top steel</span><b>{res.top_steel}</b></div>
                  <div><span>Links</span><b>{res.links}</b></div>
                  <div><span>Lever arm z</span><b>{res.z.toFixed(1)} mm<em>{(res.z / res.d).toFixed(2)} d</em></b></div>
                  <div><span>μ / μ<sub>lim</sub></span><b>{res.mu.toFixed(4)} / {res.mu_lim.toFixed(4)}</b></div>
                </div></div>
            </div>
            <div className="wk-card">
              <div className="wk-ch">Calculation trace <small>{res.trace.steps.length} steps · deterministic</small></div>
              <table className="wk-t"><thead><tr><th>#</th><th>Reference</th><th>Calculation</th><th>Output</th></tr></thead><tbody>
                {res.trace.steps.map((s: any, i: number) => { const hdr = s.section !== cur; cur = s.section; n++; return (<Fragment key={i}>
                  {hdr && <tr className="sh" key={"h" + i}><td colSpan={4}>{s.section}</td></tr>}
                  <tr key={i}><td className="nn">{String(n).padStart(2, "0")}</td><td className="rf"><a>{s.reference || "—"}</a></td>
                    <td><div className="tl">{s.title}</div><div className="ca">{s.calculation}</div>{s.comment && <div className="cm">{s.comment}</div>}</td><td className="ou">{s.output}</td></tr></Fragment>); })}
              </tbody></table>
              <div className="wk-hash">SHA-256 {res.trace_hash} · engine v{res.engine_version}</div>
            </div>
            {res.scope_notes.map((t: string, i: number) => <div className="wk-nt" key={i}>{t}</div>)}
          </>)}
        </main>

        <aside className="wk-r">
          {ok && (<>
            <div className="wk-gh">Verification status</div>
            <div className={`wk-pass ${res.overall_pass ? "" : "f"}`}>
              <div className="a"><span>{res.overall_pass ? "PASS" : "FAIL"}</span><small>Max {(res.max_util * 100).toFixed(0)}%</small></div>
              <p>{res.checks.filter((c: any) => c.pass).length}/{res.checks.length} checks satisfied. Implemented checks only; not an engineer&apos;s approval.</p>
            </div>
            <div className="wk-gh"><span>Utilisation ratios</span><span>E<sub>d</sub> / R<sub>d</sub></span></div>
            {res.checks.map((c: any, i: number) => { const col = !c.pass ? "#b42318" : c.utilisation > 0.85 ? "#d98a00" : "#0f9d6a"; return (
              <div className="wk-u" key={i}><div className="h"><span>{c.name}</span><b style={{ color: col }}>{c.utilisation >= 9.99 ? "n/a" : c.utilisation.toFixed(2)}</b></div>
                <div className="d">{c.demand} vs {c.capacity}</div><div className="b"><i style={{ width: Math.min(100, c.utilisation * 100) + "%", background: col }} /></div></div>); })}
            <div className="wk-gh"><span>Observations</span><span>{res.observations.length} open</span></div>
            {res.observations.map((o: any) => <div className="wk-ob" key={o.id}><b>{o.id}</b><span className={`lv ${o.level}`}>{o.level}</span><div style={{ marginTop: 5 }}>{o.text}</div></div>)}
          </>)}
        </aside>
      </div>
    </div>
  );
}
