// One step of the verification ledger: reference · governing equation · substituted values · result.
import { Markup } from "../../lib/markup";
import type { Step } from "../../lib/types";

export default function CalculationRow({ n, step }: { n: number; step: Step }) {
  const verdict = step.output === "PASS" ? "t-pass" : step.output === "FAIL" ? "t-fail" : "";
  return (
    <div className="cr" role="row">
      <span className="cr-n mono" role="cell">{String(n).padStart(2, "0")}</span>
      <span className="cr-ref mono" role="cell">{step.reference ? <Markup text={step.reference} /> : "—"}</span>
      <div className="cr-body" role="cell">
        <div className="cr-t"><Markup text={step.title} /></div>
        {step.formula && <div className="cr-eq"><Markup text={step.formula} /></div>}
        {step.substitution && <div className="cr-sub"><Markup text={step.substitution} /></div>}
        {step.comment && <div className="cr-cm"><Markup text={step.comment} /></div>}
      </div>
      <span className={`cr-out mono ${verdict}`} role="cell">{step.result ? <Markup text={step.result} /> : ""}</span>
    </div>
  );
}
