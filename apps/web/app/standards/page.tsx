"use client";
import { useMemo, useState } from "react";
import TopBar from "../components/TopBar";
import { useEngine } from "../../lib/useEngine";
import type { Formula, ParamSet } from "../../lib/types";
import { Button, EmptyState, EngineBoot } from "../ui/primitives";
import "./standards.css";

const FACTORS: [keyof ParamSet, string, string][] = [
  ["gamma_c", "γ", "c · concrete"], ["gamma_s", "γ", "s · reinforcement"], ["alpha_cc", "α", "cc · long-term compression"], ["alpha_cw", "α", "cw · strut"],
  ["gamma_g", "γ", "G · permanent actions"], ["gamma_q", "γ", "Q · variable actions"], ["gamma_m0", "γ", "M0 · steel cross-section"], ["gamma_m1", "γ", "M1 · steel member stability"],
];

type State = "checked" | "pending" | "other";
const stateOf = (v: string): State => (/^CHECKED/.test(v) ? "checked" : /PENDING/.test(v) ? "pending" : "other");
const STATE_LABEL: Record<State, string> = { checked: "Checked against worked examples", pending: "Pending review", other: "Software function / method note" };

export default function Standards() {
  const { engine: eng, error, retry } = useEngine();
  const [q, setQ] = useState("");
  const [std, setStd] = useState<string>("");
  const [state, setState] = useState<State | "">("");

  const all = useMemo(() => eng?.formulas ?? [], [eng]);
  const standards = useMemo(() => [...new Set(all.map((f) => f.standard))].sort(), [all]);
  const rows = useMemo(() => {
    const t = q.trim().toLowerCase();
    return all
      .filter((f) => (!std || f.standard === std) && (!state || stateOf(f.verification) === state))
      .filter((f) => !t || [f.id, f.name, f.clause, f.verification].some((x) => x.toLowerCase().includes(t)))
      .sort((a, b) => a.standard.localeCompare(b.standard) || a.id.localeCompare(b.id));
  }, [all, q, std, state]);
  const count = (pred: (f: Formula) => boolean) => all.filter(pred).length;

  if (!eng) return <div className="app"><TopBar section="standards" /><EngineBoot error={error} onRetry={retry} /></div>;

  return (
    <div className="app">
      <TopBar section="standards" context={`Engine v${eng.version} · ${all.length} formulas · ${eng.paramSets.length} parameter sets`} />
      <div className="sd-grid">
        <aside className="pane sd-facets" aria-label="Filters">
          <div className="pane-h">Filter<span className="meta">{rows.length} / {all.length}</span></div>
          <div className="pane-b">
            <div className="sd-search"><label className="sr-only" htmlFor="sd-q">Search formulas</label><input id="sd-q" className="input" type="search" placeholder="ID, name or clause" value={q} onChange={(e) => setQ(e.target.value)} /></div>
            <div className="sd-facet">
              <h3 className="rail-h">Standard</h3>
              <button type="button" className={`sd-opt${std === "" ? " on" : ""}`} onClick={() => setStd("")}><span>All standards</span><span className="mono">{all.length}</span></button>
              {standards.map((s) => <button type="button" key={s} className={`sd-opt${std === s ? " on" : ""}`} onClick={() => setStd(std === s ? "" : s)}><span>{s}</span><span className="mono">{count((f) => f.standard === s)}</span></button>)}
            </div>
            <div className="sd-facet">
              <h3 className="rail-h">Verification state</h3>
              <button type="button" className={`sd-opt${state === "" ? " on" : ""}`} onClick={() => setState("")}><span>Any</span><span className="mono">{all.length}</span></button>
              {(["checked", "pending", "other"] as State[]).map((s) => <button type="button" key={s} className={`sd-opt${state === s ? " on" : ""}`} onClick={() => setState(state === s ? "" : s)}><span><i className={`dot ${s === "checked" ? "ok" : s === "pending" ? "warn" : ""}`} /> {STATE_LABEL[s]}</span><span className="mono">{count((f) => stateOf(f.verification) === s)}</span></button>)}
            </div>
          </div>
        </aside>

        <main className="sd-main">
          <header className="page-head">
            <span className="eyebrow">Standards</span>
            <h1>Formula registry and parameter sets</h1>
            <p>Every implemented formula with its source clause and verification state, and the partial factors each parameter set applies. States are shown exactly as recorded in the engine.</p>
          </header>

          <section className="sd-sec" aria-labelledby="ps-h">
            <h2 id="ps-h" className="sd-h2"><span className="mono">A</span>Parameter sets <em>partial factors compared</em></h2>
            <div className="tbl-wrap">
              <table className="tbl sd-ps">
                <thead>
                  <tr><th>Factor</th>{eng.paramSets.map((p) => <th key={p.id} className="r"><span className="nt">{p.label}</span></th>)}</tr>
                </thead>
                <tbody>
                  {FACTORS.map(([k, sym, desc]) => {
                    const vals = eng.paramSets.map((p) => p[k] as number);
                    const differs = vals.some((v) => v !== vals[0]);
                    return (
                      <tr key={k} className={differs ? "differs" : undefined}>
                        <td><i className="sym">{sym}</i><sub>{desc.split(" · ")[0]}</sub> <span className="muted">{desc.split(" · ")[1]}</span></td>
                        {vals.map((v, i) => <td key={i} className="m r">{v.toFixed(2)}</td>)}
                      </tr>
                    );
                  })}
                  <tr className="status-row"><td className="muted">Status</td>{eng.paramSets.map((p) => <td key={p.id} className="r"><span className={`badge wrap ${/NOT/.test(p.status) ? "warn" : "ok"}`}>{p.status}</span></td>)}</tr>
                  <tr><td className="muted">Source</td>{eng.paramSets.map((p) => <td key={p.id} className="r small muted">{p.source}</td>)}</tr>
                </tbody>
              </table>
            </div>
            <p className="sd-fine">Rows where the sets differ are marked. No set is a verified National Annex for Uganda; confirm the governing annex before issue for construction.</p>
          </section>

          <section className="sd-sec" aria-labelledby="fr-h">
            <h2 id="fr-h" className="sd-h2"><span className="mono">B</span>Formula registry <em>{rows.length} of {all.length}</em></h2>
            {rows.length === 0 ? (
              <EmptyState glyph="ƒ" title="No formulas match" action={<Button onClick={() => { setQ(""); setStd(""); setState(""); }}>Clear filters</Button>}>Try a different clause or remove a filter.</EmptyState>
            ) : (
              <div className="tbl-wrap">
                <table className="tbl sd-fr">
                  <thead><tr><th>ID</th><th>Formula</th><th>Standard</th><th>Clause</th><th>Since</th><th>Verification</th></tr></thead>
                  <tbody>{rows.map((f) => {
                    const s = stateOf(f.verification);
                    return (
                      <tr key={f.id}>
                        <td className="code-id">{f.id}</td>
                        <td>{f.name}</td>
                        <td className="m muted">{f.standard}</td>
                        <td className="clause">{f.clause}</td>
                        <td className="m faint">{f.module_version}</td>
                        <td><span className={`badge wrap ${s === "checked" ? "ok" : s === "pending" ? "warn" : ""}`}>{f.verification}</span></td>
                      </tr>
                    );
                  })}</tbody>
                </table>
              </div>
            )}
            <p className="sd-fine">Reference cases, cross-checks and sign-off records: docs/verification/VERIFICATION_REGISTER.md in the repository.</p>
          </section>
        </main>
      </div>
    </div>
  );
}
