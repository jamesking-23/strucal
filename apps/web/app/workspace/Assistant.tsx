"use client";
import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { supabase } from "../../lib/supabase";
import { askAssistant, parseProposal, type AssistantTask } from "../../lib/assistant";
import { visibleFields } from "../../lib/form";
import type { FormValues, ModDef, ModResult } from "../../lib/types";
import { Button, Notice } from "../ui/primitives";

type Props = { open: boolean; onClose: () => void; def: ModDef; values: FormValues; res: ModResult | null; onApply: (patch: Record<string, number>) => void };
type Proposal = { proposals: Record<string, number>; missing: string[]; notes: string };

const QUICK: [AssistantTask, string][] = [["summarise", "Summarise result"], ["governing", "Why does this check govern?"], ["explain", "Explain the working"], ["missing", "What might be missing?"]];

export default function Assistant({ open, onClose, def, values, res, onApply }: Props) {
  const [signedIn, setSignedIn] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [answer, setAnswer] = useState("");
  const [err, setErr] = useState("");
  const [q, setQ] = useState("");
  const [step, setStep] = useState(-1);
  const [prop, setProp] = useState<Proposal | null>(null);
  const headRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    supabase.auth.getSession().then(({ data }) => setSignedIn(!!data.session)).catch(() => setSignedIn(false));
    const { data: s } = supabase.auth.onAuthStateChange((_e, ses) => setSignedIn(!!ses));
    return () => s.subscription.unsubscribe();
  }, []);
  // Answers describe one calculation; clear them when the module changes.
  useEffect(() => { setAnswer(""); setErr(""); setProp(null); setStep(-1); }, [def.id]);
  useEffect(() => {
    if (!open) return;
    headRef.current?.focus();
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

  return (<>
    <div className="dialog-backdrop" onClick={onClose} aria-hidden />
    <aside className="drawer" role="dialog" aria-modal="true" aria-labelledby="as-title">
      <div className="drawer-h"><h2 id="as-title" ref={headRef} tabIndex={-1}>Engineering assistant</h2><span className="spacer" /><Button variant="ghost" size="sm" onClick={onClose}>Close <kbd>esc</kbd></Button></div>
      <div className="drawer-b">
        <p className="muted" style={{ fontSize: "var(--fs-xs)" }}>Explains the calculation shown. It never calculates or changes results and cannot approve a design. Every number comes from the deterministic engine.</p>
        {signedIn === null && <div className="loading" style={{ minHeight: 80 }}><span className="spinner" />Checking sign-in…</div>}
        {signedIn === false && <Notice tone="info">Sign in on the <Link href="/account">Account page</Link> to use the assistant.</Notice>}
        {signedIn && (<>
          {!res?.ok && <Notice tone="warn">Explanations are available once the calculation runs without errors.</Notice>}
          <div className="grp">
            <div className="rail-h">Quick questions</div>
            <div className="chips">{QUICK.map(([t, l]) => <Button key={t} size="sm" disabled={busy || !res?.ok} onClick={() => call(t)}>{l}</Button>)}</div>
          </div>
          {res?.ok && (
            <div className="grp">
              <div className="rail-h">One step</div>
              <div style={{ display: "flex", gap: "var(--s2)" }}>
                <select className="select" aria-label="Step to explain" value={step} onChange={(e) => setStep(Number(e.target.value))}><option value={-1}>Choose a step…</option>{res.trace.steps.map((s, i) => <option key={i} value={i}>{i + 1}. {s.title}</option>)}</select>
                <Button disabled={busy || step < 0} onClick={() => call("explain")}>Explain</Button>
              </div>
            </div>
          )}
          <div className="grp">
            <div className="rail-h">Ask or describe</div>
            <textarea className="textarea" value={q} onChange={(e) => setQ(e.target.value)} rows={4} maxLength={2000} aria-label="Question or member description" placeholder="Ask a question, or describe the member (for example: 300 wide, 500 deep beam, C30/37) to get proposed inputs" />
            <div className="chips"><Button variant="primary" disabled={busy || !q.trim()} onClick={() => call("chat", q)}>Ask</Button><Button disabled={busy || !q.trim()} onClick={propose}>Propose inputs</Button></div>
          </div>
          {busy && <div className="loading" style={{ minHeight: 60 }} role="status"><span className="spinner" />Thinking…</div>}
          {err && <Notice tone="error">{err}</Notice>}
          {answer && <div className="card"><div className="card-b"><div className="answer">{answer}</div><span className="faint" style={{ fontSize: "var(--fs-2xs)" }}>AI-generated explanation of the deterministic calculation. Check it against the calculation sheet and the standard.</span></div></div>}
          {prop && (
            <div className="card">
              <div className="card-h"><h3>Proposed inputs</h3><span className="badge warn">Not applied</span></div>
              <div className="card-b">
                {Object.keys(prop.proposals).length === 0 ? <p className="muted">No input values could be taken from the description.</p> : (
                  <table className="tbl"><thead><tr><th>Input</th><th className="r">Now</th><th className="r">Proposed</th></tr></thead><tbody>
                    {Object.entries(prop.proposals).map(([k, v]) => <tr key={k}><td>{fieldOf(k)?.label}</td><td className="r num faint">{String(values[k])}</td><td className="r num"><b>{v}</b> {fieldOf(k)?.unit}</td></tr>)}
                  </tbody></table>)}
                {prop.missing.length > 0 && <Notice tone="warn">Still needed from you: {prop.missing.join("; ")}</Notice>}
                {prop.notes && <p className="muted">{prop.notes}</p>}
                <span className="faint" style={{ fontSize: "var(--fs-2xs)" }}>Review each value. The calculation only runs on values you confirm.</span>
              </div>
              {Object.keys(prop.proposals).length > 0 && <div className="card-f"><Button variant="primary" onClick={() => { onApply(prop.proposals); setProp(null); }}>Apply to the form</Button><Button onClick={() => setProp(null)}>Discard</Button></div>}
            </div>
          )}
        </>)}
      </div>
    </aside>
  </>);
}
