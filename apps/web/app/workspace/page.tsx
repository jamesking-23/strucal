"use client";
import { Fragment, useEffect, useState } from "react";
import Link from "next/link";
import { loadEngine } from "../../lib/engine";
import "./workspace.css";

const PS = "EN1992-1-1:2004/RECOMMENDED";
const initFor = (def: any) => { const o: any = { mode: "design", param_set: PS }; for (const f of def.fields) o[f.key] = f.default; return o; };

function Inp({ f, setF, k, label, unit }: any) {
  return (<div className="wk-f"><label>{label}</label><div className="wk-in"><input type="number" value={f[k]} onChange={(e) => setF({ ...f, [k]: e.target.value })} /><span>{unit}</span></div></div>);
}

function Drawing({ d }: { d: any }) {
  if (!d) return null;
  if (d.kind === "beam") {
    const { b, h, cover, link_dia, bar_dia } = d, sc = Math.min(150 / b, 250 / h), W = b * sc, H = h * sc, ox = 25, oy = 15;
    const off = (cover + link_dia / 2) * sc, ed = (cover + link_dia + bar_dia / 2) * sc, n = Math.max(2, d.n_bars), br = Math.max(2.4, (bar_dia * sc) / 2);
    return (<svg viewBox={`0 0 ${W + 50} ${H + 40}`}>
      <rect x={ox} y={oy} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
      <rect x={ox + off} y={oy + off} width={W - 2 * off} height={H - 2 * off} fill="none" stroke="#334155" strokeWidth="1.2" />
      {Array.from({ length: n }).map((_, i) => <circle key={i} cx={ox + ed + ((W - 2 * ed) * i) / (n - 1)} cy={oy + H - ed} r={br} fill="#0f172a" />)}
      {[0, 1].map((i) => <circle key={i} cx={ox + ed + (W - 2 * ed) * i} cy={oy + ed} r={Math.max(2.2, 6 * sc)} fill="#64748b" />)}
      <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">b = {b}</text>
      <text x={8} y={oy + H / 2} fontSize="10" fill="#2563eb" fontFamily="IBM Plex Mono" transform={`rotate(-90 8 ${oy + H / 2})`} textAnchor="middle">h = {h}</text></svg>);
  }
  if (d.kind === "column") {
    const sc = Math.min(170 / d.b, 250 / d.h), W = d.b * sc, H = d.h * sc, ox = 25, oy = 15, off = (d.cover + d.link_dia / 2) * sc, ed = (d.cover + d.link_dia + d.bar_dia / 2) * sc, n = Math.max(2, d.n_face), br = Math.max(2.4, (d.bar_dia * sc) / 2);
    return (<svg viewBox={`0 0 ${W + 50} ${H + 40}`}>
      <rect x={ox} y={oy} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
      <rect x={ox + off} y={oy + off} width={W - 2 * off} height={H - 2 * off} fill="none" stroke="#334155" strokeWidth="1.2" />
      {[oy + ed, oy + H - ed].map((y, r) => Array.from({ length: n }).map((_, i) => <circle key={r + "-" + i} cx={ox + ed + ((W - 2 * ed) * i) / (n - 1)} cy={y} r={br} fill="#0f172a" />))}
      <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">{d.b} × {d.h}</text></svg>);
  }
  if (d.kind === "isection") {
    const sc = Math.min(160 / d.b, 250 / d.h), W = d.b * sc, H = d.h * sc, ox = 25, oy = 15, tw = d.tw * sc, tf = d.tf * sc;
    return (<svg viewBox={`0 0 ${W + 50} ${H + 40}`}>
      <rect x={ox} y={oy} width={W} height={tf} fill="#334155" /><rect x={ox} y={oy + H - tf} width={W} height={tf} fill="#334155" /><rect x={ox + (W - tw) / 2} y={oy} width={tw} height={H} fill="#334155" />
      <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">{d.h} × {d.b} × {d.tw} / {d.tf}</text></svg>);
  }
  if (d.kind === "wall") {
    const sc = Math.min(120 / d.h, 1), W = d.h * sc, H = 260, ox = 60, oy = 15, ys: number[] = [];
    for (let y = d.spacing / 2; y < 400; y += d.spacing) ys.push(y);
    return (<svg viewBox={`0 0 ${W + 120} ${H + 40}`}>
      <rect x={ox} y={oy} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
      {ys.map((y, i) => [ox + (d.cover + 10) * sc, ox + W - (d.cover + 10) * sc].map((x, k) => <circle key={i + "-" + k} cx={x} cy={oy + (y / 400) * H} r={Math.max(2.2, (d.bar_dia * sc) / 2)} fill="#0f172a" />))}
      <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">h = {d.h}, T{d.bar_dia} @ {d.spacing}</text></svg>);
  }
  if (d.kind === "strip") {
    const sc = 190 / d.width, W = d.width * sc, H = d.h * sc, ox = 20, oy = 15, T = d.t * sc;
    return (<svg viewBox={`0 0 ${W + 40} ${H + 100}`}>
      <rect x={ox + (W - T) / 2} y={oy} width={T} height={60} fill="#334155" />
      <rect x={ox} y={oy + 60} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
      <line x1={ox + 8} x2={ox + W - 8} y1={oy + 60 + H - 8} y2={oy + 60 + H - 8} stroke="#0f172a" strokeWidth="2.5" />
      <text x={ox + W / 2} y={oy + 60 + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">B = {d.width}, h = {d.h}</text></svg>);
  }
  if (d.kind === "slab") {
    const sc = 0.55, W = 400 * sc, H = d.h * sc, ox = 20, oy = 20, bars: number[] = [];
    for (let x = d.spacing / 2; x < 400; x += d.spacing) bars.push(x);
    return (<svg viewBox={`0 0 ${W + 40} ${H + 60}`}>
      <rect x={ox} y={oy} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
      {bars.map((x, i) => <circle key={i} cx={ox + x * sc} cy={oy + H - (d.cover + d.bar_dia / 2) * sc} r={Math.max(2.2, (d.bar_dia * sc) / 2)} fill="#0f172a" />)}
      <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">T{d.bar_dia} @ {d.spacing}</text>
      <text x={ox + W / 2} y={oy + H + 38} fontSize="10" textAnchor="middle" fill="#64748b" fontFamily="IBM Plex Mono">h = {d.h} (1 m strip section)</text></svg>);
  }
  const sc = 170 / Math.max(d.bx, d.by), W = d.bx * sc, H = d.by * sc, ox = 25, oy = 15;
  return (<svg viewBox={`0 0 ${W + 50} ${H + 50}`}>
    <rect x={ox} y={oy} width={W} height={H} fill="#fff" stroke="#0f172a" strokeWidth="2" />
    {Array.from({ length: 6 }).map((_, i) => <line key={i} x1={ox + 6} x2={ox + W - 6} y1={oy + (H * (i + 1)) / 7} y2={oy + (H * (i + 1)) / 7} stroke="#94a3b8" strokeWidth="1" />)}
    <rect x={ox + (W - d.cx * sc) / 2} y={oy + (H - d.cy * sc) / 2} width={d.cx * sc} height={d.cy * sc} fill="#334155" />
    <text x={ox + W / 2} y={oy + H + 22} fontSize="10" textAnchor="middle" fill="#2563eb" fontFamily="IBM Plex Mono">{d.bx} × {d.by} (plan), h = {d.h}</text></svg>);
}

export default function Workspace() {
  const [eng, setEng] = useState<any>(null);
  const [defs, setDefs] = useState<any[]>([]);
  const [soon, setSoon] = useState<any[]>([]);
  const [mid, setMid] = useState("beam");
  const [vals, setVals] = useState<any>({});
  const [sets, setSets] = useState<any[]>([]);
  const [res, setRes] = useState<any>(null);
  const [err, setErr] = useState("");
  const [menu, setMenu] = useState(false);
  const [tab, setTab] = useState("HOME");
  const [showL, setShowL] = useState(true);
  const [showR, setShowR] = useState(true);
  const [showM, setShowM] = useState(true);

  useEffect(() => { loadEngine().then((m) => {
    setEng(m); setSets(JSON.parse(m.param_sets())); setSoon(JSON.parse(m.upcoming()));
    const d = JSON.parse(m.modules()); setDefs(d); const v: any = {}; for (const x of d) v[x.id] = initFor(x); setVals(v);
  }).catch((e) => setErr(String(e))); }, []);

  const def = defs.find((d) => d.id === mid);
  const f = vals[mid];
  const setF = (nv: any) => setVals({ ...vals, [mid]: nv });
  const payload = () => { const p: any = {}; for (const k of Object.keys(f)) p[k] = k === "mode" || k === "param_set" ? f[k] : Number(f[k]); return JSON.stringify(p); };
  useEffect(() => { if (eng && def && f) setRes(JSON.parse(eng.run_module(mid, payload()))); /* eslint-disable-next-line */ }, [eng, vals, mid, defs]);

  const save = (name: string, mime: string) => { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([eng.report_html(mid, payload())], { type: mime })); a.download = name; a.click(); setMenu(false); };
  const tag = res?.trace_hash?.slice(0, 8) ?? "calc";
  const print = () => { const w = window.open("", "_blank"); if (w) { w.document.write(eng.report_html(mid, payload())); w.document.close(); setTimeout(() => w.print(), 400); } setMenu(false); };
  const ok = res?.ok;
  if (!def || !f) return <div className="wk"><div className="wk-title"><Link href="/" className="wk-logo" aria-label="StruCal">S</Link></div><div className="wk-splash"><div className="wk-spin" /><div>{err || "Loading calculation engine..."}</div></div></div>;
  const verified = sets.find((s) => s.id === f.param_set)?.status ?? "";
  const groups: string[] = []; for (const x of def.fields) { if (x.check_only && f.mode !== "check") continue; if (!groups.includes(x.group)) groups.push(x.group); }
  let n = 0, cur = "";

  const Btn = ({ ic, label, onClick, dis, on }: any) => (<button className={`rb${on ? " on" : ""}`} onClick={onClick} disabled={dis}><span className="ic">{ic}</span><span className="lb">{label}</span></button>);
  const nChecks = res?.checks?.length ?? 0, nPass = res?.checks?.filter((c: any) => c.pass).length ?? 0;
  const tabs = ["HOME", "VIEW", "HELP"];

  return (
    <div className="wk">
      <div className="wk-title">
        <Link href="/" className="wk-logo" title="StruCal - back to home">S</Link>
        <div className="qa"><button title="Reset inputs" onClick={() => setF(initFor(def))}>↺</button><button title="Export" disabled={!ok} onClick={() => setMenu(!menu)}>⤓</button></div>
        <div className="wk-wt">StruCal — {def.name} — EN 1992-1-1:2004</div>
      </div>
      <div className="wk-tabs">
        <span className="file">FILE</span>
        {tabs.map((t) => <button key={t} className={tab === t ? "on" : ""} onClick={() => setTab(t)}>{t}</button>)}
      </div>
      <div className="wk-ribbon">
        {tab === "HOME" && (<>
          <div className="grp"><div className="cmds">
            <Btn ic="⟳" label="Reset" onClick={() => setF(initFor(def))} />
            <div className="rel"><Btn ic="⤓" label="Export" onClick={() => setMenu(!menu)} dis={!ok} />
              {menu && <div className="wk-menu">
                <button onClick={() => save(`StruCal-${mid}-${tag}.html`, "text/html")}>Calculation sheet (.html)<small>Full step-by-step working</small></button>
                <button onClick={() => save(`StruCal-${mid}-${tag}.doc`, "application/msword")}>Word document (.doc)<small>Opens and edits in Microsoft Word</small></button>
                <button onClick={print}>Print / save as PDF<small>Use your browser&apos;s print dialog</small></button>
              </div>}
            </div>
          </div><div className="gl">Document</div></div>
          <div className="grp"><div className="cmds">
            <Btn ic="▤" label="Design" on={f.mode === "design"} onClick={() => setF({ ...f, mode: "design" })} />
            {def.has_check_mode && <Btn ic="✓" label="Check Mode" on={f.mode === "check"} onClick={() => setF({ ...f, mode: "check" })} />}
          </div><div className="gl">Mode</div></div>
          <div className="grp"><div className="cmds col">
            <label className="rf">Module<select value={mid} onChange={(e) => { setMid(e.target.value); setRes(null); }}>
              {defs.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}
              {soon.map((u) => <option key={u.name} disabled>{u.name} - {u.note}</option>)}
            </select></label>
            <label className="rf">Parameters<select value={f.param_set} onChange={(e) => setF({ ...f, param_set: e.target.value })}>{sets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select></label>
          </div><div className="gl">Design basis</div></div>
        </>)}
        {tab === "VIEW" && (<div className="grp"><div className="cmds">
          <Btn ic="☰" label="Modules" on={showM} onClick={() => setShowM(!showM)} />
          <Btn ic="⌸" label="Input data" on={showL} onClick={() => setShowL(!showL)} />
          <Btn ic="◧" label="Verification" on={showR} onClick={() => setShowR(!showR)} />
        </div><div className="gl">Panels</div></div>)}
        {tab === "HELP" && (<div className="grp"><div className="cmds">
          <Btn ic="⌂" label="Home page" onClick={() => (window.location.href = "/")} />
          <Btn ic="ⓘ" label="Scope" onClick={() => setTab("HOME")} />
        </div><div className="gl">Support</div></div>)}
      </div>

      <div className="wk-body" style={{ gridTemplateColumns: `${showL || showM ? "280px" : "0"} minmax(0,1fr) ${showR ? "320px" : "0"}` }}>
        <div className="wk-dock l" style={{ display: showL || showM ? undefined : "none" }}>
          {showM && <section className="pn"><header>Project Manager<button onClick={() => setShowM(false)} title="Close">×</button></header>
            <ul className="tree"><li className="root">▾ Reinforced concrete</li>
              {defs.map((d) => <li key={d.id} className={d.id === mid ? "on" : ""} onClick={() => { setMid(d.id); setRes(null); }}>{d.name}</li>)}
              {soon.map((u) => <li key={u.name} className="dis" title={u.note}>{u.name}</li>)}</ul></section>}
          {showL && <section className="pn grow"><header>Input data — {f.mode === "check" ? "Check" : "Design"}<button onClick={() => setShowL(false)} title="Close">×</button></header>
            <div className="pb">{groups.map((g) => (<div key={g}><div className="wk-gh">{g}</div>
              <div className="wk-2">{def.fields.filter((x: any) => x.group === g && (!x.check_only || f.mode === "check")).map((x: any) => <Inp key={x.key} {...{ f, setF }} k={x.key} label={x.label} unit={x.unit} />)}</div></div>))}</div></section>}
        </div>

        <main className="wk-c">
          <div className="wk-doctabs"><span className="on">{def.name}</span><span>Calculation sheet</span></div>
          <div className="wk-doc">
          <h1 className="wk-h">{def.title} — {f.mode === "check" ? "independent check" : "design"}</h1>
          <div className="wk-st">{def.subtitle}</div>
          {err && <div className="wk-err">{err}</div>}
          {res && !res.ok && <div className="wk-err">{res.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}</div>}
          {ok && (<>
            {/NOT VERIFIED|NOT A UGANDAN/.test(verified) && <div className="wk-ban"><b>Parameter set:</b> {verified}. Confirm the governing National Annex before issue for construction.</div>}
            {res.warnings.map((w: string, i: number) => <div className="wk-ban" key={i}>{w}</div>)}
            <div className="wk-card">
              <div className="wk-ch">Section definition <small className="mono">{def.name}</small></div>
              <div className="wk-sec"><Drawing d={res.drawing} />
                <div className="wk-kv">{res.summary.map((k: any, i: number) => <div key={i}><span>{k.label}</span><b>{k.value}</b></div>)}</div></div>
            </div>
            <div className="wk-card">
              <div className="wk-ch">Calculation trace <small>{res.trace.steps.length} steps · deterministic</small></div>
              <table className="wk-t"><thead><tr><th>#</th><th>Reference</th><th>Calculation</th><th>Output</th></tr></thead><tbody>
                {res.trace.steps.map((s: any, i: number) => { const hdr = s.section !== cur; cur = s.section; n++; return (<Fragment key={i}>
                  {hdr && <tr className="sh"><td colSpan={4}>{s.section}</td></tr>}
                  <tr><td className="nn">{String(n).padStart(2, "0")}</td><td className="rf"><a>{s.reference || "—"}</a></td>
                    <td><div className="tl">{s.title}</div><div className="ca">{s.calculation}</div>{s.comment && <div className="cm">{s.comment}</div>}</td><td className="ou">{s.output}</td></tr></Fragment>); })}
              </tbody></table>
              <div className="wk-hash">SHA-256 {res.trace_hash} · engine v{res.engine_version}</div>
            </div>
            {res.scope_notes.map((t: string, i: number) => <div className="wk-nt" key={i}>{t}</div>)}
          </>)}
          </div>
        </main>

        <div className="wk-dock r" style={{ display: showR ? undefined : "none" }}>
          <section className="pn grow"><header>Verification<button onClick={() => setShowR(false)} title="Close">×</button></header>
            <div className="pb">{ok && (<>
              {nChecks === 0 ? (<div className="wk-pass"><div className="a"><span>RESULTS</span></div><p>Combined action effects. This tool gives design values, not a PASS/FAIL verdict.</p></div>) : (<div className={`wk-pass ${res.overall_pass ? "" : "f"}`}>
                <div className="a"><span>{res.overall_pass ? "PASS" : "FAIL"}</span><small>Max {(res.max_util * 100).toFixed(0)}%</small></div>
                <p>{nPass}/{nChecks} checks satisfied. Implemented checks only; not an engineer&apos;s approval.</p>
              </div>)}
              {nChecks > 0 && <div className="wk-gh"><span>Utilisation ratios</span><span>E<sub>d</sub> / R<sub>d</sub></span></div>}
              {res.checks.map((c: any, i: number) => { const col = !c.pass ? "#b42318" : c.utilisation > 0.85 ? "#d98a00" : "#0f9d6a"; return (
                <div className="wk-u" key={i}><div className="h"><span>{c.name}</span><b style={{ color: col }}>{c.utilisation >= 9.99 ? "n/a" : c.utilisation.toFixed(2)}</b></div>
                  <div className="d">{c.demand} vs {c.capacity}</div><div className="b"><i style={{ width: Math.min(100, c.utilisation * 100) + "%", background: col }} /></div></div>); })}
              <div className="wk-gh"><span>Observations</span><span>{res.observations.length} open</span></div>
              {res.observations.map((o: any) => <div className="wk-ob" key={o.id}><b>{o.id}</b><span className={`lv ${o.level}`}>{o.level}</span><div style={{ marginTop: 4 }}>{o.text}</div></div>)}
            </>)}</div></section>
        </div>
      </div>
      <footer className="wk-status">
        <span>{ok ? (nChecks === 0 ? "● Calculated" : res.overall_pass ? "● All checks satisfied" : "● Checks failing") : "● Not calculated"}</span>
        <span>{f.param_set}</span><span className="sp" /><span>{tag !== "calc" ? `Trace ${tag}` : ""}</span><span>{res ? `Engine v${res.engine_version}` : ""}</span><span>Preview build</span>
      </footer>
    </div>
  );
}
