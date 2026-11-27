"use client";
import { useEffect, useMemo, useRef, useState } from "react";
import Link from "next/link";
import { DEFAULT_PARAM_SET } from "../../lib/engine";
import { useEngine } from "../../lib/useEngine";
import { supabase } from "../../lib/supabase";
import { askAssistant } from "../../lib/assistant";
import { parseNumber, visibleFields } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText } from "../../lib/download";
import type { Link as DepLink, ModDef, NodeResult, ProjectNode, Revision } from "../../lib/types";
import "../workspace/workspace.css";
import "./project.css";

const KEY = "strucal.project.v1";

/** Brings a stored node onto the current module definition: numeric values only, new fields get defaults. */
function normalizeNode(raw: unknown, defs: ModDef[], paramSets: string[]): ProjectNode | null {
  if (!raw || typeof raw !== "object") return null;
  const n = raw as Partial<ProjectNode> & Record<string, unknown>;
  if (typeof n.id !== "string" || typeof n.module !== "string") return null;
  const def = defs.find((d) => d.id === n.module);
  const values: Record<string, number> = {};
  const src = (n.values ?? {}) as Record<string, unknown>;
  if (def) for (const f of def.fields) { const v = src[f.key]; values[f.key] = typeof v === "number" && Number.isFinite(v) ? v : f.default; }
  else for (const [k, v] of Object.entries(src)) if (typeof v === "number") values[k] = v;
  const links: DepLink[] = Array.isArray(n.links) ? n.links.filter((l): l is DepLink => !!l && typeof l.field === "string" && typeof l.from === "string" && typeof l.output === "string").map((l) => ({ ...l, factor: typeof l.factor === "number" && Number.isFinite(l.factor) ? l.factor : 1 })) : [];
  return {
    id: n.id, name: typeof n.name === "string" ? n.name : n.id, module: n.module,
    mode: n.mode === "check" && def?.has_check_mode ? "check" : "design",
    param_set: typeof n.param_set === "string" && paramSets.includes(n.param_set) ? n.param_set : DEFAULT_PARAM_SET,
    values, links,
  };
}

/** Numeric input that keeps the typed text and only commits finite numbers. */
function NumField({ value, onCommit, readOnly, className, title, step }: { value: number | ""; onCommit?: (v: number) => void; readOnly?: boolean; className?: string; title?: string; step?: string }) {
  const [draft, setDraft] = useState(String(value));
  useEffect(() => { if (parseNumber(draft) !== value) setDraft(String(value)); /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, [value]);
  const bad = !readOnly && parseNumber(draft) === null;
  return <input type="text" inputMode="decimal" step={step} title={title} readOnly={readOnly} aria-invalid={bad} className={`${className ?? ""}${bad ? " bad" : ""}`} value={readOnly ? String(value) : draft}
    onChange={(e) => { setDraft(e.target.value); const v = parseNumber(e.target.value); if (v !== null) onCommit?.(v); }} />;
}

export default function ProjectPage() {
  const { engine: eng, error: loadErr, retry } = useEngine();
  const [nodes, setNodes] = useState<ProjectNode[]>([]);
  const [revs, setRevs] = useState<Revision[]>([]);
  const [base, setBase] = useState<string>("");
  const [add, setAdd] = useState("");
  const [loaded, setLoaded] = useState(false);
  const [notice, setNotice] = useState("");
  const [cloud, setCloud] = useState("");
  const [cloudName, setCloudName] = useState("");
  const [cloudReady, setCloudReady] = useState(false);
  const [ver, setVer] = useState(1);
  const [sync, setSync] = useState("");
  const [busy, setBusy] = useState(false);
  const [explain, setExplain] = useState(""); const [exBusy, setExBusy] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const synced = useRef("");

  const defs = useMemo(() => eng?.defs ?? [], [eng]);
  const psIds = useMemo(() => (eng?.paramSets ?? []).map((p) => p.id), [eng]);
  const norm = (list: unknown): ProjectNode[] => (Array.isArray(list) ? list.map((x) => normalizeNode(x, defs, psIds)).filter((x): x is ProjectNode => !!x) : []);

  // Initial load: cloud project from ?p=<id>, otherwise the local browser project.
  useEffect(() => {
    if (!eng) return;
    setAdd(eng.defs[0]?.id ?? "");
    const pid = new URLSearchParams(window.location.search).get("p") ?? "";
    if (pid) setCloud(pid);
    else {
      const o = readJson<{ nodes?: unknown; revs?: Revision[]; base?: string }>(KEY);
      if (o) { setNodes(norm(o.nodes)); const rv = Array.isArray(o.revs) ? o.revs.map((r) => ({ ...r, project: norm(r.project) })) : []; setRevs(rv); setBase(rv.some((r) => r.id === o.base) ? (o.base as string) : ""); }
    }
    setLoaded(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eng]);

  const loadCloud = async (pid: string, keepBase = false) => {
    setSync("Loading…");
    const { data: u } = await supabase.auth.getUser();
    if (!u.user) { setSync("Sign in on the Account page to use cloud projects."); return; }
    const { data: pr, error: pe } = await supabase.from("projects").select("name").eq("id", pid).single();
    if (pe || !pr) { setSync("Project not found, or you do not have access to it."); return; }
    setCloudName(pr.name);
    const { data: d, error: de } = await supabase.from("project_drafts").select("nodes, version").eq("project_id", pid).single();
    if (de) { setSync("Could not load the project draft: " + de.message); return; }
    if (d) { const list = norm(d.nodes); synced.current = JSON.stringify(list); setNodes(list); setVer(d.version); }
    const { data: rv, error: re } = await supabase.from("revisions").select("id, number, name, nodes, results, status, created_at").eq("project_id", pid).order("number");
    if (re) setNotice("Revisions could not be loaded: " + re.message);
    const mapped: Revision[] = (rv ?? []).map((r: { id: string; name: string; status: string; created_at: string; nodes: unknown; results: NodeResult[] }) => ({ id: r.id, name: `${r.name} (${r.status.replace(/_/g, " ")})`, savedAt: new Date(r.created_at).toLocaleString(), project: norm(r.nodes), results: r.results ?? [], status: r.status }));
    setRevs(mapped); if (!keepBase) setBase(mapped.length ? mapped[mapped.length - 1].id : "");
    setCloudReady(true); setSync("Saved to cloud");
  };
  useEffect(() => { if (cloud && eng) loadCloud(cloud); /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, [cloud, eng]);

  // Local projects save to browser storage on every change.
  useEffect(() => {
    if (!loaded || cloud) return;
    if (!writeJson(KEY, { nodes, revs, base })) setNotice("This browser is not storing the project (private mode or storage full). Export it to keep a copy.");
  }, [nodes, revs, base, cloud, loaded]);

  // Cloud drafts save debounced, only when the nodes differ from what was last loaded or saved, with an optimistic version check.
  useEffect(() => {
    if (!cloud || !cloudReady) return;
    const body = JSON.stringify(nodes);
    if (body === synced.current) return;
    setSync("Saving…");
    const t = setTimeout(async () => {
      const { data: u } = await supabase.auth.getUser();
      const { data, error } = await supabase.from("project_drafts").update({ nodes, version: ver + 1, updated_by: u.user?.id, updated_at: new Date().toISOString() }).eq("project_id", cloud).eq("version", ver).select("version");
      if (error) setSync("Not saved: " + error.message);
      else if (!data || data.length === 0) setSync("Conflict: someone else changed this project. Reload to get their changes before editing.");
      else { synced.current = body; setVer(data[0].version); setSync("Saved to cloud"); }
    }, 900);
    return () => clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nodes, cloud, cloudReady]);

  const baseline = revs.find((r) => r.id === base) ?? null;
  const res = useMemo(() => (eng && nodes.length ? eng.analyse(nodes, baseline ? baseline.results : null) : null), [eng, nodes, baseline]);

  const defOf = (id: string) => defs.find((d) => d.id === id);
  const nameOf = (id: string) => nodes.find((n) => n.id === id)?.name || id;
  const addNode = () => {
    const d = defOf(add); if (!d) return;
    const values: Record<string, number> = {}; for (const f of d.fields) values[f.key] = f.default;
    let n = 1; while (nodes.some((x) => x.id === `${d.id}${n}`)) n++;
    setNodes([...nodes, { id: `${d.id}${n}`, name: `${d.name} ${n}`, module: d.id, mode: "design", param_set: DEFAULT_PARAM_SET, values, links: [] }]);
  };
  const upd = (id: string, f: (n: ProjectNode) => ProjectNode) => setNodes((all) => all.map((n) => (n.id === id ? f(n) : n)));
  const del = (id: string) => {
    const dependants = nodes.filter((n) => n.links.some((l) => l.from === id)).map((n) => n.name);
    if (!confirm(`Remove ${nameOf(id)}?${dependants.length ? `\nLinks from it in ${dependants.join(", ")} become manual inputs.` : ""}`)) return;
    setNodes(nodes.filter((n) => n.id !== id).map((n) => ({ ...n, links: n.links.filter((l) => l.from !== id) })));
  };
  const saveRev = async () => {
    if (!res?.ok) return;
    setBusy(true);
    if (cloud) {
      const { error } = await supabase.rpc("save_revision", { p_project: cloud, p_name: `Rev ${revs.length + 1}`, p_nodes: nodes, p_results: res.nodes, p_engine: res.engine_version });
      if (error) setSync("Revision not saved: " + error.message); else { await loadCloud(cloud, true); setSync("Revision saved to cloud"); }
    } else {
      let k = revs.length + 1; while (revs.some((r) => r.id === `rev${k}`)) k++;
      const id = `rev${k}`;
      setRevs([...revs, { id, name: `Rev ${k}`, savedAt: new Date().toLocaleString(), project: structuredClone(nodes), results: res.nodes }]);
      setBase(id);
    }
    setBusy(false);
  };
  const restore = () => { if (baseline && confirm(`Replace the current inputs with ${baseline.name}? Saved revisions are kept.`)) setNodes(structuredClone(baseline.project)); };
  const exportJson = () => downloadText("strucal-project.json", JSON.stringify({ nodes, revisions: revs }, null, 2), "application/json");
  const importJson = async (file: File) => {
    try {
      const o = JSON.parse(await file.text());
      const list = norm(o?.nodes);
      if (list.length === 0) throw new Error("the file has no calculations");
      if (nodes.length && !confirm(`Replace the ${nodes.length} current calculation(s) with ${list.length} from ${file.name}?`)) return;
      setNodes(list);
      if (!cloud && Array.isArray(o.revisions)) { setRevs(o.revisions.map((r: Revision) => ({ ...r, project: norm(r.project) }))); setBase(""); }
      setNotice(`Imported ${list.length} calculation(s) from ${file.name}.`);
    } catch (e) { setNotice(`Could not import ${file.name}: ${e instanceof Error ? e.message : String(e)}`); }
  };
  const explainChanges = async () => {
    setExBusy(true); setExplain("");
    try {
      const names = Object.fromEntries(nodes.map((n) => [n.id, `${n.name} (${n.module})`]));
      setExplain(await askAssistant("compare", "", { baseline: baseline?.name, impact: res?.impact, calculations: names, results: res?.nodes.map((n) => ({ id: n.id, name: n.name, status: n.status, pass: n.overall_pass, max_util: n.max_util, outputs: n.outputs })) }));
    } catch (e) { setExplain("Assistant: " + (e instanceof Error ? e.message : String(e))); }
    setExBusy(false);
  };
  const rOf = (id: string) => res?.nodes.find((n) => n.id === id);
  const imp = res?.impact;

  if (!eng) return <div className="pj"><div className="wk-title"><Link href="/" className="wk-logo" aria-label="StruCal home">S</Link><div className="wk-wt">StruCal — Project</div></div><div className="pj-empty" role="status">{loadErr ? <>{loadErr} <button className="pj-btn" onClick={retry}>Try again</button></> : "Loading calculation engine…"}</div></div>;
  return (
    <div className="pj">
      <div className="wk-title"><Link href="/" className="wk-logo" title="StruCal - home">S</Link><div className="wk-wt">StruCal — Project dependencies{cloud ? ` — ${cloudName || "cloud project"}` : ""} — {nodes.length} calculation(s){cloud && sync ? ` — ${sync}` : ""}</div></div>
      <div className="wk-tabs"><span className="file">PROJECT</span><Link href="/workspace" style={{ padding: "0 14px", lineHeight: "24px" }}>Back to workspace</Link><Link href="/account" style={{ padding: "0 14px", lineHeight: "24px" }}>Account and teams</Link></div>
      <div className="wk-ribbon" style={{ height: 62, alignItems: "center", gap: 10, padding: "0 10px", flexWrap: "wrap" }}>
        <select className="pj-btn" value={add} onChange={(e) => setAdd(e.target.value)} style={{ height: 26 }} aria-label="Module to add">{defs.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}</select>
        <button className="pj-btn pri" onClick={addNode}>+ Add calculation</button>
        <button className="pj-btn" onClick={saveRev} disabled={!res?.ok || busy}>{busy ? "Saving…" : cloud ? "Save revision to cloud" : "Save revision"}</button>
        <button className="pj-btn" onClick={exportJson} disabled={nodes.length === 0}>Export (.json)</button>
        <button className="pj-btn" onClick={() => fileRef.current?.click()}>Import (.json)</button>
        <input ref={fileRef} type="file" accept="application/json,.json" hidden onChange={(e) => { const f = e.target.files?.[0]; if (f) importJson(f); e.target.value = ""; }} />
        <span style={{ color: "#52606d" }}>Compare against:</span>
        <select className="pj-btn" value={base} onChange={(e) => setBase(e.target.value)} style={{ height: 26 }} aria-label="Baseline revision"><option value="">(no baseline)</option>{revs.map((r) => <option key={r.id} value={r.id}>{r.name} · {r.savedAt}</option>)}</select>
        <button className="pj-btn" onClick={restore} disabled={!baseline}>Restore baseline inputs</button>
      </div>
      <div className="pj-body">
        <div className="pj-main">
          {notice && <div className="pj-warn" role="status">{notice} <button className="pj-btn" onClick={() => setNotice("")}>Dismiss</button></div>}
          {nodes.length === 0 && <div className="pj-card pj-empty">No calculations yet. Add calculations and link one result to the next input, for example a wall&apos;s ULS load to a strip footing&apos;s load.</div>}
          {res && !res.ok && <div className="pj-err" role="alert">{res.errors.map((e, i) => <div key={i}>{e}</div>)}</div>}
          {nodes.map((n) => {
            const d = defOf(n.module); const r = rOf(n.id);
            if (!d) return (<div className="pj-card" key={n.id}><div className="pj-ch"><b>{n.name}</b><span className="pj-b fail">UNKNOWN MODULE</span><span style={{ flex: 1 }} /><button className="pj-btn" onClick={() => del(n.id)}>Remove</button></div><div className="pj-err">Module &quot;{n.module}&quot; is not available in engine v{eng.version}. Remove this calculation or open the project in a matching version.</div></div>);
            const linked = (k: string) => n.links.find((l) => l.field === k);
            const options = nodes.filter((o) => o.id !== n.id).flatMap((o) => (rOf(o.id)?.outputs ?? []).map((out) => ({ v: `${o.id}|${out.key}`, label: `${o.name} → ${out.label} (${out.unit})` })));
            const badge = !r ? null : r.status === "blocked" ? <span className="pj-b blk">BLOCKED</span> : r.status === "error" ? <span className="pj-b fail">INPUT ERROR</span> : !r.has_verdict ? <span className="pj-b res">RESULTS</span> : r.overall_pass ? <span className="pj-b pass">PASS {(r.max_util * 100).toFixed(0)}%</span> : <span className="pj-b fail">FAIL {(r.max_util * 100).toFixed(0)}%</span>;
            const stale = imp && (imp.edited.includes(n.id) || imp.affected.includes(n.id));
            return (
              <div className="pj-card" key={n.id}>
                <div className="pj-ch">
                  <input className="nm" value={n.name} aria-label="Calculation name" onChange={(e) => upd(n.id, (x) => ({ ...x, name: e.target.value }))} />
                  <span className="pj-tag">{d.name}</span>{badge}
                  {stale && <span className="pj-b blk" title="Changed since the baseline revision">{imp.edited.includes(n.id) ? "EDITED" : "AFFECTED"}</span>}
                  <span style={{ flex: 1 }} />
                  {d.has_check_mode && <select value={n.mode} aria-label="Mode" onChange={(e) => upd(n.id, (x) => ({ ...x, mode: e.target.value === "check" ? "check" : "design" }))}><option value="design">Design</option><option value="check">Check</option></select>}
                  <select value={n.param_set} aria-label="Parameter set" onChange={(e) => upd(n.id, (x) => ({ ...x, param_set: e.target.value }))} style={{ maxWidth: 220 }}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
                  <button className="pj-btn" onClick={() => del(n.id)}>Remove</button>
                </div>
                <div className="pj-grid">
                  {visibleFields(d, n.mode).map((f) => {
                    const l = linked(f.key); const rv = r?.resolved.find((x) => x.field === f.key);
                    const setVal = (v: number) => upd(n.id, (x) => ({ ...x, values: { ...x.values, [f.key]: v } }));
                    return (<div className="pj-f" key={f.key}><label>{f.label}{f.unit ? ` (${f.unit})` : ""}</label>
                      <div className="row">
                        {f.options && !l ? (<select value={n.values[f.key]} onChange={(e) => setVal(Number(e.target.value))} style={{ flex: 1, maxWidth: "none" }}>{f.options.map(([v, lab]) => <option key={v} value={v}>{lab}</option>)}</select>)
                          : l ? <NumField readOnly className="lk" value={rv ? Number(rv.value.toFixed(4)) : ""} /> : <NumField value={n.values[f.key]} onCommit={setVal} />}
                        <select value={l ? `${l.from}|${l.output}` : ""} title="Link this input to a result of another calculation" aria-label={`Source for ${f.label}`} onChange={(e) => upd(n.id, (x) => ({ ...x, links: [...x.links.filter((k) => k.field !== f.key), ...(e.target.value ? [{ field: f.key, from: e.target.value.split("|")[0], output: e.target.value.split("|")[1], factor: l?.factor ?? 1 }] : [])] }))}>
                          <option value="">Manual</option>{options.map((o) => <option key={o.v} value={o.v}>{o.label}</option>)}
                          {l && !options.some((o) => o.v === `${l.from}|${l.output}`) && <option value={`${l.from}|${l.output}`}>{nameOf(l.from)} → {l.output} (unavailable)</option>}
                        </select>
                        {l && <NumField className="fac" step="0.1" title="Multiplier applied to the linked value" value={l.factor} onCommit={(v) => upd(n.id, (x) => ({ ...x, links: x.links.map((k) => (k.field === f.key ? { ...k, factor: v } : k)) }))} />}
                      </div></div>);
                  })}
                </div>
                {r && r.errors.length > 0 && <div className="pj-err">{r.errors.map((e, i) => <div key={i}>{e}</div>)}</div>}
                {r && r.outputs.length > 0 && <div className="pj-out">{r.outputs.map((o) => <span key={o.key}>{o.label}: <b>{o.value.toFixed(2)}</b> {o.unit}</span>)}</div>}
              </div>
            );
          })}
        </div>
        <aside className="pj-side pj-imp">
          <h3>Project status</h3>
          {res?.ok ? <div><span className={`pj-b ${res.overall_pass ? "pass" : "fail"}`}>{res.overall_pass ? "ALL CHECKS PASS" : "ATTENTION REQUIRED"}</span><div style={{ marginTop: 6, color: "#52606d" }}>Calculation order: {res.order.map((i) => nameOf(i)).join(" → ")}</div></div> : <div style={{ color: "#52606d" }}>{nodes.length ? "Resolve the project errors to see its status." : "Add calculations to begin."}</div>}
          <h3>Change impact</h3>
          {!baseline && <div style={{ color: "#52606d" }}>Save a revision, then edit inputs to see what changes. Historical results are never overwritten.</div>}
          {baseline && imp && (<>
            <div className="row"><span>Compared with</span><b>{baseline.name}</b></div>
            <div className="row"><span>Edited</span><b>{imp.edited.length}</b></div>
            <div className="row"><span>Affected downstream</span><b>{imp.affected.length}</b></div>
            <div className="row"><span>Newly failing</span><b style={{ color: imp.newly_failing.length ? "#b42318" : undefined }}>{imp.newly_failing.length}</b></div>
            <div className="row"><span>Newly passing</span><b>{imp.newly_passing.length}</b></div>
            {imp.review_required && <div className="pj-warn">Results have changed since {baseline.name}. They must be reviewed before the next issue.</div>}
            {imp.newly_failing.length > 0 && <div className="pj-err">Now failing: {imp.newly_failing.map((i) => nameOf(i)).join(", ")}</div>}
            {imp.diffs.length > 0 && (<><h3>Output changes</h3>{imp.diffs.map((d, i) => <div className="row" key={i}><span>{nameOf(d.node)} · {d.output}</span><b>{d.old.toFixed(1)} → {d.new.toFixed(1)}</b></div>)}</>)}
            <div style={{ margin: "8px 0" }}><button className="pj-btn" disabled={exBusy} onClick={explainChanges}>{exBusy ? "Thinking…" : "Explain these changes (AI)"}</button></div>
            {explain && <div className="pj-warn" style={{ whiteSpace: "pre-wrap" }}>{explain}<div style={{ fontSize: 11, color: "#7b8794", marginTop: 4 }}>AI-generated summary of the change impact above. It does not change any result.</div></div>}
            {imp.added.length > 0 && <div style={{ marginTop: 6 }}>Added: {imp.added.map((i) => nameOf(i)).join(", ")}</div>}
            {imp.removed.length > 0 && <div style={{ marginTop: 6 }}>Removed: {imp.removed.join(", ")}</div>}
          </>)}
          <h3>Revisions</h3>
          {revs.length === 0 ? <div style={{ color: "#52606d" }}>None saved.</div> : <ul className="pj-list">{revs.map((r) => <li key={r.id}>{r.name} · {r.savedAt}</li>)}</ul>}
          {!cloud && <div className="pj-warn" style={{ marginTop: 12 }}>Working locally in this browser. Sign in on the Account page and open a project from there to save it to the cloud with approvals.</div>}
          {cloud && sync.startsWith("Conflict") && <div className="pj-err">{sync} <button className="pj-btn" onClick={() => window.location.reload()}>Reload</button></div>}
          <div className="pj-warn" style={{ marginTop: 12 }}>Links pass design values only. Combine load effects with the Load combinations module first, and confirm each linked value is the correct design action for the next member.</div>
        </aside>
      </div>
    </div>
  );
}
