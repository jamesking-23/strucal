"use client";
import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import AppShell from "../shell/AppShell";
import type { Command } from "../shell/CommandPalette";
import { DEFAULT_PARAM_SET } from "../../lib/engine";
import { useEngine } from "../../lib/useEngine";
import { useSession } from "../../lib/session";
import { supabase } from "../../lib/supabase";
import { askAssistant } from "../../lib/assistant";
import { DISCIPLINES, disciplineOf, parseNumber, standardOf, visibleFields } from "../../lib/form";
import { readJson, writeJson } from "../../lib/storage";
import { downloadText } from "../../lib/download";
import { REVIEW_THRESHOLD, fmtUtil } from "../../lib/status";
import type { Link as DepLink, ModDef, NodeResult, ProjectNode, Revision } from "../../lib/types";
import { Button, EmptyState, EngineBoot, Notice, SectionHeader, Segmented, Spinner, StatusBadge, StatusDot } from "../ui/primitives";
import { IconExport, IconPlus } from "../ui/icons";
import "./project.css";

const KEY = "strucal.project.v1";
type Cloud = "off" | "loading" | "ready" | "signed-out" | "no-access" | "error" | "conflict";
type NodeStatus = "pass" | "warn" | "fail" | "results" | "none";

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
  return { id: n.id, name: typeof n.name === "string" ? n.name : n.id, module: n.module, mode: n.mode === "check" && def?.has_check_mode ? "check" : "design", param_set: typeof n.param_set === "string" && paramSets.includes(n.param_set) ? n.param_set : DEFAULT_PARAM_SET, values, links };
}

/** Numeric cell that keeps the typed text and commits only finite numbers. */
function NumCell({ id, value, onCommit, readOnly, label, disabled }: { id?: string; value: number | ""; onCommit?: (v: number) => void; readOnly?: boolean; label: string; disabled?: boolean }) {
  const [draft, setDraft] = useState(String(value));
  useEffect(() => { if (parseNumber(draft) !== value) setDraft(String(value)); /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, [value]);
  const bad = !readOnly && parseNumber(draft) === null;
  return <input id={id} className={`pj-num${bad ? " bad" : ""}`} type="text" inputMode="decimal" aria-label={label} aria-invalid={bad} readOnly={readOnly} disabled={disabled}
    value={readOnly ? String(value) : draft} onChange={(e) => { setDraft(e.target.value); const v = parseNumber(e.target.value); if (v !== null) onCommit?.(v); }} />;
}

function ProjectWorkbench() {
  const router = useRouter();
  const params = useSearchParams();
  const auth = useSession();
  const { engine: eng, error: loadErr, retry } = useEngine();
  const [nodes, setNodes] = useState<ProjectNode[]>([]);
  const [revs, setRevs] = useState<Revision[]>([]);
  const [base, setBase] = useState("");
  const [add, setAdd] = useState("");
  const [sel, setSel] = useState("");
  const [loaded, setLoaded] = useState(false);
  const [notice, setNotice] = useState("");
  const [cloudId, setCloudId] = useState("");
  const [cloudName, setCloudName] = useState("");
  const [cloud, setCloud] = useState<Cloud>("off");
  const [cloudMsg, setCloudMsg] = useState("");
  const [sync, setSync] = useState("");
  const [ver, setVer] = useState(1);
  const [busy, setBusy] = useState(false);
  const [explain, setExplain] = useState(""); const [exBusy, setExBusy] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const synced = useRef("");

  const defs = useMemo(() => eng?.defs ?? [], [eng]);
  const psIds = useMemo(() => (eng?.paramSets ?? []).map((p) => p.id), [eng]);
  const norm = useCallback((list: unknown): ProjectNode[] => (Array.isArray(list) ? list.map((x) => normalizeNode(x, defs, psIds)).filter((x): x is ProjectNode => !!x) : []), [defs, psIds]);
  const editable = cloud === "off" || cloud === "ready";

  const loadLocal = useCallback(() => {
    const o = readJson<{ nodes?: unknown; revs?: Revision[]; base?: string }>(KEY);
    const ns = norm(o?.nodes);
    setNodes(ns);
    const rv = Array.isArray(o?.revs) ? o!.revs.map((r) => ({ ...r, project: norm(r.project) })) : [];
    setRevs(rv); setBase(rv.some((r) => r.id === o?.base) ? (o!.base as string) : "");
    setSel(ns[0]?.id ?? "");
  }, [norm]);

  const loadCloud = useCallback(async (pid: string, keepBase = false) => {
    setCloud("loading"); setCloudMsg(""); setSync("Loading…");
    const { data: u } = await supabase.auth.getUser();
    if (!u.user) { setCloud("signed-out"); setSync(""); return; }
    const { data: pr, error: pe } = await supabase.from("projects").select("name").eq("id", pid).maybeSingle();
    if (pe) { setCloud("error"); setCloudMsg(pe.message); setSync(""); return; }
    if (!pr) { setCloud("no-access"); setSync(""); return; }
    setCloudName(pr.name);
    const { data: d, error: de } = await supabase.from("project_drafts").select("nodes, version").eq("project_id", pid).maybeSingle();
    if (de || !d) { setCloud("error"); setCloudMsg(de?.message ?? "The project draft is missing."); setSync(""); return; }
    const list = norm(d.nodes);
    synced.current = JSON.stringify(list);
    setNodes(list); setVer(d.version); setSel((s) => (list.some((x) => x.id === s) ? s : list[0]?.id ?? ""));
    const { data: rv, error: re } = await supabase.from("revisions").select("id, number, name, nodes, results, status, created_at").eq("project_id", pid).order("number");
    if (re) setNotice(`Revisions could not be loaded: ${re.message}`);
    const mapped: Revision[] = (rv ?? []).map((r: { id: string; name: string; status: string; created_at: string; nodes: unknown; results: NodeResult[] }) => ({ id: r.id, name: `${r.name} · ${r.status.replace(/_/g, " ")}`, savedAt: new Date(r.created_at).toLocaleString(), project: norm(r.nodes), results: r.results ?? [], status: r.status }));
    setRevs(mapped); if (!keepBase) setBase(mapped.length ? mapped[mapped.length - 1].id : "");
    setCloud("ready"); setSync("Saved to cloud");
  }, [norm]);

  // Initial load: ?p=<id> opens a cloud project; otherwise the local browser project.
  useEffect(() => {
    if (!eng) return;
    setAdd(eng.defs.find((d) => d.id === "wall")?.id ?? eng.defs[0]?.id ?? "");
    const pid = params.get("p") ?? "";
    setCloudId(pid);
    if (pid) loadCloud(pid); else { setCloud("off"); loadLocal(); }
    setLoaded(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eng, params]);

  // Signing in (here or in another tab) resumes a cloud project that was waiting for a session.
  useEffect(() => { if (cloudId && cloud === "signed-out" && auth.status === "signed-in") loadCloud(cloudId); }, [auth.status, cloud, cloudId, loadCloud]);

  // Local projects save on every change.
  useEffect(() => {
    if (!loaded || cloudId) return;
    if (!writeJson(KEY, { nodes, revs, base })) setNotice("This browser is not storing the project (private mode or full storage). Export it to keep a copy.");
  }, [nodes, revs, base, cloudId, loaded]);

  // Cloud drafts save debounced with an optimistic version check — only while the project is loaded and writable.
  useEffect(() => {
    if (!cloudId || cloud !== "ready") return;
    const body = JSON.stringify(nodes);
    if (body === synced.current) return;
    setSync("Saving…");
    const t = setTimeout(async () => {
      const { data: u } = await supabase.auth.getUser();
      if (!u.user) { setCloud("signed-out"); setSync("Not saved: signed out"); return; }
      const { data, error } = await supabase.from("project_drafts").update({ nodes, version: ver + 1, updated_by: u.user.id, updated_at: new Date().toISOString() }).eq("project_id", cloudId).eq("version", ver).select("version");
      if (error) setSync(`Not saved: ${error.message}`);
      else if (!data || data.length === 0) { setCloud("conflict"); setSync("Conflict"); }
      else { synced.current = body; setVer(data[0].version); setSync("Saved to cloud"); }
    }, 800);
    return () => clearTimeout(t);
  }, [nodes, cloudId, cloud, ver]);

  const baseline = revs.find((r) => r.id === base) ?? null;
  const res = useMemo(() => (eng && nodes.length ? eng.analyse(nodes, baseline ? baseline.results : null) : null), [eng, nodes, baseline]);
  const defOf = (id: string) => defs.find((d) => d.id === id);
  const nameOf = (id: string) => nodes.find((n) => n.id === id)?.name || id;
  const rOf = (id: string) => res?.nodes.find((n) => n.id === id);
  const imp = res?.impact;

  const addNode = useCallback((moduleId?: string) => {
    const d = defs.find((x) => x.id === (moduleId ?? add)); if (!d || !editable) return;
    const values: Record<string, number> = {}; for (const fl of d.fields) values[fl.key] = fl.default;
    let k = 1; while (nodes.some((x) => x.id === `${d.id}${k}`)) k++;
    setNodes((all) => [...all, { id: `${d.id}${k}`, name: `${d.name} ${k}`, module: d.id, mode: "design", param_set: DEFAULT_PARAM_SET, values, links: [] }]);
    setSel(`${d.id}${k}`);
  }, [defs, add, nodes, editable]);
  const upd = (id: string, fn: (n: ProjectNode) => ProjectNode) => { if (editable) setNodes((all) => all.map((n) => (n.id === id ? fn(n) : n))); };
  const del = (id: string) => {
    if (!editable) return;
    const dependants = nodes.filter((n) => n.links.some((l) => l.from === id)).map((n) => n.name);
    if (!confirm(`Remove ${nameOf(id)}?${dependants.length ? `\nInputs linked to it in ${dependants.join(", ")} become manual.` : ""}`)) return;
    setNodes(nodes.filter((n) => n.id !== id).map((n) => ({ ...n, links: n.links.filter((l) => l.from !== id) })));
    if (sel === id) setSel("");
  };
  const saveRev = async () => {
    if (!res?.ok || !editable) return;
    setBusy(true);
    if (cloudId) {
      const { error } = await supabase.rpc("save_revision", { p_project: cloudId, p_name: `Rev ${revs.length + 1}`, p_nodes: nodes, p_results: res.nodes, p_engine: res.engine_version });
      if (error) setNotice(`Revision not saved: ${error.message}`); else { await loadCloud(cloudId, true); setNotice("Revision saved to the cloud."); }
    } else {
      let k = revs.length + 1; while (revs.some((r) => r.id === `rev${k}`)) k++;
      setRevs([...revs, { id: `rev${k}`, name: `Rev ${k}`, savedAt: new Date().toLocaleString(), project: structuredClone(nodes), results: res.nodes }]);
      setBase(`rev${k}`);
    }
    setBusy(false);
  };
  const restore = () => { if (baseline && editable && confirm(`Replace the current inputs with ${baseline.name}? Saved revisions are kept.`)) setNodes(structuredClone(baseline.project)); };
  const exportJson = () => downloadText(`strucal-${(cloudName || "project").replace(/\W+/g, "-")}.json`, JSON.stringify({ nodes, revisions: revs }, null, 2), "application/json");
  const importJson = async (file: File) => {
    try {
      const o = JSON.parse(await file.text());
      const list = norm(o?.nodes);
      if (list.length === 0) throw new Error("the file has no calculations");
      if (nodes.length && !confirm(`Replace the ${nodes.length} current calculation(s) with ${list.length} from ${file.name}?`)) return;
      setNodes(list); setSel(list[0].id);
      if (!cloudId && Array.isArray(o.revisions)) { setRevs(o.revisions.map((r: Revision) => ({ ...r, project: norm(r.project) }))); setBase(""); }
      setNotice(`Imported ${list.length} calculation(s) from ${file.name}.`);
    } catch (e) { setNotice(`Could not import ${file.name}: ${e instanceof Error ? e.message : String(e)}`); }
  };
  const explainChanges = async () => {
    setExBusy(true); setExplain("");
    try {
      setExplain(await askAssistant("compare", "", { baseline: baseline?.name, impact: res?.impact, calculations: Object.fromEntries(nodes.map((n) => [n.id, `${n.name} (${n.module})`])), results: res?.nodes.map((n) => ({ id: n.id, name: n.name, status: n.status, pass: n.overall_pass, max_util: n.max_util, outputs: n.outputs })) }));
    } catch (e) { setExplain(`Assistant: ${e instanceof Error ? e.message : String(e)}`); }
    setExBusy(false);
  };
  const workLocally = () => router.push("/project");

  const commands = useMemo<Command[]>(() => defs.map((d) => ({ id: `add-${d.id}`, group: "Add calculation", title: `Add ${d.name}`, hint: disciplineOf(d.id), run: () => addNode(d.id) })), [defs, addNode]);

  if (!eng) return <AppShell section="project" crumbs={["Project"]}><EngineBoot error={loadErr} onRetry={retry} /></AppShell>;

  const statusOf = (r?: NodeResult): NodeStatus => !r ? "none" : r.status !== "calculated" ? "fail" : !r.has_verdict ? "results" : !r.overall_pass ? "fail" : r.max_util >= REVIEW_THRESHOLD ? "warn" : "pass";
  const statusText = (r?: NodeResult) => !r ? "—" : r.status === "blocked" ? "Blocked" : r.status === "error" ? "Input error" : !r.has_verdict ? "Results" : `η ${fmtUtil(r.max_util)}`;
  const ordered = res?.ok ? res.order.map((id) => nodes.find((n) => n.id === id)).filter((n): n is ProjectNode => !!n) : nodes;
  const cur = nodes.find((n) => n.id === sel) ?? ordered[0];
  const d = cur ? defOf(cur.module) : undefined;
  const r = cur ? rOf(cur.id) : undefined;
  const mark = (id: string) => (imp ? (imp.edited.includes(id) ? "edited" : imp.affected.includes(id) ? "affected" : "") : "");
  const options = cur ? nodes.filter((o) => o.id !== cur.id).flatMap((o) => (rOf(o.id)?.outputs ?? []).map((out) => ({ v: `${o.id}|${out.key}`, label: `${o.name} → ${out.label} (${out.unit})` }))) : [];
  const usedBy = (nodeId: string, key: string) => nodes.filter((n) => n.links.some((l) => l.from === nodeId && l.output === key)).map((n) => n.name);

  const actions = (<>
    <label className="sr-only" htmlFor="pj-add">Module to add</label>
    <select id="pj-add" className="select pj-add" value={add} onChange={(e) => setAdd(e.target.value)} disabled={!editable}>{DISCIPLINES.map((g) => <optgroup key={g} label={g}>{defs.filter((x) => disciplineOf(x.id) === g).map((x) => <option key={x.id} value={x.id}>{x.name}</option>)}</optgroup>)}</select>
    <Button size="sm" variant="primary" onClick={() => addNode()} disabled={!editable}><IconPlus width={13} height={13} />Add</Button>
    <Button size="sm" onClick={saveRev} disabled={!res?.ok || busy || !editable}>{busy ? "Saving…" : "Save revision"}</Button>
    <Button size="sm" variant="ghost" icon onClick={exportJson} disabled={nodes.length === 0} aria-label="Export project (.json)" title="Export (.json)"><IconExport width={14} height={14} /></Button>
  </>);
  const status = (<>
    <span className="st"><StatusDot status={res?.ok ? (res.overall_pass ? "pass" : "fail") : "none"} />{res?.ok ? (res.overall_pass ? "All calculations pass" : "Attention required") : nodes.length ? "Project errors" : "Empty project"}</span>
    <span>{nodes.length} calculation{nodes.length === 1 ? "" : "s"}</span>
    <span className="grow" />
    <span>{cloudId ? `cloud · ${sync || cloud}` : "local · this browser"}</span>
    <span>engine v{eng.version}</span>
  </>);

  return (
    <AppShell section="project" crumbs={["Project", cloudId ? cloudName || "Cloud project" : "Local project"]} actions={actions} status={status} commands={commands}>
      <div className="pj">
        <aside className="pj-list" aria-label="Calculations">
          <SectionHeader title="Load path" meta={`${nodes.length} · dependency order`} />
          <div className="pj-list-b">
            {nodes.length === 0 ? <p className="pj-list-empty">No calculations.</p> : (
              <ol>
                {ordered.map((n, i) => {
                  const nr = rOf(n.id); const st = statusOf(nr); const m = mark(n.id);
                  return (
                    <li key={n.id}>
                      <button type="button" className={`pj-item${cur?.id === n.id ? " on" : ""}`} aria-current={cur?.id === n.id} onClick={() => setSel(n.id)}>
                        <span className="i mono">{String(i + 1).padStart(2, "0")}</span>
                        <span className="nm">{n.name}</span>
                        <span className={`st mono t-${st === "results" || st === "none" ? "" : st}`}>{statusText(nr)}</span>
                        <span className="sub mono">{defOf(n.module)?.name ?? n.module}{n.links.length > 0 ? ` · ← ${n.links.length}` : ""}{m ? ` · ${m}` : ""}</span>
                      </button>
                    </li>
                  );
                })}
              </ol>
            )}
          </div>
          <div className="pj-list-f">
            <Button size="sm" variant="ghost" onClick={() => fileRef.current?.click()} disabled={!editable}>Import .json</Button>
            <input ref={fileRef} type="file" accept="application/json,.json" hidden onChange={(e) => { const fl = e.target.files?.[0]; if (fl) importJson(fl); e.target.value = ""; }} />
          </div>
        </aside>

        <main className="pj-main" aria-label="Calculation editor">
          {cloudId && cloud !== "ready" && cloud !== "off" && (
            <div className="pj-gate">
              {cloud === "loading" && <div className="row"><Spinner /><span className="muted">Loading cloud project…</span></div>}
              {cloud === "signed-out" && <Notice tone="warn" title="Sign in to open this cloud project." action={<><Link className="btn sm primary" href="/account">Sign in</Link><Button size="sm" onClick={workLocally}>Work locally</Button></>}>Editing is disabled until you sign in, so nothing is lost. The project opens automatically once you are signed in.</Notice>}
              {cloud === "no-access" && <Notice tone="error" title="Project not found, or you are not a member of its organisation." action={<Button size="sm" onClick={workLocally}>Work locally</Button>}>Ask an organisation administrator to add you, or open a project from the Account page.</Notice>}
              {cloud === "error" && <Notice tone="error" title="The cloud project could not be loaded." action={<><Button size="sm" onClick={() => loadCloud(cloudId)}>Retry</Button><Button size="sm" onClick={workLocally}>Work locally</Button></>}>{cloudMsg}</Notice>}
              {cloud === "conflict" && <Notice tone="error" title="Someone else saved this project after you opened it." action={<Button size="sm" variant="primary" onClick={() => { if (confirm("Load the latest version? Your unsaved edits since the last save will be discarded.")) loadCloud(cloudId, true); }}>Load latest version</Button>}>Your recent edits were not saved. Load the latest version, then reapply them.</Notice>}
            </div>
          )}
          {notice && <Notice tone="info" onDismiss={() => setNotice("")}>{notice}</Notice>}
          {res && !res.ok && <Notice tone="error" title="The project cannot be analysed."><ul>{res.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></Notice>}

          {!cur && editable && (
            <div className="pj-empty">
              <EmptyState glyph="→" title="Build a load path">Each calculation can take inputs from another calculation&apos;s outputs. Changes propagate in dependency order and are compared with a saved revision.</EmptyState>
              <ol className="pj-steps"><li>Add an <b>RC wall</b>.</li><li>Add the <b>strip footing</b> it bears on.</li><li>Set the footing load&apos;s source to the wall&apos;s ULS load.</li><li>Save a revision — later edits show what changed downstream.</li></ol>
              <Button variant="primary" onClick={() => addNode()}>Add {defOf(add)?.name ?? "calculation"}</Button>
            </div>
          )}

          {cur && !d && (
            <section className="pj-ed">
              <header className="pj-ed-h"><h1>{cur.name}</h1><StatusBadge status="fail">unknown module</StatusBadge><span className="grow" /><Button size="sm" variant="danger" onClick={() => del(cur.id)} disabled={!editable}>Remove</Button></header>
              <Notice tone="error">Module &quot;{cur.module}&quot; is not available in engine v{eng.version}. Remove this calculation or open the project in a matching version.</Notice>
            </section>
          )}

          {cur && d && (
            <section className={`pj-ed${editable ? "" : " locked"}`} aria-disabled={!editable}>
              <header className="pj-ed-h">
                <input className="pj-name" value={cur.name} aria-label="Calculation name" onChange={(e) => upd(cur.id, (x) => ({ ...x, name: e.target.value }))} disabled={!editable} />
                <StatusBadge status={statusOf(r)}><span className="nt">{statusText(r)}</span></StatusBadge>
                {mark(cur.id) && <span className="badge warn">{mark(cur.id)}</span>}
                <span className="grow" />
                <Segmented label="Mode" value={cur.mode} onChange={(m) => upd(cur.id, (x) => ({ ...x, mode: m }))} options={[{ value: "design", label: "Design" }, { value: "check", label: "Check", disabled: !d.has_check_mode || !editable }]} />
                <Button size="sm" variant="danger" onClick={() => del(cur.id)} disabled={!editable}>Remove</Button>
              </header>
              <div className="pj-ed-meta">
                <span className="mono">{d.name} · {standardOf(d)}</span>
                <select className="select pj-ps" value={cur.param_set} aria-label="Parameter set" onChange={(e) => upd(cur.id, (x) => ({ ...x, param_set: e.target.value }))} disabled={!editable}>{eng.paramSets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
                <Link className="btn sm ghost" href={`/workspace?m=${d.id}`}>Open module in workbench →</Link>
              </div>
              {r && r.errors.length > 0 && <Notice tone="error" title={r.status === "blocked" ? "Blocked by an upstream calculation." : "Input error."}><ul>{r.errors.map((e, i) => <li key={i}>{e}</li>)}</ul></Notice>}

              <div className="pj-grid" role="table" aria-label="Inputs">
                <div className="pj-row pj-head" role="row"><span role="columnheader">Parameter</span><span role="columnheader" className="r">Value</span><span role="columnheader">Unit</span><span role="columnheader">Source</span><span role="columnheader" className="r">×</span></div>
                {visibleFields(d, cur.mode).map((fd) => {
                  const l = cur.links.find((k) => k.field === fd.key); const rv = r?.resolved.find((x) => x.field === fd.key);
                  const setVal = (v: number) => upd(cur.id, (x) => ({ ...x, values: { ...x.values, [fd.key]: v } }));
                  const changed = !l && Math.abs((cur.values[fd.key] ?? fd.default) - fd.default) > 1e-12;
                  return (
                    <div key={fd.key} className={`pj-row${l ? " linked" : changed ? " changed" : ""}`} role="row">
                      <label htmlFor={`pv-${fd.key}`}>{fd.label}</label>
                      {fd.options && !l
                        ? <select id={`pv-${fd.key}`} className="pj-num sel" value={cur.values[fd.key]} onChange={(e) => setVal(Number(e.target.value))} disabled={!editable}>{fd.options.map(([v, lab]) => <option key={v} value={v}>{lab}</option>)}</select>
                        : l ? <NumCell id={`pv-${fd.key}`} readOnly label={`${fd.label} (linked)`} value={rv ? Number(rv.value.toFixed(4)) : ""} /> : <NumCell id={`pv-${fd.key}`} label={fd.label} value={cur.values[fd.key]} onCommit={setVal} disabled={!editable} />}
                      <span className="u mono">{fd.unit === "-" ? "" : fd.unit}</span>
                      <select className={`pj-src${l ? " on" : ""}`} value={l ? `${l.from}|${l.output}` : ""} aria-label={`Source for ${fd.label}`} disabled={!editable}
                        onChange={(e) => upd(cur.id, (x) => ({ ...x, links: [...x.links.filter((k) => k.field !== fd.key), ...(e.target.value ? [{ field: fd.key, from: e.target.value.split("|")[0], output: e.target.value.split("|")[1], factor: l?.factor ?? 1 }] : [])] }))}>
                        <option value="">Manual</option>{options.map((o) => <option key={o.v} value={o.v}>{o.label}</option>)}
                        {l && !options.some((o) => o.v === `${l.from}|${l.output}`) && <option value={`${l.from}|${l.output}`}>{nameOf(l.from)} → {l.output} (unavailable)</option>}
                      </select>
                      {l ? <NumCell label={`Multiplier for ${fd.label}`} value={l.factor} onCommit={(v) => upd(cur.id, (x) => ({ ...x, links: x.links.map((k) => (k.field === fd.key ? { ...k, factor: v } : k)) }))} disabled={!editable} /> : <span />}
                    </div>
                  );
                })}
              </div>

              <div className="pj-out">
                <SectionHeader title="Outputs available to other calculations" meta={r?.outputs.length ? String(r.outputs.length) : "—"} />
                {r && r.outputs.length > 0 ? (
                  <table className="tbl"><thead><tr><th>Output</th><th className="r">Value</th><th>Unit</th><th>Used by</th></tr></thead>
                    <tbody>{r.outputs.map((o) => <tr key={o.key}><td>{o.label}</td><td className="m r">{o.value.toFixed(2)}</td><td className="m faint">{o.unit}</td><td className="muted">{usedBy(cur.id, o.key).join(", ") || "—"}</td></tr>)}</tbody></table>
                ) : <p className="muted small pj-pad">No outputs: the calculation has not run successfully.</p>}
              </div>
            </section>
          )}
        </main>

        <aside className="pj-side" aria-label="Project status and change impact">
          <SectionHeader title="Status" meta={res?.ok ? (res.overall_pass ? "all pass" : "attention") : "—"} />
          <div className="pj-pad">
            {res?.ok ? <div className="pj-order">{res.order.map((i) => <span key={i}>{nameOf(i)}</span>)}</div> : <p className="muted small">{nodes.length ? "Resolve the project errors to see its status." : "Add calculations to begin."}</p>}
          </div>
          <SectionHeader title="Change impact" meta={baseline ? baseline.name : "no baseline"} />
          <div className="pj-pad stack">
            <select className="select" value={base} onChange={(e) => setBase(e.target.value)} aria-label="Baseline revision"><option value="">No baseline</option>{revs.map((rv) => <option key={rv.id} value={rv.id}>Compare with {rv.name}</option>)}</select>
            {!baseline && <p className="muted small">Save a revision, then edit inputs to see what changes downstream. Saved results are never overwritten.</p>}
            {baseline && imp && (<>
              <dl className="kv">
                <div><dt>Edited</dt><dd>{imp.edited.length}</dd></div>
                <div><dt>Affected downstream</dt><dd>{imp.affected.length}</dd></div>
                <div><dt>Newly failing</dt><dd className={imp.newly_failing.length ? "t-fail" : ""}>{imp.newly_failing.length}</dd></div>
                <div><dt>Newly passing</dt><dd>{imp.newly_passing.length}</dd></div>
              </dl>
              {imp.review_required && <Notice tone="warn">Results changed since {baseline.name}. Review before the next issue.</Notice>}
              {imp.newly_failing.length > 0 && <Notice tone="error">Now failing: {imp.newly_failing.map((i) => nameOf(i)).join(", ")}</Notice>}
              {imp.diffs.length > 0 && <table className="tbl"><thead><tr><th>Output</th><th className="r">Was → now</th></tr></thead><tbody>{imp.diffs.map((df, i) => <tr key={i}><td>{nameOf(df.node)} · {df.output}</td><td className="m r">{df.old.toFixed(1)} → {df.new.toFixed(1)}</td></tr>)}</tbody></table>}
              <Button size="sm" onClick={restore} disabled={!editable}>Restore baseline inputs</Button>
              <Button size="sm" disabled={exBusy || auth.status !== "signed-in"} onClick={explainChanges} title={auth.status !== "signed-in" ? "Sign in to use the assistant" : undefined}>{exBusy ? "Thinking…" : "Explain these changes (AI)"}</Button>
              {auth.status !== "signed-in" && <p className="faint small">The assistant needs a signed-in account.</p>}
              {explain && <Notice tone="info"><span style={{ whiteSpace: "pre-wrap" }}>{explain}</span></Notice>}
            </>)}
          </div>
          <SectionHeader title="Revisions" meta={String(revs.length)} />
          <div className="pj-pad">
            {revs.length === 0 ? <p className="muted small">None saved.</p> : <ul className="pj-revs">{revs.map((rv) => <li key={rv.id} className={rv.id === base ? "on" : undefined}><span>{rv.name}</span><span className="mono faint">{rv.savedAt}</span></li>)}</ul>}
            <p className="faint small" style={{ marginTop: 12 }}>{cloudId ? "Changes save to the cloud automatically." : "Working locally in this browser. Open a project from Account to share it with approvals."} Links pass design values only: combine load effects with the Load combinations module first.</p>
          </div>
        </aside>
      </div>
    </AppShell>
  );
}

export default function ProjectPage() {
  return <Suspense fallback={<div className="boot"><span className="spinner" /></div>}><ProjectWorkbench /></Suspense>;
}
