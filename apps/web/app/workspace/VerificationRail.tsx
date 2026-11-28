// Verification rail: verdict, governing limit state, utilisation of every check and the observations.
"use client";
import { useState } from "react";
import { checkStatus, fmtPct, fmtUtil, governing, VERDICT_WORD, type Verdict } from "../../lib/status";
import type { ModResult } from "../../lib/types";
import { EmptyState, Segmented, StatusBadge, UtilizationBar } from "../ui/primitives";

export default function VerificationRail({ res, verdict, onOpenChecks }: { res: ModResult | null; verdict: Verdict; onOpenChecks: () => void }) {
  const [order, setOrder] = useState<"sheet" | "eta">("eta");
  if (!res || !res.ok) return <EmptyState glyph="η" title="Awaiting valid inputs">The verdict and utilisation appear as soon as every parameter is valid.</EmptyState>;
  const gov = governing(res);
  const checks = order === "eta" ? [...res.checks].sort((a, b) => b.utilisation - a.utilisation) : res.checks;
  const nPass = res.checks.filter((c) => c.pass).length;

  return (
    <div className="vr">
      <div className={`vr-verdict v-${verdict}`} aria-live="polite">
        <div className="vr-word mono">{VERDICT_WORD[verdict]}</div>
        <div className="vr-meta">
          {gov ? (<><span className="mono">η<sub>max</sub> {fmtUtil(gov.utilisation)}</span><span>{nPass}/{res.checks.length} checks</span></>) : <span>design values only</span>}
        </div>
        {verdict === "warn" && <p>All checks satisfied; at least one at or above the 0.85 review threshold.</p>}
        {verdict === "fail" && <p>{res.checks.length - nPass} check{res.checks.length - nPass > 1 ? "s" : ""} not satisfied.</p>}
        <p className="faint">Implemented checks only · not an engineer&apos;s approval</p>
      </div>

      {gov && (
        <div className="vr-sec">
          <div className="vr-h">Governing limit state</div>
          <button type="button" className={`vr-gov s-${checkStatus(gov)}`} onClick={onOpenChecks}>
            <span className="n">{gov.name}</span>
            <span className="c mono">{gov.reference}</span>
            <span className="eq mono"><span>E<sub>d</sub> {gov.demand}</span><span>R<sub>d</sub> {gov.capacity}</span></span>
            <span className="big mono">{fmtPct(gov.utilisation)}</span>
            <UtilizationBar value={gov.utilisation} status={checkStatus(gov)} label="Governing utilisation" />
          </button>
        </div>
      )}

      {res.checks.length > 0 && (
        <div className="vr-sec">
          <div className="vr-h-row"><span className="vr-h">Utilisation</span><Segmented xs label="Order" value={order} onChange={setOrder} options={[{ value: "eta", label: "η" }, { value: "sheet", label: "Sheet" }]} /></div>
          <ul className="vr-list">
            {checks.map((c) => {
              const st = checkStatus(c);
              return (
                <li key={c.name + c.reference}>
                  <div className="h"><span>{c.name}</span><b className={`mono t-${st}`}>{fmtUtil(c.utilisation)}</b></div>
                  <UtilizationBar thin value={c.utilisation} status={st} label={`${c.name} utilisation`} />
                </li>
              );
            })}
          </ul>
        </div>
      )}

      <div className="vr-sec">
        <div className="vr-h">Observations <span className="mono faint">{res.observations.length}</span></div>
        {res.observations.length === 0 ? <p className="muted small">None.</p> : (
          <ul className="vr-obs">{res.observations.map((o) => <li key={o.id}><div className="h"><span className="mono faint">{o.id}</span><StatusBadge status={o.level === "RED" ? "fail" : o.level === "AMBER" ? "warn" : "results"}>{o.level}</StatusBadge></div><p>{o.text}</p></li>)}</ul>
        )}
      </div>
    </div>
  );
}
