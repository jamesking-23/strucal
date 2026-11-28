"use client";
import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import AppShell from "../shell/AppShell";
import type { Command } from "../shell/CommandPalette";
import { useEngine } from "../../lib/useEngine";
import { disciplineOf, entryErrors, initialValues, mergeValues, parseNumber, splitEngineErrors, toPayload, visibleFields } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText, printHtml } from "../../lib/download";
import { fmtUtil, governing, verdictOf, VERDICT_WORD } from "../../lib/status";
import type { FormValues, ModDef, ModResult } from "../../lib/types";
import { Button, EngineBoot, EmptyState, Notice, StatusDot } from "../ui/primitives";
import { IconExport, IconPanelLeft, IconPanelRight, IconReset, IconSpark } from "../ui/icons";
import Inspector from "./Inspector";
import Viewport from "./Viewport";
import Ledger from "./Ledger";
import VerificationRail from "./VerificationRail";
import Assistant from "./Assistant";
import "./workbench.css";

const STORE = "strucal.workspace.v1";
const UI_STORE = "strucal.workbench.ui.v2";
type Saved = { mid?: string; values?: Record<string, unknown> };
type Ui = { insp: boolean; rail: boolean; split: number; collapsed: Record<string, Record<string, boolean>> };
type LedgerTab = "ledger" | "checks" | "summary" | "scope";
type MobilePane = "inputs" | "model" | "ledger" | "verdict";
const UI_DEFAULT: Ui = { insp: true, rail: true, split: 0.46, collapsed: {} };

function Workbench() {
  const router = useRouter();
  const params = useSearchParams();
  const { engine: eng, error: loadErr, retry } = useEngine();
  const [mid, setMid] = useState("beam");
  const [vals, setVals] = useState<Record<string, FormValues>>({});
  const [ui, setUi] = useState<Ui>(UI_DEFAULT);
  const [ledgerTab, setLedgerTab] = useState<LedgerTab>("ledger");
  const [pane, setPane] = useState<MobilePane>("inputs");
  const [menu, setMenu] = useState(false);
  const [notice, setNotice] = useState("");
  const [assistant, setAssistant] = useState(false);
  const [restored, setRestored] = useState(false);
  const center = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  // Restore inputs, layout and the module (URL ?m= wins over the last session) once the engine is ready.
  useEffect(() => {
    if (!eng) return;
    const saved = readJson<Saved>(STORE) ?? {};
    const ids = eng.paramSets.map((p) => p.id);
    const v: Record<string, FormValues> = {};
    for (const d of eng.defs) v[d.id] = mergeValues(d, saved.values?.[d.id], ids);
    setVals(v);
    const u = readJson<Partial<Ui>>(UI_STORE);
    if (u) setUi({ ...UI_DEFAULT, ...u, split: Math.min(0.8, Math.max(0.2, Number(u.split) || UI_DEFAULT.split)) });
    const want = params.get("m") ?? saved.mid;
    setMid(want && eng.defs.some((d) => d.id === want) ? want : eng.defs.some((d) => d.id === "beam") ? "beam" : eng.defs[0].id);
    setRestored(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eng]);

  useEffect(() => { if (restored) writeJson(STORE, { mid, values: vals }); }, [restored, mid, vals]);
  useEffect(() => { if (restored) writeJson(UI_STORE, ui); }, [restored, ui]);
  // Keep the module in the URL so a link reopens the same calculation.
  useEffect(() => { if (restored && params.get("m") !== mid) router.replace(`/workspace?m=${mid}`, { scroll: false }); }, [restored, mid, params, router]);

  useEffect(() => {
    if (!menu) return;
    const down = (e: MouseEvent) => { if (menuRef.current && !menuRef.current.contains(e.target as Node)) setMenu(false); };
    const key = (e: KeyboardEvent) => { if (e.key === "Escape") setMenu(false); };
    document.addEventListener("mousedown", down); document.addEventListener("keydown", key);
    return () => { document.removeEventListener("mousedown", down); document.removeEventListener("keydown", key); };
  }, [menu]);

  // [ and ] toggle the side panes (when not typing).
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      if (["INPUT", "SELECT", "TEXTAREA"].includes(t.tagName) || e.metaKey || e.ctrlKey || e.altKey) return;
      if (e.key === "[") setUi((u) => ({ ...u, insp: !u.insp }));
      if (e.key === "]") setUi((u) => ({ ...u, rail: !u.rail }));
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  const def: ModDef | undefined = eng?.defs.find((d) => d.id === mid);
  const f = vals[mid];
  const setF = useCallback((patch: Partial<FormValues>) => setVals((all) => ({ ...all, [mid]: { ...all[mid], ...patch } as FormValues })), [mid]);
  const entry = useMemo(() => (def && f ? entryErrors(def, f) : {}), [def, f]);
  const payload = useMemo(() => (def && f ? toPayload(def, f) : null), [def, f]);
  const res: ModResult | null = useMemo(() => (eng && def && payload && Object.keys(entry).length === 0 ? eng.run(def.id, payload) : null), [eng, def, payload, entry]);
  const engineErr = useMemo(() => (def && res && !res.ok ? splitEngineErrors(def, res.errors) : { byField: {}, general: [] as string[] }), [def, res]);
  const errors = useMemo(() => ({ ...engineErr.byField, ...entry }), [engineErr, entry]);
  const verdict = verdictOf(res);
  const gov = governing(res);
  const tag = res?.trace_hash ? res.trace_hash.slice(0, 8) : "";

  const exportAs = useCallback((kind: "html" | "doc" | "print") => {
    if (!eng || !def || !payload || !res?.ok) return;
    const html = eng.report(def.id, payload);
    setMenu(false);
    if (kind === "print") { if (!printHtml(html)) setNotice("Pop-ups are blocked. Allow pop-ups for this site, or export the .html sheet and print it."); return; }
    downloadText(`StruCal-${def.id}-${tag || "calc"}.${kind}`, html, kind === "html" ? "text/html" : "application/msword");
  }, [eng, def, payload, res, tag]);
  const resetAll = useCallback(() => { if (def && confirm(`Reset all ${def.name} inputs to their defaults?`)) setVals((all) => ({ ...all, [def.id]: initialValues(def) })); }, [def]);

  const commands = useMemo<Command[]>(() => !eng ? [] : [
    ...eng.defs.map((d) => ({ id: `m-${d.id}`, group: "Modules", title: d.name, hint: disciplineOf(d.id), keywords: `${d.title} ${d.subtitle}`, run: () => setMid(d.id) })),
    { id: "a-export", group: "Calculation", title: "Export calculation sheet (.html)", run: () => exportAs("html") },
    { id: "a-doc", group: "Calculation", title: "Export Word document (.doc)", run: () => exportAs("doc") },
    { id: "a-print", group: "Calculation", title: "Print or save as PDF", run: () => exportAs("print") },
    { id: "a-reset", group: "Calculation", title: "Reset inputs to defaults", run: resetAll },
    { id: "a-assist", group: "Calculation", title: "Open engineering assistant", run: () => setAssistant(true) },
    { id: "v-insp", group: "Layout", title: ui.insp ? "Hide inspector" : "Show inspector", hint: "[", run: () => setUi((u) => ({ ...u, insp: !u.insp })) },
    { id: "v-rail", group: "Layout", title: ui.rail ? "Hide verification rail" : "Show verification rail", hint: "]", run: () => setUi((u) => ({ ...u, rail: !u.rail })) },
  ], [eng, exportAs, resetAll, ui.insp, ui.rail]);

  // Splitter between viewport and ledger (pointer drag or arrow keys).
  const startSplit = (e: React.PointerEvent) => {
    const el = center.current;
    if (!el) return;
    const box = el.getBoundingClientRect();
    const move = (ev: PointerEvent) => setUi((u) => ({ ...u, split: Math.min(0.8, Math.max(0.2, (ev.clientY - box.top) / box.height)) }));
    const up = () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); };
    window.addEventListener("pointermove", move); window.addEventListener("pointerup", up);
    e.preventDefault();
  };

  if (!eng || !def || !f) return <AppShell section="workbench" crumbs={["Workbench"]}><EngineBoot error={loadErr} onRetry={retry} /></AppShell>;

  const ps = eng.paramSets.find((p) => p.id === f.param_set);
  const fields = visibleFields(def, f.mode);
  const nInvalid = Object.keys(errors).length;
  const nChanged = fields.filter((x) => { const v = parseNumber(f[x.key]); return v !== null && Math.abs(v - x.default) > 1e-12; }).length;

  const actions = (<>
    <Button size="sm" icon variant="ghost" aria-pressed={ui.insp} onClick={() => setUi((u) => ({ ...u, insp: !u.insp }))} aria-label="Toggle inspector" title="Inspector  [" className="wb-hide-m"><IconPanelLeft width={15} height={15} /></Button>
    <Button size="sm" icon variant="ghost" aria-pressed={ui.rail} onClick={() => setUi((u) => ({ ...u, rail: !u.rail }))} aria-label="Toggle verification rail" title="Verification  ]" className="wb-hide-m"><IconPanelRight width={15} height={15} /></Button>
    <Button size="sm" variant="ghost" onClick={() => setAssistant(true)}><IconSpark width={14} height={14} /><span className="wb-hide-m">Assistant</span></Button>
    <div className="wb-menu-wrap" ref={menuRef}>
      <Button size="sm" variant="primary" disabled={!res?.ok} aria-haspopup="menu" aria-expanded={menu} onClick={() => setMenu(!menu)}><IconExport width={14} height={14} />Export</Button>
      {menu && <div className="wb-menu" role="menu">
        <button role="menuitem" onClick={() => exportAs("html")}><b>Calculation sheet</b><span>.html · full working, A4 print layout</span></button>
        <button role="menuitem" onClick={() => exportAs("doc")}><b>Word document</b><span>.doc · editable in Microsoft Word</span></button>
        <button role="menuitem" onClick={() => exportAs("print")}><b>Print / PDF</b><span>browser print dialog</span></button>
      </div>}
    </div>
  </>);

  const status = (<>
    <span className="st"><StatusDot status={verdict} />{verdict === "none" ? (nInvalid ? `${nInvalid} invalid input${nInvalid > 1 ? "s" : ""}` : "Not calculated") : verdict === "results" ? "Design values" : `${VERDICT_WORD[verdict]}${verdict === "warn" ? " · review" : ""}`}</span>
    {gov && <span>η max {fmtUtil(gov.utilisation)} — {gov.name}</span>}
    <span className="grow" />
    <span>{nChanged} changed</span>
    <span>{f.param_set}</span>
    {tag && <span>trace {tag}</span>}
    <span>engine v{eng.version}</span>
  </>);

  const cols = `${ui.insp ? "340px" : "0px"} minmax(0, 1fr) ${ui.rail ? "300px" : "0px"}`;

  return (
    <AppShell section="workbench" crumbs={["Workbench", disciplineOf(def.id), def.name]} actions={actions} status={status} commands={commands}>
      <div className="wb" data-pane={pane} style={{ gridTemplateColumns: cols }}>
        <div className="wb-mtabs" role="tablist" aria-label="Workbench panes">
          {([["inputs", "Inputs"], ["model", "Model"], ["ledger", "Ledger"], ["verdict", "Verdict"]] as [MobilePane, string][]).map(([p, l]) => (
            <button key={p} role="tab" aria-selected={pane === p} onClick={() => setPane(p)}>{l}{p === "inputs" && nInvalid > 0 && <span className="badge fail">{nInvalid}</span>}{p === "verdict" && verdict !== "none" && <StatusDot status={verdict} />}</button>
          ))}
        </div>

        <aside className={`wb-insp${ui.insp ? "" : " off"}`} aria-label="Inspector">
          <div className="wb-insp-b">
            <Inspector defs={eng.defs} def={def} values={f} paramSets={eng.paramSets} errors={errors} collapsed={ui.collapsed[mid] ?? {}}
              onToggle={(g) => setUi((u) => ({ ...u, collapsed: { ...u.collapsed, [mid]: { ...(u.collapsed[mid] ?? {}), [g]: !(u.collapsed[mid]?.[g] ?? false) } } }))}
              onChange={setF} onModule={setMid} />
          </div>
          <div className="wb-insp-f">
            <span className="mono">{nInvalid > 0 ? <span className="t-fail">{nInvalid} invalid</span> : `${nChanged} of ${fields.length} changed`}</span>
            <span className="grow" />
            <Button size="sm" variant="ghost" onClick={resetAll} disabled={nChanged === 0}><IconReset width={13} height={13} />Reset</Button>
          </div>
        </aside>

        <div className="wb-center" ref={center} style={{ gridTemplateRows: `${ui.split * 100}% 7px minmax(0, 1fr)` }}>
          <div className="wb-vp"><Viewport moduleId={def.id} drawing={res?.ok ? res.drawing : null} values={payload ?? {}} state={res?.ok ? "ready" : "invalid"} /></div>
          <div className="wb-split" role="separator" aria-orientation="horizontal" aria-label="Resize viewport and ledger" tabIndex={0}
            aria-valuenow={Math.round(ui.split * 100)} aria-valuemin={20} aria-valuemax={80} onPointerDown={startSplit}
            onKeyDown={(e) => { if (e.key === "ArrowUp" || e.key === "ArrowDown") { e.preventDefault(); setUi((u) => ({ ...u, split: Math.min(0.8, Math.max(0.2, u.split + (e.key === "ArrowUp" ? -0.04 : 0.04))) })); } }} />
          <div className="wb-ledger">
            {notice && <div className="wb-pad"><Notice tone="info" onDismiss={() => setNotice("")}>{notice}</Notice></div>}
            {res?.ok ? <Ledger res={res} paramStatus={ps?.status ?? ""} tab={ledgerTab} onTab={setLedgerTab} /> : (
              <div className="wb-pad">
                <Notice tone="error" title="The calculation cannot run.">
                  {nInvalid > 0 ? `Correct ${nInvalid} parameter${nInvalid > 1 ? "s" : ""} marked in the inspector.` : "Review the inputs."}
                  {engineErr.general.length > 0 && <ul>{engineErr.general.map((e, i) => <li key={i}>{e}</li>)}</ul>}
                </Notice>
                <EmptyState glyph="Σ" title="Ledger pending">The verification ledger is generated from the engine trace as soon as every input is valid.</EmptyState>
              </div>
            )}
          </div>
        </div>

        <aside className={`wb-rail${ui.rail ? "" : " off"}`} aria-label="Verification">
          <div className="pane-h">Verification<span className="meta">{res?.ok ? `${res.checks.length} checks` : "—"}</span></div>
          <div className="wb-rail-b"><VerificationRail res={res} verdict={verdict} onOpenChecks={() => { setLedgerTab("checks"); setPane("ledger"); }} /></div>
        </aside>
      </div>
      <Assistant open={assistant} onClose={() => setAssistant(false)} def={def} values={f} res={res} onApply={(patch) => setF(patch)} />
    </AppShell>
  );
}

export default function WorkbenchPage() {
  return <Suspense fallback={<div className="boot"><span className="spinner" /></div>}><Workbench /></Suspense>;
}
