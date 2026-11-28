"use client";
import { useEffect, useMemo, useRef, useState } from "react";
import TopBar from "../components/TopBar";
import { DEFAULT_PARAM_SET } from "../../lib/engine";
import { useEngine } from "../../lib/useEngine";
import { supabase } from "../../lib/supabase";
import { askAssistant } from "../../lib/assistant";
import { DISCIPLINES, disciplineOf, parseNumber, visibleFields } from "../../lib/form";
import { REVIEW_THRESHOLD, fmtUtil } from "../../lib/status";
import { Button, EmptyState, EngineBoot, Notice, Segmented, StatusBadge, StatusDot } from "../ui/primitives";
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
  return <input type="text" inputMode="decimal" aria-label={label} readOnly={readOnly} aria-invalid={bad} className={className ?? "input num"} value={readOnly ? String(value) : draft}
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
  const [sel, setSel] = useState("");
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
    setSel(`${d.id}${n}`);
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

  const statusOf = (r: NodeResult | undefined): "ok" | "warn" | "fail" | "results" | "none" =>
    !r ? "none" : r.status !== "calculated" ? "fail" : !r.has_verdict ? "results" : !r.overall_pass ? "fail" : r.max_util > REVIEW_THRESHOLD ? "warn" : "ok";
  const statusText = (r: NodeResult | undefined) =>
    !r ? "—" : r.status === "blocked" ? "Blocked" : r.status === "error" ? "Input error" : !r.has_verdict ? "Results" : `${r.overall_pass ? "Pass" : "Fail"} · η ${fmtUtil(r.max_util)}`;

  if (!eng) return <div className="app"><TopBar section="project" /><EngineBoot error={loadErr} onRetry={retry} /></div>;

  const ordered = res?.ok ? res.order.map((id) => nodes.find((n) => n.id === id)).filter((n): n is ProjectNode => !!n) : nodes;
  const cur = nodes.find((n) => n.id === sel) ?? ordered[0];
  const d = cur ? defOf(cur.module) : undefined;
  const r = cur ? rOf(cur.id) : undefined;
  const changedMark = (id: string) => (imp ? (imp.edited.includes(id) ? "edited" : imp.affected.includes(id) ? "affected" : "") : "");
  const options = cur ? nodes.filter((o) => o.id !== cur.id).flatMap((o) => (rOf(o.id)?.outputs ?? []).map((out) => ({ v: `${o.id}|${out.key}`, label: `${o.name} → ${out.label} (${out.unit})` }))) : [];
  const usedBy = (nodeId: string, key: string) => nodes.filter((n) => n.links.some((l) => l.from === nodeId && l.output === key)).map((n) => n.name);

  return (
    <div className="app pj">
      <TopBar section="project" context={cloud ? <>{cloudName || "Cloud project"} · {sync}</> : "Local project · this browser"} />
      <div className="toolbar" role="toolbar" aria-label="Project">
        <span className="lbl">Add</span>
        <select className="select pj-add" value={add} onChange={(e) => setAdd(e.target.value)} aria-label="Module to add">{DISCIPLINES.map((g) => <optgroup key={g} label={g}>{defs.filter((x) => disciplineOf(x.id) === g).map((x) => <option key={x.id} value={x.id}>{x.name}</option>)}</optgroup>)}</select>
        <Button variant="primary" onClick={addNode}>Add calculation</Button>
        <span className="sep" />
        <Button onClick={saveRev} disabled={!res?.ok || busy}>{busy ? "Saving…" : cloud ? "Save revision to cloud" : "Save revision"}</Button>
        <select className="select pj-base" value={base} onChange={(e) => setBase(e.target.value)} aria-label="Baseline revision"><option value="">No baseline</option>{revs.map((rv) => <option key={rv.id} value={rv.id}>Compare with {rv.name}</option>)}</select>
        <Button variant="ghost" onClick={restore} disabled={!baseline}>Restore baseline</Button>
        <span className="spacer" />
        <Button variant="ghost" onClick={() => fileRef.current?.click()}>Import</Button>
        <Button variant="ghost" onClick={exportJson} disabled={nodes.length === 0}>Export</Button>
        <input ref={fileRef} type="file" accept="application/json,.json" hidden onChange={(e) => { const fl = e.target.files?.[0]; if (fl) importJson(fl); e.target.value = ""; }} />
      </div>

      <div className="pj-grid">
        <section className="pane pj-list" aria-label="Calculations">
          <div className="pane-h">Calculations<span className="meta">{nodes.length} · dependency order</span></div>
          <div className="pane-b">
            {nodes.length === 0 ? <p className="pj-list-empty">No calculations yet.</p> : (
              <ol className="pj-nodes">
                {ordered.map((n, i) => {
                  const nr = rOf(n.id); const mark = changedMark(n.id);
                  return (
                    <li key={n.id}>
                      <button type="button" className={`pj-node${cur?.id === n.id ? " on" : ""}`} aria-current={cur?.id === n.id} onClick={() => setSel(n.id)}>
                        <span className="idx mono">{String(i + 1).padStart(2, "0")}</span>
                        <span className="nm">{n.name}</span>
                        <StatusDot status={statusOf(nr)} />
                        <span className="sub mono">{defOf(n.module)?.name ?? n.module}{n.links.length > 0 && ` · ${n.links.length} link${n.links.length > 1 ? "s" : ""}`}</span>
                        <span className="st mono">{statusText(nr)}</span>
                        {mark && <span className="mark">{mark}</span>}
                      </button>
                    </li>
                  );
                })}
              </ol>
            )}
          </div>
        </section>

        <main className="pj-main" aria-label="Selected calculation">
          {notice && <Notice tone="info" onDismiss={() => setNotice("")}>{notice}</Notice>}
          {cloud && sync.startsWith("Conflict") && <div className="alert error" role="alert"><span className="grow">{sync}</span><Button size="sm" onClick={() => window.location.reload()}>Reload</Button></div>}
          {res && !res.ok && <Notice tone="error" title="The project cannot be analysed."><ul>{res.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></Notice>}

          {!cur && (
            <div className="pj-empty">
              <EmptyState glyph="→" title="Link calculations into a load path">Each calculation can take its inputs from another calculation&apos;s outputs. Edits propagate in dependency order and are compared against a saved revision.</EmptyState>
              <ol className="pj-steps">
                <li>Add an RC wall from the toolbar.</li>
                <li>Add the strip footing it bears on.</li>
                <li>Set the footing&apos;s load source to the wall&apos;s ULS load.</li>
                <li>Save a revision; later edits show what changed downstream.</li>
              </ol>
              <Button variant="primary" onClick={addNode}>Add {defOf(add)?.name ?? "calculation"}</Button>
            </div>
          )}

          {cur && !d && (
            <section className="pj-editor">
              <header className="pj-ed-head"><h1>{cur.name}</h1><StatusBadge status="fail">Unknown module</StatusBadge><span className="spacer" /><Button size="sm" variant="danger" onClick={() => del(cur.id)}>Remove</Button></header>
              <Notice tone="error">Module &quot;{cur.module}&quot; is not available in engine v{eng.version}. Remove this calculation or open the project in a matching version.</Notice>
            </section>
          )}

          {cur && d && (
            <section className="pj-editor">
              <header className="pj-ed-head">
                <input className="nm" value={cur.name} aria-label="Calculation name" onChange={(e) => upd(cur.id, (x) => ({ ...x, name: e.target.value }))} />
                <span className="badge">{d.name}</span>
                <StatusBadge status={statusOf(r)}><span className="nt">{statusText(r)}</span></StatusBadge>
                {changedMark(cur.id) && <span className="badge warn">{changedMark(cur.id)}</span>}
                <span className="spacer" />
                <Segmented label="Mode" value={cur.mode} onChange={(m) => upd(cur.id, (x) => ({ ...x, mode: m }))} options={[{ value: "design", label: "Design" }, { value: "check", label: "Check", disabled: !d.has_check_mode }]} />
                <select className="select pj-ps" value={cur.param_set} aria-label="Parameter set" onChange={(e) => upd(cur.id, (x) => ({ ...x, param_set: e.target.value }))}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
                <Button size="sm" variant="danger" onClick={() => del(cur.id)}>Remove</Button>
              </header>
              <p className="pj-ed-sub mono">{d.subtitle}</p>
              {r && r.errors.length > 0 && <Notice tone="error" title={r.status === "blocked" ? "Blocked by an upstream calculation." : "Input error."}><ul>{r.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></Notice>}

              <div className="pj-sec">
                <div className="lprop lhead" aria-hidden><span>Parameter</span><span className="r">Value</span><span>Unit</span><span>Source</span><span className="r">×</span></div>
                {visibleFields(d, cur.mode).map((f) => {
                  const l = cur.links.find((k) => k.field === f.key); const rv = r?.resolved.find((x) => x.field === f.key);
                  const setVal = (v: number) => upd(cur.id, (x) => ({ ...x, values: { ...x.values, [f.key]: v } }));
                  const modified = !l && Math.abs((cur.values[f.key] ?? f.default) - f.default) > 1e-12;
                  return (
                    <div className={`lprop${l ? " linked" : modified ? " modified" : ""}`} key={f.key}>
                      <label htmlFor={`pv-${f.key}`}>{f.label}</label>
                      {f.options && !l
                        ? <select id={`pv-${f.key}`} className="val sel" value={cur.values[f.key]} onChange={(e) => setVal(Number(e.target.value))}>{f.options.map(([v, lab]) => <option key={v} value={v}>{lab}</option>)}</select>
                        : l ? <NumField readOnly className="val" label={`${f.label} (linked)`} value={rv ? Number(rv.value.toFixed(4)) : ""} /> : <NumField className="val" label={f.label} value={cur.values[f.key]} onCommit={setVal} />}
                      <span className="u">{f.unit}</span>
                      <select className={`src${l ? " on" : ""}`} value={l ? `${l.from}|${l.output}` : ""} aria-label={`Source for ${f.label}`} onChange={(e) => upd(cur.id, (x) => ({ ...x, links: [...x.links.filter((k) => k.field !== f.key), ...(e.target.value ? [{ field: f.key, from: e.target.value.split("|")[0], output: e.target.value.split("|")[1], factor: l?.factor ?? 1 }] : [])] }))}>
                        <option value="">Manual</option>{options.map((o) => <option key={o.v} value={o.v}>{o.label}</option>)}
                        {l && !options.some((o) => o.v === `${l.from}|${l.output}`) && <option value={`${l.from}|${l.output}`}>{nameOf(l.from)} → {l.output} (unavailable)</option>}
                      </select>
                      {l ? <NumField className="val fac" label={`Multiplier for ${f.label}`} value={l.factor} onCommit={(v) => upd(cur.id, (x) => ({ ...x, links: x.links.map((k) => (k.field === f.key ? { ...k, factor: v } : k)) }))} /> : <span />}
                    </div>
                  );
                })}
              </div>

              <div className="pj-sec">
                <h2 className="pj-h2">Outputs available to other calculations</h2>
                {r && r.outputs.length > 0 ? (
                  <table className="tbl">
                    <thead><tr><th>Output</th><th className="r">Value</th><th>Unit</th><th>Used by</th></tr></thead>
                    <tbody>{r.outputs.map((o) => <tr key={o.key}><td>{o.label}</td><td className="m r">{o.value.toFixed(2)}</td><td className="m muted">{o.unit}</td><td className="muted">{usedBy(cur.id, o.key).join(", ") || "—"}</td></tr>)}</tbody>
                  </table>
                ) : <p className="muted small">No outputs: the calculation has not run successfully.</p>}
              </div>
            </section>
          )}
        </main>

        <aside className="pane pj-rail" aria-label="Project status">
          <div className="pane-h">Project<span className="meta">{res?.ok ? (res.overall_pass ? "all pass" : "attention") : "—"}</span></div>
          <div className="pane-b">
            <section className="rail-sec">
              <h3 className="rail-h">Status</h3>
              {res?.ok ? (<><StatusBadge status={res.overall_pass ? "ok" : "fail"}>{res.overall_pass ? "All calculations pass" : "Attention required"}</StatusBadge><div className="pj-order">{res.order.map((i) => <span key={i}>{nameOf(i)}</span>)}</div></>)
                : <p className="muted small">{nodes.length ? "Resolve the project errors to see its status." : "Add calculations to begin."}</p>}
            </section>
            <section className="rail-sec">
              <h3 className="rail-h">Change impact</h3>
              {!baseline && <p className="muted small">Save a revision, then edit inputs to see what changes. Saved results are never overwritten.</p>}
              {baseline && imp && (<>
                <dl className="kv">
                  <div><dt>Compared with</dt><dd>{baseline.name}</dd></div>
                  <div><dt>Edited</dt><dd>{imp.edited.length}</dd></div>
                  <div><dt>Affected downstream</dt><dd>{imp.affected.length}</dd></div>
                  <div><dt>Newly failing</dt><dd className={imp.newly_failing.length ? "t-fail" : ""}>{imp.newly_failing.length}</dd></div>
                  <div><dt>Newly passing</dt><dd>{imp.newly_passing.length}</dd></div>
                </dl>
                {imp.review_required && <Notice tone="warn">Results changed since {baseline.name}. Review before the next issue.</Notice>}
                {imp.newly_failing.length > 0 && <Notice tone="error">Now failing: {imp.newly_failing.map((i) => nameOf(i)).join(", ")}</Notice>}
                {imp.diffs.length > 0 && <table className="tbl"><thead><tr><th>Output</th><th className="r">Was → now</th></tr></thead><tbody>{imp.diffs.map((df, i) => <tr key={i}><td>{nameOf(df.node)} · {df.output}</td><td className="m r">{df.old.toFixed(1)} → {df.new.toFixed(1)}</td></tr>)}</tbody></table>}
                {(imp.added.length > 0 || imp.removed.length > 0) && <p className="muted small">{imp.added.length > 0 && <>Added: {imp.added.map((i) => nameOf(i)).join(", ")}. </>}{imp.removed.length > 0 && <>Removed: {imp.removed.join(", ")}.</>}</p>}
                <Button disabled={exBusy} onClick={explainChanges}>{exBusy ? "Thinking…" : "Explain these changes (AI)"}</Button>
                {explain && <Notice tone="info"><span style={{ whiteSpace: "pre-wrap" }}>{explain}</span><br /><span className="faint small">AI-generated summary of the change impact. It does not change any result.</span></Notice>}
              </>)}
            </section>
            <section className="rail-sec">
              <h3 className="rail-h">Revisions <span className="mono faint">{revs.length}</span></h3>
              {revs.length === 0 ? <p className="muted small">None saved.</p> : <ul className="pj-revs">{revs.map((rv) => <li key={rv.id} className={rv.id === base ? "on" : undefined}><span>{rv.name}</span><span className="mono faint">{rv.savedAt}</span></li>)}</ul>}
            </section>
            <section className="rail-sec"><p className="muted small">{cloud ? "Changes save to the cloud automatically." : "Working locally in this browser. Open a project from the Account page to save it to the cloud with approvals."} Links pass design values only: combine load effects with the Load combinations module first.</p></section>
          </div>
        </aside>
      </div>
    </div>
  );
}
