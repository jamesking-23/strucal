"use client";
import { Fragment, useCallback, useEffect, useMemo, useRef, useState } from "react";
import Link from "next/link";
import { failedResult } from "../../lib/engine";
import { useEngine } from "../../lib/useEngine";
import { DISCIPLINES, disciplineOf, entryErrors, fieldGroups, initialValues, mergeValues, standardOf, toPayload } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText, printHtml } from "../../lib/download";
import type { Field, FormValues, ModDef, ModResult } from "../../lib/types";
import SectionDrawing from "./SectionDrawing";
import Assistant from "./Assistant";
import "./workspace.css";

const STORE = "strucal.workspace.v1";
type Saved = { mid?: string; values?: Record<string, unknown> };

function Inp({ f, value, error, onChange }: { f: Field; value: string | number; error?: string; onChange: (v: string) => void }) {
  const id = `in-${f.key}`;
  if (f.options) {
    return (
      <div className="wk-f" style={{ gridColumn: "1 / -1" }}>
        <label htmlFor={id}>{f.label}</label>
        <div className="wk-in">
          <select id={id} style={{ width: "100%", border: 0, background: "transparent", padding: "6px 4px", font: "inherit" }} value={String(value)} onChange={(e) => onChange(e.target.value)}>
            {f.options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
          </select>
        </div>
      </div>
    );
  }
  return (
    <div className={`wk-f${error ? " bad" : ""}`}>
      <label htmlFor={id} title={f.label}>{f.label}</label>
      <div className="wk-in">
        <input id={id} type="text" inputMode="decimal" autoComplete="off" spellCheck={false} value={String(value)} aria-invalid={!!error} aria-describedby={error ? `${id}-e` : undefined} onChange={(e) => onChange(e.target.value)} />
        {f.unit && <span>{f.unit}</span>}
      </div>
      {error && <div className="fe" id={`${id}-e`}>{error}</div>}
    </div>
  );
}

export default function Workspace() {
  const { engine: eng, error: loadErr, retry } = useEngine();
  const [mid, setMid] = useState("beam");
  const [vals, setVals] = useState<Record<string, FormValues>>({});
  const [menu, setMenu] = useState(false);
  const [notice, setNotice] = useState("");
  const [tab, setTab] = useState("HOME");
  const [showL, setShowL] = useState(true);
  const [showR, setShowR] = useState(true);
  const [showM, setShowM] = useState(true);
  const menuRef = useRef<HTMLDivElement>(null);
  const scopeRef = useRef<HTMLDivElement>(null);

  // Restore saved inputs once the engine (and so the module definitions) is available.
  useEffect(() => {
    if (!eng) return;
    const saved = readJson<Saved>(STORE) ?? {};
    const ids = eng.paramSets.map((p) => p.id);
    const v: Record<string, FormValues> = {};
    for (const d of eng.defs) v[d.id] = mergeValues(d, saved.values?.[d.id], ids);
    setVals(v);
    if (saved.mid && eng.defs.some((d) => d.id === saved.mid)) setMid(saved.mid);
    else if (!eng.defs.some((d) => d.id === "beam") && eng.defs[0]) setMid(eng.defs[0].id);
  }, [eng]);

  // Persist inputs on every change so a reload or closed tab never loses work (the payload is small).
  useEffect(() => {
    if (eng && Object.keys(vals).length > 0) writeJson(STORE, { mid, values: vals });
  }, [eng, mid, vals]);

  // Close the export menu on outside click or Escape.
  useEffect(() => {
    if (!menu) return;
    const down = (e: MouseEvent) => { if (menuRef.current && !menuRef.current.contains(e.target as Node)) setMenu(false); };
    const key = (e: KeyboardEvent) => { if (e.key === "Escape") setMenu(false); };
    document.addEventListener("mousedown", down);
    document.addEventListener("keydown", key);
    return () => { document.removeEventListener("mousedown", down); document.removeEventListener("keydown", key); };
  }, [menu]);

  const def: ModDef | undefined = eng?.defs.find((d) => d.id === mid);
  const f = vals[mid];
  const setF = useCallback((patch: Partial<FormValues>) => setVals((all) => ({ ...all, [mid]: { ...all[mid], ...patch } as FormValues })), [mid]);
  const entry = useMemo(() => (def && f ? entryErrors(def, f) : {}), [def, f]);
  const payload = useMemo(() => (def && f ? toPayload(def, f) : null), [def, f]);
  const res: ModResult | null = useMemo(() => {
    if (!eng || !def || !payload) return null;
    const bad = Object.keys(entry);
    if (bad.length) return failedResult([`Enter a number for: ${bad.map((k) => def.fields.find((x) => x.key === k)?.label ?? k).join(", ")}`], eng.version);
    return eng.run(def.id, payload);
  }, [eng, def, payload, entry]);

  const ok = !!res?.ok;
  const tag = res?.trace_hash ? res.trace_hash.slice(0, 8) : "calc";
  const exportAs = (kind: "html" | "doc" | "print") => {
    if (!eng || !def || !payload) return;
    const html = eng.report(def.id, payload);
    setMenu(false);
    if (kind === "print") { if (!printHtml(html)) setNotice("Pop-ups are blocked. Allow pop-ups for this site, or export the .html sheet and print it."); return; }
    downloadText(`StruCal-${def.id}-${tag}.${kind}`, html, kind === "html" ? "text/html" : "application/msword");
  };
  const selectModule = (id: string) => { setMid(id); setMenu(false); setNotice(""); };

  if (!eng || !def || !f) {
    return (
      <div className="wk">
        <div className="wk-title"><Link href="/" className="wk-logo" aria-label="StruCal home">S</Link></div>
        <div className="wk-splash" role="status" aria-live="polite">
          {loadErr ? (<><div>{loadErr}</div><button className="pj-btn" onClick={retry}>Try again</button></>) : (<><div className="wk-spin" /><div>Loading calculation engine…</div></>)}
        </div>
      </div>
    );
  }

  const paramStatus = eng.paramSets.find((s) => s.id === f.param_set)?.status ?? "";
  const nChecks = res?.checks.length ?? 0, nPass = res?.checks.filter((c) => c.pass).length ?? 0;
  const Btn = ({ ic, label, onClick, dis, on }: { ic: string; label: string; onClick: () => void; dis?: boolean; on?: boolean }) => (
    <button className={`rb${on ? " on" : ""}`} onClick={onClick} disabled={dis} aria-pressed={on}><span className="ic" aria-hidden>{ic}</span><span className="lb">{label}</span></button>
  );
  const reset = () => { if (confirm(`Reset all ${def.name} inputs to their defaults?`)) setVals((all) => ({ ...all, [mid]: initialValues(def) })); };
  let n = 0, cur = "";

  return (
    <div className="wk">
      <div className="wk-title">
        <Link href="/" className="wk-logo" title="StruCal - back to home">S</Link>
        <div className="qa"><button title="Reset inputs" aria-label="Reset inputs" onClick={reset}>↺</button><button title="Export" aria-label="Export" disabled={!ok} onClick={() => setMenu(!menu)}>⤓</button></div>
        <div className="wk-wt">StruCal — {def.name} — {standardOf(def)}</div>
      </div>
      <div className="wk-tabs">
        <span className="file">FILE</span>
        {["HOME", "VIEW", "HELP"].map((t) => <button key={t} className={tab === t ? "on" : ""} onClick={() => setTab(t)}>{t}</button>)}
        <Link href="/project" style={{ padding: "0 14px", lineHeight: "24px", color: "#333", textDecoration: "none" }}>PROJECT</Link>
        <Link href="/account" style={{ padding: "0 14px", lineHeight: "24px", color: "#333", textDecoration: "none" }}>ACCOUNT</Link>
      </div>
      <div className="wk-ribbon">
        {tab === "HOME" && (<>
          <div className="grp"><div className="cmds">
            <Btn ic="⟳" label="Reset" onClick={reset} />
            <div className="rel" ref={menuRef}><Btn ic="⤓" label="Export" onClick={() => setMenu(!menu)} dis={!ok} />
              {menu && <div className="wk-menu" role="menu">
                <button role="menuitem" onClick={() => exportAs("html")}>Calculation sheet (.html)<small>Full step-by-step working</small></button>
                <button role="menuitem" onClick={() => exportAs("doc")}>Word document (.doc)<small>Opens and edits in Microsoft Word</small></button>
                <button role="menuitem" onClick={() => exportAs("print")}>Print / save as PDF<small>Use your browser&apos;s print dialog</small></button>
              </div>}
            </div>
          </div><div className="gl">Document</div></div>
          <div className="grp"><div className="cmds">
            <Btn ic="▤" label="Design" on={f.mode === "design"} onClick={() => setF({ mode: "design" })} />
            {def.has_check_mode && <Btn ic="✓" label="Check Mode" on={f.mode === "check"} onClick={() => setF({ mode: "check" })} />}
          </div><div className="gl">Mode</div></div>
          <div className="grp"><div className="cmds col">
            <label className="rf">Module<select value={mid} onChange={(e) => selectModule(e.target.value)}>
              {DISCIPLINES.map((g) => <optgroup key={g} label={g}>{eng.defs.filter((d) => disciplineOf(d.id) === g).map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}</optgroup>)}
              {eng.upcoming.length > 0 && <optgroup label="Planned">{eng.upcoming.map((u) => <option key={u.name} disabled>{u.name} - {u.note}</option>)}</optgroup>}
            </select></label>
            <label className="rf">Parameters<select value={f.param_set} onChange={(e) => setF({ param_set: e.target.value })}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select></label>
          </div><div className="gl">Design basis</div></div>
        </>)}
        {tab === "VIEW" && (<div className="grp"><div className="cmds">
          <Btn ic="☰" label="Modules" on={showM} onClick={() => setShowM(!showM)} />
          <Btn ic="⌸" label="Input data" on={showL} onClick={() => setShowL(!showL)} />
          <Btn ic="◧" label="Verification" on={showR} onClick={() => setShowR(!showR)} />
        </div><div className="gl">Panels</div></div>)}
        {tab === "HELP" && (<div className="grp"><div className="cmds">
          <Btn ic="⌂" label="Home page" onClick={() => (window.location.href = "/")} />
          <Btn ic="§" label="Formulas" onClick={() => (window.location.href = "/standards")} />
          <Btn ic="ⓘ" label="Scope" dis={!ok} onClick={() => scopeRef.current?.scrollIntoView({ behavior: "smooth" })} />
        </div><div className="gl">Support</div></div>)}
      </div>

      <div className="wk-body" style={{ gridTemplateColumns: `${showL || showM ? "280px" : "0"} minmax(0,1fr) ${showR ? "320px" : "0"}` }}>
        <div className="wk-dock l" style={{ display: showL || showM ? undefined : "none" }}>
          {showM && <section className="pn"><header>Modules<button onClick={() => setShowM(false)} title="Close" aria-label="Close modules panel">×</button></header>
            <ul className="tree">
              {DISCIPLINES.map((g) => { const ds = eng.defs.filter((d) => disciplineOf(d.id) === g); return ds.length === 0 ? null : (<Fragment key={g}>
                <li className="root">▾ {g}</li>
                {ds.map((d) => <li key={d.id} className={d.id === mid ? "on" : ""} tabIndex={0} aria-current={d.id === mid} onClick={() => selectModule(d.id)} onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); selectModule(d.id); } }}>{d.name}</li>)}
              </Fragment>); })}
              {eng.upcoming.map((u) => <li key={u.name} className="dis" title={u.note}>{u.name}</li>)}
            </ul></section>}
          {showL && <section className="pn grow"><header>Input data — {f.mode === "check" ? "Check" : "Design"}<button onClick={() => setShowL(false)} title="Close" aria-label="Close input panel">×</button></header>
            <div className="pb">{fieldGroups(def, f.mode).map(({ group, fields }) => (<div key={group}><div className="wk-gh">{group}</div>
              <div className="wk-2">{fields.map((x) => <Inp key={x.key} f={x} value={f[x.key]} error={entry[x.key]} onChange={(v) => setF({ [x.key]: x.options ? Number(v) : v })} />)}</div></div>))}</div></section>}
        </div>

        <main className="wk-c">
          <div className="wk-doc">
            <h1 className="wk-h">{def.title} — {f.mode === "check" ? "independent check" : "design"}</h1>
            <div className="wk-st">{def.subtitle}</div>
            {notice && <div className="wk-ban" role="status">{notice} <button className="pj-btn" onClick={() => setNotice("")}>Dismiss</button></div>}
            {res && !res.ok && <div className="wk-err" role="alert">{res.errors.map((e, i) => <div key={i}>{e}</div>)}</div>}
            {res?.ok && (<>
              {/NOT/.test(paramStatus) && <div className="wk-ban"><b>Parameter set:</b> {paramStatus}. Confirm the governing National Annex before issue for construction.</div>}
              {res.warnings.map((w, i) => <div className="wk-ban" key={i}>{w}</div>)}
              <div className="wk-card">
                <div className="wk-ch">Section definition <small className="mono">{def.name}</small></div>
                <div className="wk-sec"><SectionDrawing d={res.drawing} />
                  <div className="wk-kv">{res.summary.map((k, i) => <div key={i}><span>{k.label}</span><b>{k.value}</b></div>)}</div></div>
              </div>
              <div className="wk-card">
                <div className="wk-ch">Calculation trace <small>{res.trace.steps.length} steps · deterministic</small></div>
                <table className="wk-t"><thead><tr><th>#</th><th>Reference</th><th>Calculation</th><th>Output</th></tr></thead><tbody>
                  {res.trace.steps.map((s, i) => { const hdr = s.section !== cur; cur = s.section; n++; return (<Fragment key={i}>
                    {hdr && <tr className="sh"><td colSpan={4}>{s.section}</td></tr>}
                    <tr><td className="nn">{String(n).padStart(2, "0")}</td><td className="rf"><span>{s.reference || "—"}</span></td>
                      <td><div className="tl">{s.title}</div><div className="ca">{s.calculation}</div>{s.comment && <div className="cm">{s.comment}</div>}</td><td className="ou">{s.output}</td></tr></Fragment>); })}
                </tbody></table>
                <div className="wk-hash">SHA-256 {res.trace_hash} · engine v{res.engine_version}</div>
              </div>
              <div ref={scopeRef}>{res.scope_notes.map((t, i) => <div className="wk-nt" key={i}>{t}</div>)}</div>
            </>)}
          </div>
        </main>

        <div className="wk-dock r" style={{ display: showR ? undefined : "none" }}>
          <section className="pn grow"><header>Verification<button onClick={() => setShowR(false)} title="Close" aria-label="Close verification panel">×</button></header>
            <div className="pb">{!ok && <div className="wk-nt">Results appear here once the inputs are valid.</div>}{res?.ok && (<>
              {nChecks === 0 ? (<div className="wk-pass"><div className="a"><span>RESULTS</span></div><p>Combined action effects. This tool gives design values, not a PASS/FAIL verdict.</p></div>) : (<div className={`wk-pass ${res.overall_pass ? "" : "f"}`}>
                <div className="a"><span>{res.overall_pass ? "PASS" : "FAIL"}</span><small>Max {(res.max_util * 100).toFixed(0)}%</small></div>
                <p>{nPass}/{nChecks} checks satisfied. Implemented checks only; not an engineer&apos;s approval.</p>
              </div>)}
              {nChecks > 0 && <div className="wk-gh"><span>Utilisation ratios</span><span>E<sub>d</sub> / R<sub>d</sub></span></div>}
              {res.checks.map((c, i) => { const col = !c.pass ? "#b42318" : c.utilisation > 0.85 ? "#d98a00" : "#0f9d6a"; return (
                <div className="wk-u" key={i}><div className="h"><span>{c.name}</span><b style={{ color: col }}>{c.utilisation >= 9.99 ? "n/a" : c.utilisation.toFixed(2)}</b></div>
                  <div className="d">{c.demand} vs {c.capacity}</div><div className="b"><i style={{ width: Math.min(100, c.utilisation * 100) + "%", background: col }} /></div></div>); })}
              <div className="wk-gh"><span>Observations</span><span>{res.observations.length} open</span></div>
              {res.observations.length === 0 && <div className="wk-nt">No observations.</div>}
              {res.observations.map((o) => <div className="wk-ob" key={o.id}><b>{o.id}</b><span className={`lv ${o.level}`}>{o.level}</span><div style={{ marginTop: 4 }}>{o.text}</div></div>)}
            </>)}</div></section>
        </div>
      </div>
      <Assistant def={def} values={f} res={res} onApply={(patch) => setF(patch)} />
      <footer className="wk-status">
        <span>{ok ? (nChecks === 0 ? "● Calculated" : res?.overall_pass ? "● All checks satisfied" : "● Checks failing") : "● Not calculated"}</span>
        <span>{f.param_set}</span><span className="sp" /><span>{tag !== "calc" ? `Trace ${tag}` : ""}</span><span>Engine v{eng.version}</span><span>Preview build</span>
      </footer>
    </div>
  );
}
