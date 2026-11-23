"use client";
import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { loadEngine } from "../../lib/engine";
import { supabase } from "../../lib/supabase";
import "../workspace/workspace.css";
import "./project.css";

const KEY = "strucal.project.v1";
const PS = "EN1992-1-1:2004/RECOMMENDED";
type Node = { id: string; name: string; module: string; mode: string; param_set: string; values: any; links: { field: string; from: string; output: string; factor: number }[] };

export default function ProjectPage() {
  const [eng, setEng] = useState<any>(null);
  const [defs, setDefs] = useState<any[]>([]);
  const [sets, setSets] = useState<any[]>([]);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [revs, setRevs] = useState<any[]>([]);
  const [base, setBase] = useState<string>("");
  const [add, setAdd] = useState("");
  const [err, setErr] = useState("");
  const [loaded, setLoaded] = useState(false);
  const [cloud, setCloud] = useState("");
  const [cloudName, setCloudName] = useState("");
  const [cloudReady, setCloudReady] = useState(false);
  const [ver, setVer] = useState(1);
  const [sync, setSync] = useState("");
  const [explain, setExplain] = useState(""); const [exBusy, setExBusy] = useState(false);

  useEffect(() => {
    loadEngine().then((m) => { setEng(m); const d = JSON.parse(m.modules()); setDefs(d); setSets(JSON.parse(m.param_sets())); setAdd(d[0]?.id ?? ""); }).catch((e) => setErr(String(e)));
    const pid = new URLSearchParams(window.location.search).get("p") ?? "";
    if (pid) setCloud(pid);
    else { try { const s = localStorage.getItem(KEY); if (s) { const o = JSON.parse(s); setNodes(o.nodes ?? []); setRevs(o.revs ?? []); setBase(o.base ?? ""); } } catch { /* empty storage */ } }
    setLoaded(true);
  }, []);
  const loadCloud = async (pid: string, keepBase = false) => {
    const { data: u } = await supabase.auth.getUser();
    if (!u.user) { setSync("Sign in on the Account page to use cloud projects."); return; }
    const { data: pr, error: pe } = await supabase.from("projects").select("name").eq("id", pid).single();
    if (pe || !pr) { setSync("Project not found or no access."); return; }
    setCloudName(pr.name);
    const { data: d } = await supabase.from("project_drafts").select("nodes, version").eq("project_id", pid).single();
    if (d) { setNodes(d.nodes ?? []); setVer(d.version); }
    const { data: rv } = await supabase.from("revisions").select("id, number, name, nodes, results, status, created_at").eq("project_id", pid).order("number");
    const mapped = (rv ?? []).map((r: any) => ({ id: r.id, name: `${r.name} (${r.status.replace("_", " ")})`, savedAt: new Date(r.created_at).toLocaleString(), project: r.nodes, results: r.results, status: r.status }));
    setRevs(mapped); if (!keepBase) setBase(mapped.length ? mapped[mapped.length - 1].id : "");
    setCloudReady(true); setSync("Saved to cloud");
  };
  useEffect(() => { if (cloud) loadCloud(cloud); /* eslint-disable-next-line */ }, [cloud]);
  useEffect(() => {
    if (!loaded) return;
    if (!cloud) { try { localStorage.setItem(KEY, JSON.stringify({ nodes, revs, base })); } catch { /* storage unavailable */ } return; }
    if (!cloudReady) return;
    setSync("Saving...");
    const t = setTimeout(async () => {
      const { data: u } = await supabase.auth.getUser();
      const { data, error } = await supabase.from("project_drafts").update({ nodes, version: ver + 1, updated_by: u.user?.id, updated_at: new Date().toISOString() }).eq("project_id", cloud).eq("version", ver).select("version");
      if (error) setSync("Not saved: " + error.message);
      else if (!data || data.length === 0) setSync("Conflict: someone else changed this project. Reload the page to get their changes before editing.");
      else { setVer(data[0].version); setSync("Saved to cloud"); }
    }, 900);
    return () => clearTimeout(t);
    // eslint-disable-next-line
  }, [nodes, cloud, cloudReady, loaded]);

  const baseline = revs.find((r) => r.id === base) ?? null;
  const res = useMemo(() => {
    if (!eng || nodes.length === 0) return null;
    return JSON.parse(eng.analyse_project(JSON.stringify({ nodes }), baseline ? JSON.stringify(baseline.results) : "null"));
  }, [eng, nodes, baseline]);

  const defOf = (id: string) => defs.find((d) => d.id === id);
  const nameOf = (id: string) => nodes.find((n) => n.id === id)?.name || id;
  const addNode = () => {
    const d = defOf(add); if (!d) return;
    const values: any = {}; for (const f of d.fields) values[f.key] = f.default;
    let n = 1; while (nodes.some((x) => x.id === `${d.id}${n}`)) n++;
    setNodes([...nodes, { id: `${d.id}${n}`, name: `${d.name} ${n}`, module: d.id, mode: "design", param_set: PS, values, links: [] }]);
  };
  const upd = (id: string, f: (n: Node) => Node) => setNodes(nodes.map((n) => (n.id === id ? f(n) : n)));
  const del = (id: string) => setNodes(nodes.filter((n) => n.id !== id).map((n) => ({ ...n, links: n.links.filter((l) => l.from !== id) })));
  const saveRev = async () => {
    if (!res?.ok) return;
    if (cloud) {
      const { error } = await supabase.rpc("save_revision", { p_project: cloud, p_name: `Rev ${revs.length + 1}`, p_nodes: nodes, p_results: res.nodes, p_engine: res.engine_version });
      if (error) setSync("Revision not saved: " + error.message); else { await loadCloud(cloud, true); setSync("Revision saved to cloud"); }
      return;
    }
    const id = `rev${revs.length + 1}`;
    setRevs([...revs, { id, name: `Rev ${revs.length + 1}`, savedAt: new Date().toLocaleString(), project: JSON.parse(JSON.stringify(nodes)), results: res.nodes }]);
    setBase(id);
  };
  const restore = () => { if (baseline && confirm(`Replace the current inputs with ${baseline.name}? Saved revisions are kept.`)) setNodes(JSON.parse(JSON.stringify(baseline.project))); };
  const exportJson = () => { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([JSON.stringify({ nodes, revisions: revs }, null, 2)], { type: "application/json" })); a.download = "strucal-project.json"; a.click(); };
  const explainChanges = async () => {
    setExBusy(true); setExplain("");
    try {
      const names = Object.fromEntries(nodes.map((n) => [n.id, `${n.name} (${n.module})`]));
      const { data, error } = await supabase.functions.invoke("assistant", { body: { task: "compare", question: "", context: { baseline: baseline?.name, impact: res?.impact, calculations: names, results: res?.nodes?.map((n: any) => ({ id: n.id, name: n.name, status: n.status, pass: n.overall_pass, max_util: n.max_util, outputs: n.outputs })) } } });
      if (error) { let m = error.message; try { const j = await (error as any).context?.json?.(); if (j?.error) m = j.error; } catch { /* keep */ } throw new Error(m); }
      if (data?.error) throw new Error(data.error);
      setExplain(data.answer);
    } catch (e: any) { setExplain("Assistant: " + (e?.message ?? String(e))); }
    setExBusy(false);
  };
  const rOf = (id: string) => res?.nodes?.find((n: any) => n.id === id);
  const imp = res?.impact;

  if (!eng) return <div className="pj"><div className="wk-title"><Link href="/" className="wk-logo">S</Link><div className="wk-wt">StruCal — Project</div></div><div className="pj-empty">{err || "Loading calculation engine..."}</div></div>;
  return (
    <div className="pj">
      <div className="wk-title"><Link href="/" className="wk-logo" title="StruCal - home">S</Link><div className="wk-wt">StruCal — Project dependencies{cloud ? ` — ${cloudName || "cloud project"}` : ""} — {nodes.length} calculation(s){cloud && sync ? ` — ${sync}` : ""}</div></div>
      <div className="wk-tabs"><span className="file">PROJECT</span><Link href="/workspace" style={{ padding: "0 14px", lineHeight: "24px" }}>Back to workspace</Link><Link href="/account" style={{ padding: "0 14px", lineHeight: "24px" }}>Account and teams</Link></div>
      <div className="wk-ribbon" style={{ height: 62, alignItems: "center", gap: 10, padding: "0 10px" }}>
        <select className="pj-btn" value={add} onChange={(e) => setAdd(e.target.value)} style={{ height: 26 }}>{defs.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}</select>
        <button className="pj-btn pri" onClick={addNode}>+ Add calculation</button>
        <button className="pj-btn" onClick={saveRev} disabled={!res?.ok}>{cloud ? "Save revision to cloud" : "Save revision"}</button>
        <button className="pj-btn" onClick={exportJson} disabled={nodes.length === 0}>Export project (.json)</button>
        <span style={{ color: "#52606d" }}>Compare against:</span>
        <select className="pj-btn" value={base} onChange={(e) => setBase(e.target.value)} style={{ height: 26 }}><option value="">(no baseline)</option>{revs.map((r) => <option key={r.id} value={r.id}>{r.name} · {r.savedAt}</option>)}</select>
        <button className="pj-btn" onClick={restore} disabled={!baseline}>Restore baseline inputs</button>
      </div>
      <div className="pj-body">
        <div className="pj-main">
          {nodes.length === 0 && <div className="pj-card pj-empty">No calculations yet. Add calculations and link one result to the next input, for example a wall&apos;s ULS load to a strip footing&apos;s load.</div>}
          {res && !res.ok && <div className="pj-err">{res.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}</div>}
          {nodes.map((n) => {
            const d = defOf(n.module); const r = rOf(n.id); if (!d) return null;
            const linked = (k: string) => n.links.find((l) => l.field === k);
            const options = nodes.filter((o) => o.id !== n.id).flatMap((o) => (rOf(o.id)?.outputs ?? []).map((out: any) => ({ v: `${o.id}|${out.key}`, label: `${o.name} → ${out.label} (${out.unit})` })));
            const badge = !r ? null : r.status === "blocked" ? <span className="pj-b blk">BLOCKED</span> : r.status === "error" ? <span className="pj-b fail">INPUT ERROR</span> : !r.has_verdict ? <span className="pj-b res">RESULTS</span> : r.overall_pass ? <span className="pj-b pass">PASS {(r.max_util * 100).toFixed(0)}%</span> : <span className="pj-b fail">FAIL {(r.max_util * 100).toFixed(0)}%</span>;
            const stale = imp && (imp.edited.includes(n.id) || imp.affected.includes(n.id));
            return (
              <div className="pj-card" key={n.id}>
                <div className="pj-ch">
                  <input className="nm" value={n.name} onChange={(e) => upd(n.id, (x) => ({ ...x, name: e.target.value }))} />
                  <span className="pj-tag">{d.name}</span>{badge}
                  {stale && <span className="pj-b blk" title="Changed since the baseline revision">{imp.edited.includes(n.id) ? "EDITED" : "AFFECTED"}</span>}
                  <span style={{ flex: 1 }} />
                  {d.has_check_mode && <select value={n.mode} onChange={(e) => upd(n.id, (x) => ({ ...x, mode: e.target.value }))}><option value="design">Design</option><option value="check">Check</option></select>}
                  <select value={n.param_set} onChange={(e) => upd(n.id, (x) => ({ ...x, param_set: e.target.value }))} style={{ maxWidth: 220 }}>{sets.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}</select>
                  <button className="pj-btn" onClick={() => del(n.id)}>Remove</button>
                </div>
                <div className="pj-grid">
                  {d.fields.filter((f: any) => !f.check_only || n.mode === "check").map((f: any) => {
                    const l = linked(f.key); const rv = r?.resolved?.find((x: any) => x.field === f.key);
                    return (<div className="pj-f" key={f.key}><label>{f.label} ({f.unit})</label>
                      <div className="row">
                        {f.options && !l ? (<select value={n.values[f.key]} onChange={(e) => upd(n.id, (x) => ({ ...x, values: { ...x.values, [f.key]: Number(e.target.value) } }))} style={{ flex: 1, maxWidth: "none" }}>{f.options.map(([v, lab]: any) => <option key={v} value={v}>{lab}</option>)}</select>) : (<input type="number" className={l ? "lk" : ""} readOnly={!!l} value={l ? (rv ? Number(rv.value.toFixed(4)) : "") : n.values[f.key]} onChange={(e) => upd(n.id, (x) => ({ ...x, values: { ...x.values, [f.key]: Number(e.target.value) } }))} />)}
                        <select value={l ? `${l.from}|${l.output}` : ""} title="Link this input to a result of another calculation" onChange={(e) => upd(n.id, (x) => ({ ...x, links: [...x.links.filter((k) => k.field !== f.key), ...(e.target.value ? [{ field: f.key, from: e.target.value.split("|")[0], output: e.target.value.split("|")[1], factor: l?.factor ?? 1 }] : [])] }))}>
                          <option value="">Manual</option>{options.map((o) => <option key={o.v} value={o.v}>{o.label}</option>)}
                        </select>
                        {l && <input className="fac" type="number" step="0.1" title="Multiplier applied to the linked value" value={l.factor} onChange={(e) => upd(n.id, (x) => ({ ...x, links: x.links.map((k) => (k.field === f.key ? { ...k, factor: Number(e.target.value) } : k)) }))} />}
                      </div></div>);
                  })}
                </div>
                {r?.errors?.length > 0 && <div className="pj-err">{r.errors.map((e: string, i: number) => <div key={i}>{e}</div>)}</div>}
                {r?.outputs?.length > 0 && <div className="pj-out">{r.outputs.map((o: any) => <span key={o.key}>{o.label}: <b>{o.value.toFixed(2)}</b> {o.unit}</span>)}</div>}
              </div>
            );
          })}
        </div>
        <aside className="pj-side pj-imp">
          <h3>Project status</h3>
          {res?.ok ? <div><span className={`pj-b ${res.overall_pass ? "pass" : "fail"}`}>{res.overall_pass ? "ALL CHECKS PASS" : "ATTENTION REQUIRED"}</span><div style={{ marginTop: 6, color: "#52606d" }}>Calculation order: {res.order.map((i: string) => nameOf(i)).join(" → ")}</div></div> : <div style={{ color: "#52606d" }}>Add calculations to begin.</div>}
          <h3>Change impact</h3>
          {!baseline && <div style={{ color: "#52606d" }}>Save a revision, then edit inputs to see what changes. Historical results are never overwritten.</div>}
          {baseline && imp && (<>
            <div className="row"><span>Compared with</span><b>{baseline.name}</b></div>
            <div className="row"><span>Edited</span><b>{imp.edited.length}</b></div>
            <div className="row"><span>Affected downstream</span><b>{imp.affected.length}</b></div>
            <div className="row"><span>Newly failing</span><b style={{ color: imp.newly_failing.length ? "#b42318" : undefined }}>{imp.newly_failing.length}</b></div>
            <div className="row"><span>Newly passing</span><b>{imp.newly_passing.length}</b></div>
            {imp.review_required && <div className="pj-warn">Results have changed since {baseline.name}. They must be reviewed before the next issue.</div>}
            {imp.newly_failing.length > 0 && <div className="pj-err">Now failing: {imp.newly_failing.map((i: string) => nameOf(i)).join(", ")}</div>}
            {imp.diffs.length > 0 && (<><h3>Output changes</h3>{imp.diffs.map((d: any, i: number) => <div className="row" key={i}><span>{nameOf(d.node)} · {d.output}</span><b>{d.old.toFixed(1)} → {d.new.toFixed(1)}</b></div>)}</>)}
            <div style={{ margin: "8px 0" }}><button className="pj-btn" disabled={exBusy} onClick={explainChanges}>{exBusy ? "Thinking..." : "Explain these changes (AI)"}</button></div>
            {explain && <div className="pj-warn" style={{ whiteSpace: "pre-wrap" }}>{explain}<div style={{ fontSize: 11, color: "#7b8794", marginTop: 4 }}>AI-generated summary of the change impact above. It does not change any result.</div></div>}
            {imp.added.length > 0 && <div style={{ marginTop: 6 }}>Added: {imp.added.map((i: string) => nameOf(i)).join(", ")}</div>}
            {imp.removed.length > 0 && <div style={{ marginTop: 6 }}>Removed: {imp.removed.join(", ")}</div>}
          </>)}
          <h3>Revisions</h3>
          {revs.length === 0 ? <div style={{ color: "#52606d" }}>None saved.</div> : <ul className="pj-list">{revs.map((r) => <li key={r.id}>{r.name} · {r.savedAt}</li>)}</ul>}
          {!cloud && <div className="pj-warn" style={{ marginTop: 12 }}>Working locally in this browser. Sign in on the Account page and open a project from there to save it to the cloud with approvals.</div>}
          {cloud && sync.startsWith("Conflict") && <div className="pj-err">{sync}</div>}
          <div className="pj-warn" style={{ marginTop: 12 }}>Links pass design values only. Combine load effects with the Load combinations module first, and confirm each linked value is the correct design action for the next member.</div>
        </aside>
      </div>
    </div>
  );
}
