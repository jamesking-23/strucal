"use client";
import { Fragment, useCallback, useEffect, useMemo, useRef, useState } from "react";
import TopBar from "../components/TopBar";
import { failedResult } from "../../lib/engine";
import { useEngine } from "../../lib/useEngine";
import { DISCIPLINES, disciplineOf, entryErrors, fieldGroups, initialValues, mergeValues, standardOf, toPayload } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText, printHtml } from "../../lib/download";
import type { Check, Field, FormValues, ModDef, ModResult } from "../../lib/types";
import SectionDrawing from "./SectionDrawing";
import Assistant from "./Assistant";
import "./workspace.css";

const STORE = "strucal.workspace.v1";
const REVIEW = 0.85;
type Saved = { mid?: string; values?: Record<string, unknown> };
type Pane = "in" | "ver" | "doc";

const utilClass = (c: Check) => (!c.pass ? "fail" : c.utilisation > REVIEW ? "warn" : "");

function Inp({ f, value, error, onChange }: { f: Field; value: string | number; error?: string; onChange: (v: string) => void }) {
  const id = `in-${f.key}`;
  if (f.options) {
    return (
      <div className="field wide">
        <label htmlFor={id} title={f.label}>{f.label}</label>
        <select id={id} className="select" value={String(value)} onChange={(e) => onChange(e.target.value)}>
          {f.options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
        </select>
      </div>
    );
  }
  return (
    <div className="field">
      <label htmlFor={id} title={f.label}>{f.label}</label>
      <div className="affix">
        <input id={id} className="input num" type="text" inputMode="decimal" autoComplete="off" spellCheck={false} value={String(value)} aria-invalid={!!error} aria-describedby={error ? `${id}-e` : undefined} onChange={(e) => onChange(e.target.value)} />
        {f.unit && <span className="unit">{f.unit}</span>}
      </div>
      {error && <span className="field-error" id={`${id}-e`}>{error}</span>}
    </div>
  );
}

export default function Workspace() {
  const { engine: eng, error: loadErr, retry } = useEngine();
  const [mid, setMid] = useState("beam");
  const [vals, setVals] = useState<Record<string, FormValues>>({});
  const [menu, setMenu] = useState(false);
  const [notice, setNotice] = useState("");
  const [pane, setPane] = useState<Pane>("in");
  const [assistant, setAssistant] = useState(false);
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
  const tag = res?.trace_hash ? res.trace_hash.slice(0, 8) : "";
  const exportAs = (kind: "html" | "doc" | "print") => {
    if (!eng || !def || !payload) return;
    const html = eng.report(def.id, payload);
    setMenu(false);
    if (kind === "print") { if (!printHtml(html)) setNotice("Pop-ups are blocked. Allow pop-ups for this site, or export the .html sheet and print it."); return; }
    downloadText(`StruCal-${def.id}-${tag || "calc"}.${kind}`, html, kind === "html" ? "text/html" : "application/msword");
  };

  if (!eng || !def || !f) {
    return (
      <div className="app">
        <TopBar section="workspace" />
        <div className="loading" role="status" aria-live="polite">
          {loadErr ? (<div className="empty"><h2>The calculation engine did not load</h2><p>{loadErr}</p><button className="btn btn-primary" onClick={retry}>Try again</button></div>) : (<><span className="spinner" />Loading calculation engine…</>)}
        </div>
      </div>
    );
  }

  const paramStatus = eng.paramSets.find((s) => s.id === f.param_set)?.status ?? "";
  const nChecks = res?.checks.length ?? 0, nPass = res?.checks.filter((c) => c.pass).length ?? 0;
  const verdict = !res?.ok ? "none" : nChecks === 0 ? "results" : res.overall_pass ? "ok" : "fail";
  const reset = () => { if (confirm(`Reset all ${def.name} inputs to their defaults?`)) setVals((all) => ({ ...all, [mid]: initialValues(def) })); };
  let n = 0, cur = "";

  return (
    <div className="app ws" data-pane={pane}>
      <TopBar section="workspace" context={<>{def.name} · <span className="mono">{standardOf(def)}</span></>} />

      <div className="toolbar ws-bar" role="toolbar" aria-label="Calculation">
        <span className="lbl">Module</span>
        <select className="select ws-module" aria-label="Module" value={mid} onChange={(e) => { setMid(e.target.value); setNotice(""); }}>
          {DISCIPLINES.map((g) => <optgroup key={g} label={g}>{eng.defs.filter((d) => disciplineOf(d.id) === g).map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}</optgroup>)}
          {eng.upcoming.length > 0 && <optgroup label="Planned">{eng.upcoming.map((u) => <option key={u.name} disabled>{u.name} — {u.note}</option>)}</optgroup>}
        </select>
        <div className="seg" role="group" aria-label="Mode">
          <button aria-pressed={f.mode === "design"} onClick={() => setF({ mode: "design" })}>Design</button>
          <button aria-pressed={f.mode === "check"} disabled={!def.has_check_mode} title={def.has_check_mode ? "Check the member on the drawing" : "This module has no Check Mode"} onClick={() => setF({ mode: "check" })}>Check</button>
        </div>
        <span className="sep" />
        <span className="lbl">Parameters</span>
        <select className="select ws-params" aria-label="Parameter set" value={f.param_set} onChange={(e) => setF({ param_set: e.target.value })}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
        <span className="spacer" />
        <button className="btn btn-ghost" onClick={() => setAssistant(true)}>Assistant</button>
        <button className="btn btn-ghost" onClick={reset}>Reset</button>
        <div className="rel" ref={menuRef}>
          <button className="btn btn-primary" disabled={!ok} aria-haspopup="menu" aria-expanded={menu} onClick={() => setMenu(!menu)}>Export ▾</button>
          {menu && <div className="menu" role="menu">
            <button role="menuitem" onClick={() => exportAs("html")}>Calculation sheet (.html)<small>Full step-by-step working</small></button>
            <button role="menuitem" onClick={() => exportAs("doc")}>Word document (.doc)<small>Opens and edits in Microsoft Word</small></button>
            <button role="menuitem" onClick={() => exportAs("print")}>Print or save as PDF<small>Uses the browser print dialog</small></button>
          </div>}
        </div>
      </div>

      <div className="ws-tabs" role="tablist" aria-label="Workspace panes">
        <button role="tab" aria-selected={pane === "in"} onClick={() => setPane("in")}>Inputs{Object.keys(entry).length > 0 && <span className="badge fail">{Object.keys(entry).length}</span>}</button>
        <button role="tab" aria-selected={pane === "ver"} onClick={() => setPane("ver")}>Results{verdict === "ok" && <span className="badge ok">Pass</span>}{verdict === "fail" && <span className="badge fail">Fail</span>}</button>
        <button role="tab" aria-selected={pane === "doc"} onClick={() => setPane("doc")}>Sheet</button>
      </div>

      <div className="ws-body">
        <aside className="pane ws-in" aria-label="Inputs">
          <div className="pane-h">Inputs<span className="spacer" /><span className="badge">{f.mode === "check" ? "Check" : "Design"}</span></div>
          <div className="pane-b">
            {fieldGroups(def, f.mode).map(({ group, fields }) => (
              <div className="grp" key={group}>
                <div className="section-title">{group}</div>
                <div className="grp-fields">{fields.map((x) => <Inp key={x.key} f={x} value={f[x.key]} error={entry[x.key]} onChange={(v) => setF({ [x.key]: x.options ? Number(v) : v })} />)}</div>
                {fields.some((x) => x.check_only) && <span className="grp-note">Values as shown on the drawing (Check Mode).</span>}
              </div>
            ))}
          </div>
        </aside>

        <main className="pane ws-doc" aria-label="Calculation sheet">
          <div className="doc">
            <header className="doc-head">
              <span className="eyebrow">{f.mode === "check" ? "Independent check" : "Design"}</span>
              <h1>{def.title}</h1>
              <span className="std">{def.subtitle}</span>
            </header>
            {notice && <div className="alert info" role="status"><span className="grow">{notice}</span><button className="btn btn-sm" onClick={() => setNotice("")}>Dismiss</button></div>}
            {res && !res.ok && <div className="alert error" role="alert"><div className="grow"><b>The calculation cannot run.</b><ul>{res.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></div></div>}
            {res?.ok && (<>
              {/NOT/.test(paramStatus) && <div className="alert warn"><span className="grow"><b>Parameter set:</b> {paramStatus}. Confirm the governing National Annex before issue for construction.</span></div>}
              {res.warnings.map((w, i) => <div className="alert warn" key={i}><span className="grow">{w}</span></div>)}
              <section className="card">
                <div className="card-h"><h2>Section</h2><span className="spacer" /><span className="mono faint">{def.name}</span></div>
                <div className="card-b sect">
                  <SectionDrawing d={res.drawing} />
                  <dl className="kv">{res.summary.map((k, i) => <div key={i}><dt>{k.label}</dt><dd>{k.value}</dd></div>)}</dl>
                </div>
              </section>
              <section className="card">
                <div className="card-h"><h2>Calculation</h2><span className="spacer" /><span className="faint">{res.trace.steps.length} steps · deterministic</span></div>
                <div className="tbl-wrap">
                  <table className="trace">
                    <thead><tr><th>#</th><th>Reference</th><th>Calculation</th><th>Output</th></tr></thead>
                    <tbody>
                      {res.trace.steps.map((s, i) => { const hdr = s.section !== cur; cur = s.section; n++; return (<Fragment key={i}>
                        {hdr && <tr className="sh"><td colSpan={4}>{s.section}</td></tr>}
                        <tr>
                          <td className="n">{String(n).padStart(2, "0")}</td>
                          <td className="ref">{s.reference || "—"}</td>
                          <td><div className="t">{s.title}</div>{s.calculation && <div className="ca">{s.calculation}</div>}{s.comment && <div className="cm">{s.comment}</div>}</td>
                          <td className={`out${s.output === "PASS" ? " pass" : s.output === "FAIL" ? " fail" : ""}`}>{s.output}</td>
                        </tr></Fragment>); })}
                    </tbody>
                  </table>
                </div>
                <div className="notes" ref={scopeRef}>
                  <span className="section-title">Scope and limitations</span>
                  {res.scope_notes.map((t, i) => <p key={i}>{t}</p>)}
                  <span className="hash">SHA-256 {res.trace_hash} · engine v{res.engine_version}</span>
                </div>
              </section>
            </>)}
          </div>
        </main>

        <aside className="pane ws-ver" aria-label="Verification">
          <div className="pane-h">Verification</div>
          <div className="pane-b">
            {!res?.ok && <div className="empty"><h2>No result yet</h2><p>Results appear here as soon as every input is valid.</p>{pane !== "in" && <button className="btn" onClick={() => setPane("in")}>Review inputs</button>}</div>}
            {res?.ok && (<>
              <div className={`verdict ${verdict === "ok" ? "ok" : verdict === "fail" ? "fail" : ""}`} aria-live="polite">
                <div className="w"><b>{verdict === "results" ? "RESULTS" : verdict === "ok" ? "PASS" : "FAIL"}</b>{nChecks > 0 && <span>max {(res.max_util * 100).toFixed(0)}%</span>}</div>
                <p>{nChecks === 0 ? "Combined action effects. Design values only, no PASS/FAIL verdict." : `${nPass} of ${nChecks} checks satisfied. Implemented checks only; not an engineer's approval.`}</p>
              </div>
              {nChecks > 0 && (
                <div className="grp">
                  <div className="section-title">Utilisation <span className="num">E<sub>d</sub>/R<sub>d</sub></span></div>
                  <div className="chks">{res.checks.map((c, i) => (
                    <div className="chk" key={i}>
                      <div className="h"><span>{c.name}</span><b className={utilClass(c)}>{c.utilisation >= 9.99 ? "n/a" : c.utilisation.toFixed(2)}</b></div>
                      <div className={`util ${utilClass(c)}`}><i style={{ width: `${Math.min(100, c.utilisation * 100)}%` }} /></div>
                      <div className="d"><span>{c.demand} / {c.capacity}</span><span>{c.reference}</span></div>
                    </div>))}
                  </div>
                </div>
              )}
              <div className="grp">
                <div className="section-title">Observations <span className="num">{res.observations.length}</span></div>
                {res.observations.length === 0 ? <p className="muted">No observations.</p> : res.observations.map((o) => (
                  <div className="obs" key={o.id}><div className="h">{o.id}<span className={`badge ${o.level === "RED" ? "fail" : o.level === "AMBER" ? "warn" : "info"}`}>{o.level}</span></div><p>{o.text}</p></div>
                ))}
              </div>
              <button className="btn btn-ghost" onClick={() => { setPane("doc"); setTimeout(() => scopeRef.current?.scrollIntoView({ behavior: "smooth" }), 0); }}>Scope and limitations</button>
            </>)}
          </div>
        </aside>
      </div>

      <footer className="ws-status">
        <span><i className={`dot ${verdict === "ok" ? "ok" : verdict === "fail" ? "fail" : ""}`} />{!ok ? "Not calculated" : verdict === "results" ? "Calculated" : verdict === "ok" ? "All checks satisfied" : "Checks failing"}</span>
        <span>{f.param_set}</span><span className="spacer" />{tag && <span>trace {tag}</span>}<span>engine v{eng.version}</span><span>preview build</span>
      </footer>
      <Assistant open={assistant} onClose={() => setAssistant(false)} def={def} values={f} res={res} onApply={(patch) => setF(patch)} />
    </div>
  );
}
