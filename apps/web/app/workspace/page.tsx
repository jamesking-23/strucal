"use client";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import TopBar from "../components/TopBar";
import { useEngine } from "../../lib/useEngine";
import { entryErrors, initialValues, mergeValues, parseNumber, splitEngineErrors, standardOf, toPayload, visibleFields } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText, printHtml } from "../../lib/download";
import { governing, fmtUtil, verdictOf } from "../../lib/status";
import type { FormValues, ModDef, ModResult } from "../../lib/types";
import { Button, EngineBoot, EmptyState, Notice, Segmented, StatusDot } from "../ui/primitives";
import ModulePicker from "./ModulePicker";
import Inspector from "./Inspector";
import Viewport from "./Viewport";
import CalcSheet from "./CalcSheet";
import VerificationRail from "./VerificationRail";
import Assistant from "./Assistant";
import "./workspace.css";

const STORE = "strucal.workspace.v1";
const UI_STORE = "strucal.workspace.ui.v1";
type Saved = { mid?: string; values?: Record<string, unknown> };
type Pane = "inputs" | "section" | "sheet" | "checks";

export default function Workspace() {
  const { engine: eng, error: loadErr, retry } = useEngine();
  const [mid, setMid] = useState("beam");
  const [vals, setVals] = useState<Record<string, FormValues>>({});
  const [collapsed, setCollapsed] = useState<Record<string, Record<string, boolean>>>({});
  const [menu, setMenu] = useState(false);
  const [notice, setNotice] = useState("");
  const [pane, setPane] = useState<Pane>("inputs");
  const [assistant, setAssistant] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const centerRef = useRef<HTMLDivElement>(null);

  // Restore saved inputs and panel state once the engine (and so the module definitions) is available.
  useEffect(() => {
    if (!eng) return;
    const saved = readJson<Saved>(STORE) ?? {};
    const ids = eng.paramSets.map((p) => p.id);
    const v: Record<string, FormValues> = {};
    for (const d of eng.defs) v[d.id] = mergeValues(d, saved.values?.[d.id], ids);
    setVals(v);
    setCollapsed(readJson<Record<string, Record<string, boolean>>>(UI_STORE) ?? {});
    if (saved.mid && eng.defs.some((d) => d.id === saved.mid)) setMid(saved.mid);
    else if (!eng.defs.some((d) => d.id === "beam") && eng.defs[0]) setMid(eng.defs[0].id);
  }, [eng]);

  // Persist inputs on every change so a reload or closed tab never loses work (the payload is small).
  useEffect(() => { if (eng && Object.keys(vals).length > 0) writeJson(STORE, { mid, values: vals }); }, [eng, mid, vals]);
  useEffect(() => { if (eng) writeJson(UI_STORE, collapsed); }, [eng, collapsed]);

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
    if (!eng || !def || !payload || Object.keys(entry).length) return null;
    return eng.run(def.id, payload);
  }, [eng, def, payload, entry]);
  const engineErr = useMemo(() => (def && res && !res.ok ? splitEngineErrors(def, res.errors) : { byField: {}, general: [] as string[] }), [def, res]);
  const fieldErrors = useMemo(() => ({ ...engineErr.byField, ...entry }), [engineErr, entry]);

  const verdict = verdictOf(res);
  const gov = governing(res);
  const tag = res?.trace_hash ? res.trace_hash.slice(0, 8) : "";
  const exportAs = (kind: "html" | "doc" | "print") => {
    if (!eng || !def || !payload) return;
    const html = eng.report(def.id, payload);
    setMenu(false);
    if (kind === "print") { if (!printHtml(html)) setNotice("Pop-ups are blocked. Allow pop-ups for this site, or export the .html sheet and print it."); return; }
    downloadText(`StruCal-${def.id}-${tag || "calc"}.${kind}`, html, kind === "html" ? "text/html" : "application/msword");
  };

  if (!eng || !def || !f) return <div className="app"><TopBar section="workspace" /><EngineBoot error={loadErr} onRetry={retry} /></div>;

  const paramSet = eng.paramSets.find((s) => s.id === f.param_set);
  const fields = visibleFields(def, f.mode);
  const nChanged = fields.filter((x) => { const v = parseNumber(f[x.key]); return v !== null && Math.abs(v - x.default) > 1e-12; }).length;
  const nInvalid = Object.keys(fieldErrors).length;
  const reset = () => { if (confirm(`Reset all ${def.name} parameters to their defaults?`)) setVals((all) => ({ ...all, [mid]: initialValues(def) })); };
  const showScope = () => { setPane("sheet"); setTimeout(() => document.getElementById("sc-h")?.scrollIntoView({ behavior: "smooth" }), 0); };
  const vpState = res?.ok ? "ready" : nInvalid > 0 || (res && !res.ok) ? "invalid" : "none";

  return (
    <div className="app wb" data-pane={pane}>
      <TopBar section="workspace" context={<>{def.name} · {standardOf(def)}{tag && ` · ${tag}`}</>} />

      <div className="toolbar wb-bar" role="toolbar" aria-label="Calculation">
        <ModulePicker defs={eng.defs} upcoming={eng.upcoming} value={mid} onChange={(id) => { setMid(id); setNotice(""); }} />
        <Segmented label="Mode" value={f.mode} onChange={(m) => setF({ mode: m })} options={[
          { value: "design", label: "Design" },
          { value: "check", label: "Check", disabled: !def.has_check_mode, title: def.has_check_mode ? "Verify the member as drawn" : "This module has no Check Mode" },
        ]} />
        <span className="sep" />
        <span className="lbl">Basis</span>
        <select className="select wb-params" aria-label="Parameter set" value={f.param_set} onChange={(e) => setF({ param_set: e.target.value })}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
        <span className="spacer" />
        <Button variant="ghost" onClick={() => setAssistant(true)}>Assistant</Button>
        <div className="rel" ref={menuRef}>
          <Button variant="primary" disabled={!res?.ok} aria-haspopup="menu" aria-expanded={menu} onClick={() => setMenu(!menu)}>Export ▾</Button>
          {menu && <div className="menu" role="menu">
            <button role="menuitem" onClick={() => exportAs("html")}>Calculation sheet (.html)<small>Full step-by-step working, A4 print layout</small></button>
            <button role="menuitem" onClick={() => exportAs("doc")}>Word document (.doc)<small>Opens and edits in Microsoft Word</small></button>
            <button role="menuitem" onClick={() => exportAs("print")}>Print or save as PDF<small>Uses the browser print dialog</small></button>
          </div>}
        </div>
      </div>

      <div className="tabs wb-tabs" role="tablist" aria-label="Workbench panes">
        {([["inputs", "Inputs"], ["section", "Section"], ["sheet", "Sheet"], ["checks", "Checks"]] as [Pane, string][]).map(([p, l]) => (
          <button key={p} role="tab" aria-selected={pane === p} onClick={() => setPane(p)}>
            {l}{p === "inputs" && nInvalid > 0 && <span className="badge fail">{nInvalid}</span>}{p === "checks" && verdict !== "none" && <StatusDot status={verdict} />}
          </button>
        ))}
      </div>

      <div className="wb-grid">
        <section className="pane wb-insp" aria-label="Parameters">
          <div className="pane-h">Parameters<span className="meta">{fields.length} · {f.mode}</span></div>
          <div className="pane-b">
            <Inspector def={def} values={f} errors={fieldErrors} collapsed={collapsed[mid] ?? {}}
              onToggle={(g, open) => setCollapsed((c) => ({ ...c, [mid]: { ...(c[mid] ?? {}), [g]: !open } }))} onChange={setF} />
          </div>
          <div className="wb-insp-foot">
            <span>{nInvalid > 0 ? <span className="t-fail">{nInvalid} invalid</span> : `${nChanged} changed from defaults`}</span>
            <span className="spacer" />
            <Button size="sm" variant="ghost" disabled={nChanged === 0 && f.mode === "design"} onClick={reset}>Reset all</Button>
          </div>
        </section>

        <main className="wb-center" ref={centerRef} aria-label="Section and calculation sheet">
          <Viewport drawing={res?.ok ? res.drawing : null} title={`Section · ${def.name}`} state={vpState} />
          <div className="wb-sheet">
            {notice && <div className="sheet" style={{ paddingBottom: 0 }}><Notice tone="info" onDismiss={() => setNotice("")}>{notice}</Notice></div>}
            {!res?.ok ? (
              <div className="sheet">
                <Notice tone="error" title="The calculation cannot run.">
                  {nInvalid > 0 ? `Correct ${nInvalid} parameter${nInvalid > 1 ? "s" : ""} marked in the inspector.` : "Check the inputs below."}
                  {engineErr.general.length > 0 && <ul>{engineErr.general.map((e, i) => <li key={i}>{e}</li>)}</ul>}
                </Notice>
                <EmptyState glyph="Σ" title="No calculation sheet yet">The sheet is generated from the engine trace as soon as every parameter is valid.</EmptyState>
              </div>
            ) : (
              <CalcSheet def={def} values={f} res={res} verdict={verdict} paramLabel={paramSet?.label ?? f.param_set} paramStatus={paramSet?.status ?? ""} />
            )}
          </div>
        </main>

        <section className="pane wb-rail" aria-label="Verification">
          <div className="pane-h">Verification<span className="meta">{res?.ok ? `${res.checks.length} checks` : "—"}</span></div>
          <div className="pane-b"><VerificationRail res={res} verdict={verdict} onShowScope={showScope} /></div>
        </section>
      </div>

      <footer className="wb-status">
        <span className="st"><StatusDot status={verdict} />{verdict === "none" ? "Not calculated" : verdict === "results" ? "Calculated" : verdict === "fail" ? "Checks failing" : verdict === "warn" ? "All checks satisfied · review" : "All checks satisfied"}</span>
        {gov && <span>η max {fmtUtil(gov.utilisation)} · {gov.name}</span>}
        <span className="spacer" />
        <span>{f.param_set}</span>{tag && <span>trace {tag}</span>}<span>engine v{eng.version}</span>
      </footer>
      <Assistant open={assistant} onClose={() => setAssistant(false)} def={def} values={f} res={res} onApply={(patch) => setF(patch)} />
    </div>
  );
}
