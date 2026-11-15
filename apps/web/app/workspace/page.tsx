"use client";
import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { loadEngine } from "../../lib/engine";

const init = { mode: "design", param_set: "EN1992-1-1:2004/RECOMMENDED", b: 300, h: 500, cover: 30, link_dia: 8, bar_dia: 20, fck: 30, fyk: 500, m_ed: 200, v_ed: 120, as_provided: 1256, asw_s_provided: 0.67 };
type Num = keyof typeof init;

export default function Workspace() {
  const [eng, setEng] = useState<any>(null);
  const [f, setF] = useState<any>(init);
  const [sets, setSets] = useState<any[]>([]);
  const [res, setRes] = useState<any>(null);
  const [err, setErr] = useState("");

  useEffect(() => { loadEngine().then((m) => { setEng(m); setSets(JSON.parse(m.param_sets())); }).catch((e) => setErr(String(e))); }, []);

  const run = () => {
    if (!eng) return;
    const p: any = { ...f };
    for (const k of Object.keys(p)) if (k !== "mode" && k !== "param_set") p[k] = Number(p[k]);
    if (p.mode !== "check") { delete p.as_provided; delete p.asw_s_provided; }
    setRes(JSON.parse(eng.run_rc_beam(JSON.stringify(p))));
  };
  useEffect(() => { if (eng) run(); /* eslint-disable-next-line */ }, [eng]);

  const field = (k: Num, label: string, unit: string) => (
    <div className="fg"><label>{label} ({unit})</label><input type="number" value={f[k]} onChange={(e) => setF({ ...f, [k]: e.target.value })} /></div>
  );
  const status = useMemo(() => (res?.ok ? (res.overall_pass ? "p" : "f") : ""), [res]);
  const exportJson = () => { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([JSON.stringify({ input: f, result: res }, null, 2)], { type: "application/json" })); a.download = `strucal-rc-beam-${res.trace_hash.slice(0, 8)}.json`; a.click(); };

  return (
    <>
      <header className="nav"><div className="wrap" style={{ maxWidth: "none" }}>
        <Link href="/" className="brand"><span className="mark">S</span>STRUCAL</Link>
        <nav><span>RC Beam</span></nav>
        <span style={{ fontSize: 12, color: "#9db0c6" }}>Preview build{eng ? ` - engine ${eng.engine_version()}` : ""}</span>
      </div></header>
      <div className="ws">
        <aside className="l">
          <div className="seg"><button className={f.mode === "design" ? "on" : ""} onClick={() => setF({ ...f, mode: "design" })}>Design</button><button className={f.mode === "check" ? "on" : ""} onClick={() => setF({ ...f, mode: "check" })}>Check Mode</button></div>
          <div className="gh">Design basis</div>
          <div className="fg"><label>Parameter set</label><select value={f.param_set} onChange={(e) => setF({ ...f, param_set: e.target.value })}>{sets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select></div>
          <div className="gh">Geometry</div>
          <div className="row2">{field("b", "Width b", "mm")}{field("h", "Depth h", "mm")}</div>
          <div className="row2">{field("cover", "Cover", "mm")}{field("link_dia", "Link dia", "mm")}</div>
          {field("bar_dia", "Main bar dia (assumed)", "mm")}
          <div className="gh">Materials</div>
          <div className="row2">{field("fck", "fck", "MPa")}{field("fyk", "fyk", "MPa")}</div>
          <div className="gh">Actions (ULS)</div>
          <div className="row2">{field("m_ed", "M Ed", "kNm")}{field("v_ed", "V Ed", "kN")}</div>
          {f.mode === "check" && (<><div className="gh">Provided (drawing)</div>{field("as_provided", "As provided", "mm2")}{field("asw_s_provided", "Asw/s provided", "mm2/mm")}</>)}
          <button className="btn dark" style={{ width: "100%", marginTop: 8 }} onClick={run} disabled={!eng}>{eng ? "Run calculation" : "Loading engine..."}</button>
        </aside>

        <main>
          <h2 style={{ fontSize: 26, marginBottom: 4 }}>Reinforced concrete beam - {f.mode === "check" ? "independent check" : "design"}</h2>
          <p style={{ color: "var(--mute)", marginTop: 0 }}>EN 1992-1-1:2004 - rectangular section, sagging moment and shear</p>
          {err && <div className="err">{err}</div>}
          {res && !res.ok && <div className="err">{res.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}</div>}
          {res?.ok && (<>
            {res.warnings.map((w: string, i: number) => <div className="note" key={i}>{w}</div>)}
            <table className="calc"><thead><tr><th>Reference</th><th>Calculation</th><th>Output</th></tr></thead><tbody>
              {res.trace.steps.map((s: any, i: number) => (<tr key={i}><td>{s.reference}</td><td>{s.calculation}</td><td>{s.output}</td></tr>))}
            </tbody></table>
            <p style={{ fontSize: 11.5, color: "var(--mute)", wordBreak: "break-all" }}>Trace hash (SHA-256): {res.trace_hash} - engine {res.engine_version}</p>
            <div style={{ marginTop: 18 }}>{res.scope_notes.map((n: string, i: number) => <div className="note" key={i}>{n}</div>)}</div>
          </>)}
        </main>

        <aside className="r">
          {res?.ok && (<>
            <div className={`badge ${status}`}>{res.overall_pass ? "PASS" : "FAIL"}</div>
            <div className="gh">Utilisation</div>
            {res.checks.map((c: any, i: number) => { const u = isFinite(c.utilisation) ? c.utilisation : 9.99; return (
              <div className="util" key={i}><div style={{ display: "flex", justifyContent: "space-between" }}><b>{c.name}</b><span className={c.pass ? "pass" : "fail"}>{isFinite(c.utilisation) ? (u * 100).toFixed(0) + "%" : "n/a"}</span></div>
                <div style={{ color: "var(--mute)" }}>{c.demand} vs {c.capacity}</div>
                <div className="bar"><i style={{ width: Math.min(100, u * 100) + "%", background: c.pass ? "var(--ok)" : "var(--bad)" }} /></div></div>); })}
            <div className="gh">Result</div>
            <div style={{ fontSize: 13.5, lineHeight: 1.8 }}>d = {res.d.toFixed(0)} mm<br />As,req = {res.as_req.toFixed(0)} mm2<br />{res.as_comp_req > 0 && <>As2,req = {res.as_comp_req.toFixed(0)} mm2<br /></>}As,min = {res.as_min.toFixed(0)} / As,max = {res.as_max.toFixed(0)} mm2<br />Tension: {res.bars}<br />Shear: {res.links}</div>
            <div className="noprint" style={{ display: "grid", gap: 8, marginTop: 18 }}><button className="btn dark" onClick={() => window.print()}>Print / save as PDF</button><button className="btn line" onClick={exportJson}>Export JSON</button></div>
          </>)}
        </aside>
      </div>
    </>
  );
}
