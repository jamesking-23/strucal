// Calculation sheet: an auditable record of the run — design basis, itemised limit-state checks,
// result summary, then every step (reference · working · output) in collapsible sections, scope and trace hash.
"use client";
import { useMemo, useState } from "react";
import { checkStatus, fmtUtil, governing, type Verdict } from "../../lib/status";
import type { FormValues, ModDef, ModResult, Step } from "../../lib/types";
import { Button, StatusBadge, UtilBar } from "../ui/primitives";
import { Markup } from "../../lib/markup";

type Props = { def: ModDef; values: FormValues; res: ModResult; verdict: Verdict; paramLabel: string; paramStatus: string };

function groupSteps(steps: Step[]) {
  const out: { section: string; steps: { s: Step; n: number }[] }[] = [];
  steps.forEach((s, i) => {
    const last = out[out.length - 1];
    if (last && last.section === s.section) last.steps.push({ s, n: i + 1 });
    else out.push({ section: s.section, steps: [{ s, n: i + 1 }] });
  });
  return out;
}

export default function CalcSheet({ def, values, res, verdict, paramLabel, paramStatus }: Props) {
  const sections = useMemo(() => groupSteps(res.trace.steps), [res]);
  const gov = governing(res);
  const [openAll, setOpenAll] = useState<boolean | null>(null);
  const [rev, setRev] = useState(0);
  const toggleAll = (o: boolean) => { setOpenAll(o); setRev((r) => r + 1); };

  return (
    <article className="sheet" aria-label="Calculation sheet">
      <header className="sheet-head">
        <div className="sheet-title">
          <span className="eyebrow">{values.mode === "check" ? "Independent check" : "Design"} · calculation sheet</span>
          <h1>{def.title}</h1>
          <p className="mono">{def.subtitle}</p>
        </div>
        <dl className="sheet-meta">
          <div><dt>Result</dt><dd><StatusBadge status={verdict} /></dd></div>
          <div><dt>Governing η</dt><dd className="mono">{gov ? fmtUtil(gov.utilisation) : "—"}</dd></div>
          <div><dt>Parameter set</dt><dd title={paramLabel}>{paramLabel}</dd></div>
          <div><dt>Engine</dt><dd className="mono">v{res.engine_version}</dd></div>
          <div><dt>Trace</dt><dd className="mono" title={res.trace_hash}>{res.trace_hash.slice(0, 12)}</dd></div>
        </dl>
      </header>

      {/NOT/.test(paramStatus) && <div className="alert warn"><span className="grow"><b>Parameter set:</b> {paramStatus}. Confirm the governing National Annex before issue for construction.</span></div>}
      {res.warnings.map((w, i) => <div className="alert warn" key={i}><span className="grow">{w}</span></div>)}

      {res.checks.length > 0 ? (
        <section className="sheet-sec" aria-labelledby="ls-h">
          <h2 id="ls-h" className="sheet-h2"><span>A</span>Limit-state verification</h2>
          <div className="tbl-wrap">
            <table className="tbl ls">
              <thead><tr><th>#</th><th>Check</th><th>Clause</th><th className="r"><span className="nt">E<sub>d</sub></span></th><th className="r"><span className="nt">R<sub>d</sub></span></th><th className="r"><span className="nt">η</span></th><th className="bar"><span className="sr-only">Utilisation bar</span></th><th className="r">Status</th></tr></thead>
              <tbody>
                {res.checks.map((c, i) => {
                  const st = checkStatus(c);
                  return (
                    <tr key={i} className={c === gov ? "is-gov" : undefined}>
                      <td className="m faint">{String(i + 1).padStart(2, "0")}</td>
                      <td>{c.name}{c === gov && <span className="gov-tag">governing</span>}</td>
                      <td className="m clause">{c.reference}</td>
                      <td className="m r">{c.demand}</td>
                      <td className="m r">{c.capacity}</td>
                      <td className={`m r eta t-${st}`}>{fmtUtil(c.utilisation)}</td>
                      <td className="bar"><UtilBar value={c.utilisation} status={st} thin label={`${c.name} utilisation`} /></td>
                      <td className="r"><StatusBadge status={st}>{st === "fail" ? "Fail" : st === "warn" ? "Review" : "OK"}</StatusBadge></td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          <p className="sheet-fine">η = E<sub>d</sub> / R<sub>d</sub>. Review threshold 0.85; limit 1.00. Checks listed are the implemented checks only.</p>
        </section>
      ) : (
        <section className="sheet-sec"><h2 className="sheet-h2"><span>A</span>Design values</h2><p className="sheet-fine">This module combines action effects. It gives design values, not a PASS/FAIL verdict.</p></section>
      )}

      <section className="sheet-sec" aria-labelledby="rs-h">
        <h2 id="rs-h" className="sheet-h2"><span>B</span>Result summary</h2>
        <dl className="kv two">{res.summary.map((k, i) => <div key={i}><dt>{k.label}</dt><dd>{k.value}</dd></div>)}</dl>
      </section>

      <section className="sheet-sec" aria-labelledby="cw-h">
        <div className="sheet-h2-row">
          <h2 id="cw-h" className="sheet-h2"><span>C</span>Calculation <em>{res.trace.steps.length} steps · deterministic</em></h2>
          <span className="spacer" />
          <Button size="sm" variant="ghost" onClick={() => toggleAll(true)}>Expand all</Button>
          <Button size="sm" variant="ghost" onClick={() => toggleAll(false)}>Collapse all</Button>
        </div>
        <div className="steps-head" aria-hidden><span>#</span><span>Reference</span><span>Working</span><span>Output</span></div>
        {sections.map((sec, si) => (
          <details key={`${rev}-${si}`} className="step-sec" open={openAll ?? true}>
            <summary><span className="t"><Markup text={sec.section} /></span><span className="meta">{sec.steps.length} {sec.steps.length === 1 ? "step" : "steps"}</span></summary>
            {sec.steps.map(({ s, n }) => (
              <div className="step" key={n}>
                <span className="n">{String(n).padStart(2, "0")}</span>
                <span className="ref">{s.reference ? <Markup text={s.reference} /> : "—"}</span>
                <div className="body">
                  <div className="t"><Markup text={s.title} /></div>
                  {s.calculation && <div className="ca">{s.calculation}</div>}
                  {s.comment && <div className="cm"><Markup text={s.comment} /></div>}
                </div>
                <span className={`out${s.output === "PASS" ? " t-ok" : s.output === "FAIL" ? " t-fail" : ""}`}>{s.output}</span>
              </div>
            ))}
          </details>
        ))}
      </section>

      <section className="sheet-sec" aria-labelledby="sc-h">
        <h2 id="sc-h" className="sheet-h2"><span>D</span>Scope and limitations</h2>
        <ul className="scope">{res.scope_notes.map((t, i) => <li key={i}>{t}</li>)}</ul>
        <p className="hash">SHA-256 {res.trace_hash} · engine v{res.engine_version} · identical inputs reproduce this hash</p>
      </section>
    </article>
  );
}
