"use client";
import { useMemo, useState } from "react";
import TopBar from "../components/TopBar";
import { useEngine } from "../../lib/useEngine";
import type { Formula } from "../../lib/types";

const FACTORS: [keyof import("../../lib/types").ParamSet, string][] = [["gamma_c", "γc concrete"], ["gamma_s", "γs reinforcement"], ["alpha_cc", "αcc"], ["gamma_g", "γG permanent"], ["gamma_q", "γQ variable"], ["gamma_m0", "γM0"], ["gamma_m1", "γM1"]];

/** Badge style for a verification state as recorded in the engine. */
function stateClass(v: string) {
  if (/^CHECKED/.test(v)) return "ok";
  if (/PENDING/.test(v)) return "warn";
  return "";
}

export default function Standards() {
  const { engine: eng, error, retry } = useEngine();
  const [q, setQ] = useState("");
  const groups = useMemo(() => {
    const t = q.trim().toLowerCase();
    const rows = (eng?.formulas ?? []).filter((f) => !t || [f.id, f.name, f.standard, f.clause, f.verification].some((x) => x.toLowerCase().includes(t)));
    const m = new Map<string, Formula[]>();
    for (const f of rows) m.set(f.standard, [...(m.get(f.standard) ?? []), f]);
    return [...m.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  }, [eng, q]);
  const shown = groups.reduce((s, [, r]) => s + r.length, 0);

  return (
    <div className="app">
      <TopBar section="standards" context={eng ? `Engine v${eng.version}` : undefined} />
      <main className="page">
        <header className="page-head">
          <span className="eyebrow">Standards</span>
          <h1>Formula registry and parameter sets</h1>
          <p>Every implemented formula with its source clause and verification state, and the partial factors each parameter set applies. States are shown exactly as recorded in the engine.</p>
        </header>
        {!eng && (error
          ? <div className="card"><div className="empty"><h2>The calculation engine did not load</h2><p>{error}</p><button className="btn btn-primary" onClick={retry}>Try again</button></div></div>
          : <div className="loading" role="status"><span className="spinner" />Loading calculation engine…</div>)}
        {eng && (<>
          <section className="grp" aria-labelledby="ps-h">
            <div className="section-title" id="ps-h">Parameter sets <span className="num">{eng.paramSets.length}</span></div>
            <div className="grid-cards">
              {eng.paramSets.map((p) => (
                <article className="card" key={p.id}>
                  <div className="card-h"><h3>{p.label}</h3></div>
                  <div className="card-b">
                    <span className={`badge wrap ${/NOT/.test(p.status) ? "warn" : "ok"}`}>{p.status}</span>
                    <dl className="kv">{FACTORS.map(([k, l]) => <div key={k}><dt>{l}</dt><dd>{String(p[k])}</dd></div>)}</dl>
                    <p className="faint small">{p.source}</p>
                  </div>
                </article>
              ))}
            </div>
          </section>

          <section className="grp" aria-labelledby="fr-h">
            <div className="section-title" id="fr-h">Formula registry <span className="num">{shown} of {eng.formulas.length}</span></div>
            <div className="field mw-420">
              <label htmlFor="fr-q">Filter</label>
              <input id="fr-q" className="input" type="search" placeholder="ID, name, clause or state, e.g. EC3 or PENDING" value={q} onChange={(e) => setQ(e.target.value)} />
            </div>
            {groups.length === 0 && <div className="card"><div className="empty"><h2>No formulas match “{q}”</h2><button className="btn" onClick={() => setQ("")}>Clear filter</button></div></div>}
            {groups.map(([std, rows]) => (
              <article className="card" key={std}>
                <div className="card-h"><h3>{std}</h3><span className="spacer" /><span className="num faint">{rows.length}</span></div>
                <div className="tbl-wrap">
                  <table className="tbl">
                    <thead><tr><th>ID</th><th>Formula</th><th>Clause</th><th>Since</th><th>Verification</th></tr></thead>
                    <tbody>{rows.map((f) => (
                      <tr key={f.id}>
                        <td className="code-id">{f.id}</td>
                        <td>{f.name}</td>
                        <td className="clause">{f.clause}</td>
                        <td className="code-id faint">{f.module_version}</td>
                        <td><span className={`badge wrap ${stateClass(f.verification)}`}>{f.verification}</span></td>
                      </tr>))}
                    </tbody>
                  </table>
                </div>
              </article>
            ))}
          </section>
          <p className="faint small">Reference cases, cross-checks and sign-off records: docs/verification/VERIFICATION_REGISTER.md in the repository.</p>
        </>)}
      </main>
    </div>
  );
}
