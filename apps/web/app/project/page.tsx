"use client";
import { useEffect, useMemo, useRef, useState } from "react";
import TopBar from "../components/TopBar";
import { DEFAULT_PARAM_SET } from "../../lib/engine";
import { useEngine } from "../../lib/useEngine";
import { supabase } from "../../lib/supabase";
import { askAssistant } from "../../lib/assistant";
import { parseNumber, visibleFields } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText } from "../../lib/download";
import type { Link as DepLink, ModDef, NodeResult, ProjectNode, Revision } from "../../lib/types";
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
function NumField({ value, onCommit, readOnly, className, label }: { value: number | ""; onCommit?: (v: number) => void; readOnly?: boolean; className?: string; label: string }) {
  const [draft, setDraft] = useState(String(value));
  useEffect(() => { if (parseNumber(draft) !== value) setDraft(String(value)); /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, [value]);
  const bad = !readOnly && parseNumber(draft) === null;
  return <input type="text" inputMode="decimal" aria-label={label} readOnly={readOnly} aria-invalid={bad} className={`input num ${className ?? ""}`} value={readOnly ? String(value) : draft}
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

  const statusBadge = (r: NodeResult | undefined) => !r ? null
    : r.status === "blocked" ? <span className="badge warn">Blocked</span>
    : r.status === "error" ? <span className="badge fail">Input error</span>
    : !r.has_verdict ? <span className="badge info">Results</span>
    : r.overall_pass ? <span className="badge ok">Pass {(r.max_util * 100).toFixed(0)}%</span>
    : <span className="badge fail">Fail {(r.max_util * 100).toFixed(0)}%</span>;

  if (!eng) return (
    <div className="app"><TopBar section="project" />
      <div className="loading" role="status">{loadErr ? <div className="empty"><h2>The calculation engine did not load</h2><p>{loadErr}</p><button className="btn btn-primary" onClick={retry}>Try again</button></div> : <><span className="spinner" />Loading calculation engine…</>}</div>
    </div>
  );
  return (
    <div className="app">
      <TopBar section="project" context={cloud ? <>{cloudName || "Cloud project"} · {sync}</> : "Local project in this browser"} />
      <div className="toolbar pj-bar" role="toolbar" aria-label="Project">
        <select className="select" value={add} onChange={(e) => setAdd(e.target.value)} aria-label="Module to add">{defs.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}</select>
        <button className="btn btn-primary" onClick={addNode}>Add calculation</button>
        <span className="sep" />
        <button className="btn" onClick={saveRev} disabled={!res?.ok || busy}>{busy ? "Saving…" : cloud ? "Save revision to cloud" : "Save revision"}</button>
        <select className="select" value={base} onChange={(e) => setBase(e.target.value)} aria-label="Baseline revision"><option value="">No baseline</option>{revs.map((r) => <option key={r.id} value={r.id}>Compare with {r.name}</option>)}</select>
        <button className="btn btn-ghost" onClick={restore} disabled={!baseline}>Restore baseline</button>
        <span className="spacer" />
        {cloud && <span className="pj-sync" role="status">{sync}</span>}
        <button className="btn btn-ghost" onClick={() => fileRef.current?.click()}>Import</button>
        <button className="btn btn-ghost" onClick={exportJson} disabled={nodes.length === 0}>Export</button>
        <input ref={fileRef} type="file" accept="application/json,.json" hidden onChange={(e) => { const f = e.target.files?.[0]; if (f) importJson(f); e.target.value = ""; }} />
      </div>

      <div className="pj-layout">
        <main className="pj-main">
          <header className="page-head"><span className="eyebrow">{cloud ? "Cloud project" : "Local project"}</span><h1>{cloud ? cloudName || "Project" : "Project dependencies"}</h1><p>Link one calculation&apos;s design value to the next calculation&apos;s input. Edits propagate in dependency order and are compared against a saved revision.</p></header>
          {notice && <div className="alert info" role="status"><span className="grow">{notice}</span><button className="btn btn-sm" onClick={() => setNotice("")}>Dismiss</button></div>}
          {cloud && sync.startsWith("Conflict") && <div className="alert error" role="alert"><span className="grow">{sync}</span><button className="btn btn-sm" onClick={() => window.location.reload()}>Reload</button></div>}
          {res && !res.ok && <div className="alert error" role="alert"><div className="grow"><b>The project cannot be analysed.</b><ul>{res.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></div></div>}
          {nodes.length === 0 && (
            <div className="card"><div className="empty">
              <h2>No calculations yet</h2>
              <ol className="steps">
                <li>Choose a module in the toolbar and add it, for example an RC wall.</li>
                <li>Add the member it bears on, for example a strip footing.</li>
                <li>On the footing&apos;s load input, pick the wall&apos;s ULS load as its source.</li>
                <li>Save a revision. Later edits show what changed downstream.</li>
              </ol>
              <button className="btn btn-primary" onClick={addNode}>Add {defOf(add)?.name ?? "calculation"}</button>
            </div></div>
          )}
          {nodes.map((n) => {
            const d = defOf(n.module); const r = rOf(n.id);
            if (!d) return (
              <section className="card" key={n.id}>
                <div className="card-h"><h2>{n.name}</h2><span className="badge fail">Unknown module</span><span className="spacer" /><button className="btn btn-sm btn-danger" onClick={() => del(n.id)}>Remove</button></div>
                <div className="card-b"><div className="alert error"><span className="grow">Module &quot;{n.module}&quot; is not available in engine v{eng.version}. Remove this calculation or open the project in a matching version.</span></div></div>
              </section>
            );
            const linked = (k: string) => n.links.find((l) => l.field === k);
            const options = nodes.filter((o) => o.id !== n.id).flatMap((o) => (rOf(o.id)?.outputs ?? []).map((out) => ({ v: `${o.id}|${out.key}`, label: `${o.name} → ${out.label} (${out.unit})` })));
            const changed = imp && (imp.edited.includes(n.id) ? "Edited" : imp.affected.includes(n.id) ? "Affected" : "");
            return (
              <section className="card" key={n.id} aria-label={n.name}>
                <div className="card-h node-h">
                  <input className="nm" value={n.name} aria-label="Calculation name" onChange={(e) => upd(n.id, (x) => ({ ...x, name: e.target.value }))} />
                  <span className="badge">{d.name}</span>{statusBadge(r)}
                  {changed && <span className="badge warn" title="Changed since the baseline revision">{changed}</span>}
                  <span className="spacer" />
                  {d.has_check_mode && <select className="select" value={n.mode} aria-label="Mode" onChange={(e) => upd(n.id, (x) => ({ ...x, mode: e.target.value === "check" ? "check" : "design" }))}><option value="design">Design</option><option value="check">Check</option></select>}
                  <select className="select ps" value={n.param_set} aria-label="Parameter set" onChange={(e) => upd(n.id, (x) => ({ ...x, param_set: e.target.value }))}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
                  <button className="btn btn-sm btn-danger" onClick={() => del(n.id)}>Remove</button>
                </div>
                <div className="card-b">
                  <div className="node-grid">
                    {visibleFields(d, n.mode).map((f) => {
                      const l = linked(f.key); const rv = r?.resolved.find((x) => x.field === f.key);
                      const setVal = (v: number) => upd(n.id, (x) => ({ ...x, values: { ...x.values, [f.key]: v } }));
                      const label = `${f.label}${f.unit ? ` (${f.unit})` : ""}`;
                      return (
                        <div className="node-field" key={f.key}>
                          <span className="label" title={label}>{label}</span>
                          <div className={`row${l ? " linked" : ""}`}>
                            {f.options && !l ? (<select className="select" aria-label={f.label} value={n.values[f.key]} onChange={(e) => setVal(Number(e.target.value))}>{f.options.map(([v, lab]) => <option key={v} value={v}>{lab}</option>)}</select>)
                              : l ? <NumField readOnly label={`${f.label} (linked)`} value={rv ? Number(rv.value.toFixed(4)) : ""} /> : <NumField label={f.label} value={n.values[f.key]} onCommit={setVal} />}
                            <select className={`select src${l ? " on" : ""}`} value={l ? `${l.from}|${l.output}` : ""} title="Source of this input" aria-label={`Source for ${f.label}`} onChange={(e) => upd(n.id, (x) => ({ ...x, links: [...x.links.filter((k) => k.field !== f.key), ...(e.target.value ? [{ field: f.key, from: e.target.value.split("|")[0], output: e.target.value.split("|")[1], factor: l?.factor ?? 1 }] : [])] }))}>
                              <option value="">Manual</option>{options.map((o) => <option key={o.v} value={o.v}>{o.label}</option>)}
                              {l && !options.some((o) => o.v === `${l.from}|${l.output}`) && <option value={`${l.from}|${l.output}`}>{nameOf(l.from)} → {l.output} (unavailable)</option>}
                            </select>
                            {l && <NumField className="fac" label={`Multiplier for ${f.label}`} value={l.factor} onCommit={(v) => upd(n.id, (x) => ({ ...x, links: x.links.map((k) => (k.field === f.key ? { ...k, factor: v } : k)) }))} />}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                  {r && r.errors.length > 0 && <div className="alert error"><ul className="grow">{r.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></div>}
                </div>
                {r && r.outputs.length > 0 && <div className="card-f node-out">{r.outputs.map((o) => <span key={o.key}>{o.label} <b>{o.value.toFixed(2)}</b> {o.unit}</span>)}</div>}
              </section>
            );
          })}
        </main>

        <aside className="pj-side" aria-label="Project status">
          <section className="card">
            <div className="card-h"><h2>Status</h2><span className="spacer" />{res?.ok && <span className={`badge ${res.overall_pass ? "ok" : "fail"}`}>{res.overall_pass ? "All pass" : "Attention"}</span>}</div>
            <div className="card-b">
              {res?.ok ? (<><span className="label">Calculation order</span><div className="order">{res.order.map((i) => <span key={i}>{nameOf(i)}</span>)}</div></>)
                : <p className="muted">{nodes.length ? "Resolve the project errors to see its status." : "Add calculations to begin."}</p>}
            </div>
          </section>
          <section className="card">
            <div className="card-h"><h2>Change impact</h2></div>
            <div className="card-b">
              {!baseline && <p className="muted">Save a revision, then edit inputs to see what changes. Saved results are never overwritten.</p>}
              {baseline && imp && (<>
                <div>
                  <div className="stat-row"><span>Compared with</span><b>{baseline.name}</b></div>
                  <div className="stat-row"><span>Edited</span><b>{imp.edited.length}</b></div>
                  <div className="stat-row"><span>Affected downstream</span><b>{imp.affected.length}</b></div>
                  <div className="stat-row"><span>Newly failing</span><b className={imp.newly_failing.length ? "fail" : ""}>{imp.newly_failing.length}</b></div>
                  <div className="stat-row"><span>Newly passing</span><b>{imp.newly_passing.length}</b></div>
                </div>
                {imp.review_required && <div className="alert warn"><span className="grow">Results changed since {baseline.name}. Review before the next issue.</span></div>}
                {imp.newly_failing.length > 0 && <div className="alert error"><span className="grow">Now failing: {imp.newly_failing.map((i) => nameOf(i)).join(", ")}</span></div>}
                {imp.diffs.length > 0 && (<div><span className="label">Output changes</span>{imp.diffs.map((d, i) => <div className="stat-row" key={i}><span>{nameOf(d.node)} · {d.output}</span><b>{d.old.toFixed(1)} → {d.new.toFixed(1)}</b></div>)}</div>)}
                {(imp.added.length > 0 || imp.removed.length > 0) && <p className="muted" style={{ fontSize: "var(--fs-xs)" }}>{imp.added.length > 0 && <>Added: {imp.added.map((i) => nameOf(i)).join(", ")}. </>}{imp.removed.length > 0 && <>Removed: {imp.removed.join(", ")}.</>}</p>}
                <button className="btn" disabled={exBusy} onClick={explainChanges}>{exBusy ? "Thinking…" : "Explain these changes (AI)"}</button>
                {explain && <div className="alert info"><div className="grow" style={{ whiteSpace: "pre-wrap" }}>{explain}<div className="faint" style={{ fontSize: "var(--fs-2xs)", marginTop: 4 }}>AI-generated summary of the change impact. It does not change any result.</div></div></div>}
              </>)}
            </div>
          </section>
          <section className="card">
            <div className="card-h"><h2>Revisions</h2><span className="spacer" /><span className="num faint">{revs.length}</span></div>
            <div className="card-b">{revs.length === 0 ? <p className="muted">None saved.</p> : <ul className="rev-list">{revs.map((r) => <li key={r.id}><span>{r.name}</span><span className="faint">{r.savedAt}</span></li>)}</ul>}</div>
          </section>
          <div className="alert info"><span className="grow">{cloud ? "Changes save to the cloud automatically." : "Working locally in this browser. Open a project from the Account page to save it to the cloud with approvals."} Links pass design values only: combine load effects with the Load combinations module first.</span></div>
        </aside>
      </div>
    </div>
  );
}
