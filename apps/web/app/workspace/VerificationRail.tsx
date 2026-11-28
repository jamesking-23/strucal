// Verification rail: verdict, governing limit state, utilisation of every check and the observations.
"use client";
import { useState } from "react";
import { checkStatus, fmtUtil, governing, type Verdict } from "../../lib/status";
import type { ModResult } from "../../lib/types";
import { EmptyState, Segmented, StatusBadge, UtilBar } from "../ui/primitives";

const WORD: Record<Verdict, string> = { ok: "PASS", warn: "PASS", fail: "FAIL", results: "RESULTS", none: "—" };

export default function VerificationRail({ res, verdict, onShowScope }: { res: ModResult | null; verdict: Verdict; onShowScope?: () => void }) {
  const [order, setOrder] = useState<"sheet" | "eta">("sheet");
  if (!res || !res.ok) return <EmptyState glyph="η" title="No verification yet">Results appear as soon as every input is valid.</EmptyState>;
  const gov = governing(res);
  const nPass = res.checks.filter((c) => c.pass).length;
  const checks = order === "eta" ? [...res.checks].sort((a, b) => b.utilisation - a.utilisation) : res.checks;

  return (
    <div className="rail">
      <div className={`verdict v-${verdict}`} aria-live="polite">
        <div className="v-row"><b>{WORD[verdict]}</b>{gov && <span className="mono">η<sub>max</sub> {fmtUtil(gov.utilisation)}</span>}</div>
        <p>{res.checks.length === 0 ? "Design values only; no PASS/FAIL verdict." : verdict === "warn" ? `${nPass} of ${res.checks.length} checks satisfied; at least one above the 0.85 review threshold.` : `${nPass} of ${res.checks.length} checks satisfied.`}</p>
        <p className="faint">Implemented checks only. Not an engineer&apos;s approval.</p>
      </div>

      {gov && (
        <section className="rail-sec" aria-labelledby="gov-h">
          <h3 id="gov-h" className="rail-h">Governing limit state</h3>
          <div className="gov">
            <div className="gov-name">{gov.name}</div>
            <div className="gov-clause mono">{gov.reference}</div>
            <div className="gov-eq mono"><span>E<sub>d</sub> {gov.demand}</span><span>R<sub>d</sub> {gov.capacity}</span></div>
            <UtilBar value={gov.utilisation} status={checkStatus(gov)} label="Governing utilisation" />
          </div>
        </section>
      )}

      {res.checks.length > 0 && (
        <section className="rail-sec" aria-labelledby="chk-h">
          <div className="rail-h-row"><h3 id="chk-h" className="rail-h">Utilisation <span className="nt">η = E<sub>d</sub>/R<sub>d</sub></span></h3>
            <Segmented label="Order checks" value={order} onChange={setOrder} options={[{ value: "sheet", label: "Sheet" }, { value: "eta", label: "η" }]} />
          </div>
          <ul className="chk-list">
            {checks.map((c, i) => {
              const st = checkStatus(c);
              return (
                <li key={i} className={c === gov ? "gov-row" : undefined}>
                  <div className="h"><span>{c.name}</span><b className={`mono t-${st}`}>{fmtUtil(c.utilisation)}</b></div>
                  <UtilBar value={c.utilisation} status={st} thin label={`${c.name} utilisation`} />
                  <div className="d mono"><span>{c.demand} / {c.capacity}</span></div>
                </li>
              );
            })}
          </ul>
        </section>
      )}

      <section className="rail-sec" aria-labelledby="obs-h">
        <h3 id="obs-h" className="rail-h">Observations <span className="mono faint">{res.observations.length}</span></h3>
        {res.observations.length === 0 ? <p className="muted small">None.</p> : (
          <ul className="obs-list">
            {res.observations.map((o) => (
              <li key={o.id}><div className="h"><span className="mono faint">{o.id}</span><StatusBadge status={o.level === "RED" ? "fail" : o.level === "AMBER" ? "warn" : "results"}>{o.level}</StatusBadge></div><p>{o.text}</p></li>
            ))}
          </ul>
        )}
        {onShowScope && <button type="button" className="link-btn" onClick={onShowScope}>Scope and limitations →</button>}
      </section>
    </div>
  );
}
