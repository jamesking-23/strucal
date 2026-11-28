// Verification ledger: the auditable record of a run.
//   Ledger  — every step (reference · governing equation · substitution · result), grouped by section
//   Checks  — itemised code checks with utilisation bars, governing check marked
//   Summary — result values; Scope — limitations and the reproducible trace hash
"use client";
import { useMemo, useState } from "react";
import { checkStatus, fmtUtil, governing } from "../../lib/status";
import { Markup } from "../../lib/markup";
import type { ModResult, Step } from "../../lib/types";
import { Button, StatusBadge, Tabs, UtilizationBar } from "../ui/primitives";
import CalculationRow from "../ui/CalculationRow";

type Tab = "ledger" | "checks" | "summary" | "scope";

function group(steps: Step[]) {
  const out: { section: string; rows: { s: Step; n: number }[] }[] = [];
  steps.forEach((s, i) => {
    const last = out[out.length - 1];
    if (last && last.section === s.section) last.rows.push({ s, n: i + 1 });
    else out.push({ section: s.section, rows: [{ s, n: i + 1 }] });
  });
  return out;
}

export default function Ledger({ res, paramStatus, tab, onTab }: { res: ModResult; paramStatus: string; tab: Tab; onTab: (t: Tab) => void }) {
  const sections = useMemo(() => group(res.trace.steps), [res]);
  const [closed, setClosed] = useState<Set<number>>(new Set());
  const gov = governing(res);
  const toggle = (i: number) => setClosed((c) => { const n = new Set(c); if (n.has(i)) n.delete(i); else n.add(i); return n; });

  return (
    <section className="lg" aria-label="Verification ledger">
      <Tabs label="Ledger" value={tab} onChange={onTab} tabs={[
        { value: "ledger", label: <>Ledger <span className="tab-n mono">{res.trace.steps.length}</span></> },
        { value: "checks", label: <>Checks <span className="tab-n mono">{res.checks.length}</span></> },
        { value: "summary", label: "Summary" },
        { value: "scope", label: "Scope" },
      ]} tools={tab === "ledger" ? <>
        <Button size="sm" variant="ghost" onClick={() => setClosed(new Set())}>Expand all</Button>
        <Button size="sm" variant="ghost" onClick={() => setClosed(new Set(sections.map((_, i) => i)))}>Collapse all</Button>
      </> : undefined} />

      <div className="lg-b">
        {(/NOT/.test(paramStatus) || res.warnings.length > 0) && (
          <div className="lg-flags">
            {/NOT/.test(paramStatus) && <div className="lg-flag warn"><b>Parameter set</b> {paramStatus}. Confirm the governing National Annex before issue for construction.</div>}
            {res.warnings.map((w, i) => <div className="lg-flag warn" key={i}>{w}</div>)}
          </div>
        )}

        {tab === "ledger" && (
          <div role="table" aria-label="Calculation steps">
            <div className="cr cr-head" role="row"><span role="columnheader">#</span><span role="columnheader">Reference</span><span role="columnheader">Equation · substitution</span><span role="columnheader">Result</span></div>
            {sections.map((sec, i) => (
              <div key={i} className="lg-sec">
                <button type="button" className="lg-sec-h" aria-expanded={!closed.has(i)} onClick={() => toggle(i)}>
                  <span className="chev" aria-hidden /><Markup text={sec.section} /><span className="meta mono">{sec.rows.length} {sec.rows.length === 1 ? "step" : "steps"}</span>
                </button>
                {!closed.has(i) && sec.rows.map(({ s, n }) => <CalculationRow key={n} n={n} step={s} />)}
              </div>
            ))}
          </div>
        )}

        {tab === "checks" && (res.checks.length === 0 ? (
          <p className="lg-note">This module combines action effects and gives design values, not a PASS/FAIL verdict. See Summary.</p>
        ) : (
          <div className="tbl-wrap">
            <table className="tbl lg-checks">
              <thead><tr><th>#</th><th>Limit state / check</th><th>Clause</th><th className="r">E<sub className="nt">d</sub></th><th className="r">R<sub className="nt">d</sub></th><th className="r"><span className="nt">η</span></th><th>Utilisation</th><th className="r">Status</th></tr></thead>
              <tbody>{res.checks.map((c, i) => {
                const st = checkStatus(c);
                return (
                  <tr key={i} className={c === gov ? "gov" : undefined}>
                    <td className="m faint">{String(i + 1).padStart(2, "0")}</td>
                    <td>{c.name}{c === gov && <span className="badge solid gov-b">governing</span>}</td>
                    <td className="m t-signal">{c.reference}</td>
                    <td className="m r">{c.demand}</td>
                    <td className="m r">{c.capacity}</td>
                    <td className={`m r eta t-${st}`}>{fmtUtil(c.utilisation)}</td>
                    <td className="ubar-cell"><UtilizationBar value={c.utilisation} status={st} label={`${c.name} utilisation`} /></td>
                    <td className="r"><StatusBadge status={st}>{st === "fail" ? "Fail" : st === "warn" ? "≥ 0.85" : "OK"}</StatusBadge></td>
                  </tr>
                );
              })}</tbody>
            </table>
            <p className="lg-note">η = E<sub>d</sub> / R<sub>d</sub>. Amber at η ≥ 0.85 (review), crimson when a check fails. Implemented checks only.</p>
          </div>
        ))}

        {tab === "summary" && <dl className="kv lg-summary">{res.summary.map((k, i) => <div key={i}><dt>{k.label}</dt><dd>{k.value}</dd></div>)}</dl>}

        {tab === "scope" && (
          <div className="lg-scope">
            <ul>{res.scope_notes.map((t, i) => <li key={i}>{t}</li>)}</ul>
            <div className="lg-hash mono"><span>SHA-256</span><b>{res.trace_hash}</b><span>engine v{res.engine_version} · identical inputs reproduce this hash</span></div>
          </div>
        )}
      </div>
    </section>
  );
}
