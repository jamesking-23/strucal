// Engineering assistant drawer: explains the deterministic result and proposes inputs for confirmation.
// It never calculates or changes results. Uses the shared session store, so sign-in state is always current.
"use client";
import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useSession } from "../../lib/session";
import { askAssistant, parseProposal, type AssistantTask } from "../../lib/assistant";
import { visibleFields } from "../../lib/form";
import type { FormValues, ModDef, ModResult } from "../../lib/types";
import { Button, Notice, Spinner } from "../ui/primitives";
import { IconClose, IconSpark } from "../ui/icons";

type Props = { open: boolean; onClose: () => void; def: ModDef; values: FormValues; res: ModResult | null; onApply: (patch: Record<string, number>) => void };
type Proposal = { proposals: Record<string, number>; missing: string[]; notes: string };

const QUICK: [AssistantTask, string][] = [["summarise", "Summarise the result"], ["governing", "Why does this check govern?"], ["explain", "Explain the working"], ["missing", "What might be missing?"]];

export default function Assistant({ open, onClose, def, values, res, onApply }: Props) {
  const auth = useSession();
  const [busy, setBusy] = useState(false);
  const [answer, setAnswer] = useState("");
  const [err, setErr] = useState("");
  const [q, setQ] = useState("");
  const [step, setStep] = useState(-1);
  const [prop, setProp] = useState<Proposal | null>(null);
  const head = useRef<HTMLHeadingElement>(null);

  useEffect(() => { setAnswer(""); setErr(""); setProp(null); setStep(-1); }, [def.id]);
  useEffect(() => {
    if (!open) return;
    head.current?.focus();
    const key = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    document.addEventListener("keydown", key);
    return () => document.removeEventListener("keydown", key);
  }, [open, onClose]);
  if (!open) return null;

  const fields = visibleFields(def, values.mode);
  const context = () => ({
    module: def.title, module_id: def.id, mode: values.mode, parameter_set_status: res?.param_status,
    inputs: Object.fromEntries(fields.map((f) => [f.label, `${values[f.key]} ${f.unit}`.trim()])),
    summary: res?.summary, checks: res?.checks, observations: res?.observations, scope_notes: res?.scope_notes, errors: res?.errors,
    step: step >= 0 ? res?.trace.steps[step] : undefined,
    trace: res?.trace.steps.map((s, i) => ({ n: i + 1, section: s.section, title: s.title, reference: s.reference, calculation: s.calculation, output: s.output, comment: s.comment })),
    trace_hash: res?.trace_hash,
  });
  const call = async (task: AssistantTask, question = "", ctx: unknown = context()) => {
    setBusy(true); setErr(""); setAnswer(""); setProp(null);
    try {
      const a = await askAssistant(task, question, ctx);
      if (task === "propose") setProp(parseProposal(a, new Set(fields.map((f) => f.key))));
      else setAnswer(a);
    } catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
    setBusy(false);
  };
  const propose = () => call("propose", q, { module: def.title, allowed_fields: fields.map((f) => ({ key: f.key, label: f.label, unit: f.unit })), current_inputs: Object.fromEntries(def.fields.map((f) => [f.key, values[f.key]])) });
  const fieldOf = (k: string) => def.fields.find((f) => f.key === k);
  const ready = auth.status === "signed-in";

  return (<>
    <div className="backdrop" onClick={onClose} aria-hidden />
    <aside className="drawer" role="dialog" aria-modal="true" aria-labelledby="as-title">
      <div className="drawer-h"><span className="ai-ico" data-active="true" data-busy={busy || undefined} aria-hidden><IconSpark width={13} height={13} /></span><h2 id="as-title" ref={head} tabIndex={-1}>StruCal Assistant</h2><span className="grow" /><Button size="sm" icon variant="ghost" onClick={onClose} aria-label="Close assistant"><IconClose width={14} height={14} /></Button></div>
      <div className="drawer-b">
        <p className="small muted">Explains the calculation on screen. It never calculates or changes a result and cannot approve a design; every number comes from the deterministic engine.</p>
        {auth.status === "loading" && <div className="row"><Spinner /><span className="muted small">Checking your session…</span></div>}
        {auth.status === "signed-out" && <Notice tone="info" action={<Link className="btn sm" href="/account">Sign in</Link>}>The assistant needs a StruCal account.</Notice>}
        {ready && (<>
          {!res?.ok && <Notice tone="warn">Explanations become available once the calculation runs without errors.</Notice>}
          <div className="as-sec">
            <div className="label">Quick questions</div>
            <div className="row">{QUICK.map(([t, l]) => <Button key={t} size="sm" disabled={busy || !res?.ok} onClick={() => call(t)}>{l}</Button>)}</div>
          </div>
          {res?.ok && (
            <div className="as-sec">
              <label className="label" htmlFor="as-step">Explain one step</label>
              <div className="row" style={{ flexWrap: "nowrap" }}>
                <select id="as-step" className="select" value={step} onChange={(e) => setStep(Number(e.target.value))}><option value={-1}>Choose a step…</option>{res.trace.steps.map((s, i) => <option key={i} value={i}>{String(i + 1).padStart(2, "0")} · {s.title}</option>)}</select>
                <Button disabled={busy || step < 0} onClick={() => call("explain")}>Explain</Button>
              </div>
            </div>
          )}
          <div className="as-sec">
            <label className="label" htmlFor="as-q">Ask, or describe a member</label>
            <textarea id="as-q" className="textarea" value={q} onChange={(e) => setQ(e.target.value)} rows={4} maxLength={2000} placeholder="e.g. 300 wide, 500 deep beam, C30/37, 200 kNm — to get proposed inputs" />
            <div className="row"><Button variant="primary" disabled={busy || !q.trim()} onClick={() => call("chat", q)}>Ask</Button><Button disabled={busy || !q.trim()} onClick={propose}>Propose inputs</Button></div>
          </div>
          {busy && <div className="row"><Spinner /><span className="muted small">Thinking…</span></div>}
          {err && <Notice tone="error">{err}</Notice>}
          {answer && <div className="as-answer"><div>{answer}</div><p className="faint small">AI-generated explanation of the deterministic calculation. Verify against the ledger and the standard.</p></div>}
          {prop && (
            <div className="as-prop">
              <div className="sh">Proposed inputs <span className="meta"><span className="badge warn">not applied</span></span></div>
              {Object.keys(prop.proposals).length === 0 ? <p className="muted small as-pad">No input values could be taken from the description.</p> : (
                <table className="tbl"><thead><tr><th>Input</th><th className="r">Now</th><th className="r">Proposed</th></tr></thead><tbody>
                  {Object.entries(prop.proposals).map(([k, v]) => <tr key={k}><td>{fieldOf(k)?.label}</td><td className="m r faint">{String(values[k])}</td><td className="m r"><b>{v}</b> {fieldOf(k)?.unit}</td></tr>)}
                </tbody></table>)}
              {prop.missing.length > 0 && <div className="as-pad"><Notice tone="warn">Still needed: {prop.missing.join("; ")}</Notice></div>}
              {prop.notes && <p className="muted small as-pad">{prop.notes}</p>}
              {Object.keys(prop.proposals).length > 0 && <div className="row as-pad"><Button variant="primary" onClick={() => { onApply(prop.proposals); setProp(null); }}>Apply to inputs</Button><Button onClick={() => setProp(null)}>Discard</Button></div>}
            </div>
          )}
        </>)}
      </div>
    </aside>
  </>);
}
