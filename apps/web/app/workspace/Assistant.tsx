"use client";
import { useEffect, useState } from "react";
import Link from "next/link";
import { supabase } from "../../lib/supabase";
import { askAssistant, parseProposal, type AssistantTask } from "../../lib/assistant";
import { visibleFields } from "../../lib/form";
import type { FormValues, ModDef, ModResult } from "../../lib/types";

type Props = { def: ModDef; values: FormValues; res: ModResult | null; onApply: (patch: Record<string, number>) => void };
type Proposal = { proposals: Record<string, number>; missing: string[]; notes: string };

const QUICK: [AssistantTask, string][] = [["summarise", "Summarise result"], ["governing", "Why does this check govern?"], ["explain", "Explain the working"], ["missing", "What might be missing?"]];

export default function Assistant({ def, values, res, onApply }: Props) {
  const [open, setOpen] = useState(false);
  const [signedIn, setSignedIn] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [answer, setAnswer] = useState("");
  const [err, setErr] = useState("");
  const [q, setQ] = useState("");
  const [step, setStep] = useState(-1);
  const [prop, setProp] = useState<Proposal | null>(null);

  useEffect(() => {
    supabase.auth.getSession().then(({ data }) => setSignedIn(!!data.session)).catch(() => setSignedIn(false));
    const { data: s } = supabase.auth.onAuthStateChange((_e, ses) => setSignedIn(!!ses));
    return () => s.subscription.unsubscribe();
  }, []);
  // Answers describe one calculation; clear them when the module changes.
  useEffect(() => { setAnswer(""); setErr(""); setProp(null); setStep(-1); }, [def.id]);

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

  const box: React.CSSProperties = { position: "fixed", right: 14, bottom: 34, width: 380, maxWidth: "calc(100vw - 28px)", maxHeight: "72vh", overflow: "auto", background: "#fff", border: "1px solid #8b95a1", boxShadow: "0 8px 28px rgba(0,0,0,.25)", zIndex: 60, font: "13px 'Segoe UI',sans-serif", color: "#1f2933" };
  const btn: React.CSSProperties = { height: 26, padding: "0 10px", border: "1px solid #aab1b9", background: "#fff", cursor: "pointer", font: "inherit", fontSize: 12 };
  return (<>
    <button onClick={() => setOpen(true)} aria-expanded={open} style={{ position: "fixed", right: 14, bottom: 34, zIndex: 61, height: 32, padding: "0 14px", border: "1px solid #2b579a", background: "#2b579a", color: "#fff", cursor: "pointer", font: "600 13px 'Segoe UI',sans-serif", display: open ? "none" : "block" }}>Assistant</button>
    {open && (<div style={box} role="dialog" aria-label="StruCal Assistant">
      <div style={{ display: "flex", alignItems: "center", padding: "8px 10px", background: "#2b579a", color: "#fff" }}><b style={{ flex: 1 }}>StruCal Assistant</b><button onClick={() => setOpen(false)} style={{ ...btn, height: 22, background: "transparent", color: "#fff", border: "1px solid #7da0d6" }}>Close</button></div>
      <div style={{ padding: 10 }}>
        <div style={{ fontSize: 11.5, color: "#52606d", marginBottom: 8 }}>The assistant explains the calculation shown. It never calculates or changes results, and cannot approve a design. All numbers come from the deterministic engine.</div>
        {signedIn === null && <div style={{ color: "#52606d" }}>Checking sign-in…</div>}
        {signedIn === false && <div style={{ background: "#fff4d6", borderLeft: "3px solid #e0a82e", padding: 8 }}>Sign in on the <Link href="/account">Account page</Link> to use the assistant.</div>}
        {signedIn && (<>
          {!res?.ok && <div style={{ color: "#8a5a00", marginBottom: 6 }}>Explanations are available once the calculation runs without errors.</div>}
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>{QUICK.map(([t, l]) => <button key={t} style={btn} disabled={busy || !res?.ok} onClick={() => call(t)}>{l}</button>)}</div>
          {res?.ok && <div style={{ marginTop: 8 }}><label style={{ fontSize: 11.5 }} htmlFor="as-step">Explain one step: </label><select id="as-step" value={step} onChange={(e) => setStep(Number(e.target.value))} style={{ maxWidth: 230, fontSize: 12 }}><option value={-1}>(whole calculation)</option>{res.trace.steps.map((s, i) => <option key={i} value={i}>{i + 1}. {s.title}</option>)}</select> <button style={btn} disabled={busy || step < 0} onClick={() => call("explain")}>Explain</button></div>}
          <div style={{ marginTop: 10, borderTop: "1px solid #e4e7eb", paddingTop: 8 }}>
            <textarea value={q} onChange={(e) => setQ(e.target.value)} rows={3} maxLength={2000} aria-label="Question or member description" placeholder="Ask a question, or describe the member in words (for example: 300 wide, 500 deep beam, C30/37 concrete) to get proposed inputs" style={{ width: "100%", font: "inherit", fontSize: 12.5, padding: 6, border: "1px solid #aab1b9" }} />
            <div style={{ display: "flex", gap: 6, marginTop: 4 }}><button style={btn} disabled={busy || !q.trim()} onClick={() => call("chat", q)}>Ask</button><button style={btn} disabled={busy || !q.trim()} onClick={propose}>Propose inputs from description</button></div>
          </div>
          {busy && <div style={{ marginTop: 8, color: "#52606d" }} role="status">Thinking…</div>}
          {err && <div style={{ marginTop: 8, background: "#fdeceb", borderLeft: "3px solid #b42318", padding: 8, color: "#b42318" }} role="alert">{err}</div>}
          {answer && <div style={{ marginTop: 8 }}><div style={{ whiteSpace: "pre-wrap", lineHeight: 1.5 }}>{answer}</div><div style={{ marginTop: 6, fontSize: 11, color: "#7b8794" }}>AI-generated explanation of the deterministic calculation. Check it against the calculation sheet and the standard.</div></div>}
          {prop && (<div style={{ marginTop: 8, border: "1px solid #c9ced6", padding: 8 }}>
            <b>Proposed inputs (not applied yet)</b>
            {Object.keys(prop.proposals).length === 0 ? <div style={{ color: "#52606d" }}>No input values could be taken from the description.</div> : (<table style={{ width: "100%", marginTop: 4, fontSize: 12 }}><tbody>{Object.entries(prop.proposals).map(([k, v]) => <tr key={k}><td>{fieldOf(k)?.label}</td><td style={{ color: "#7b8794" }}>{String(values[k])}</td><td>→</td><td><b>{v}</b> {fieldOf(k)?.unit}</td></tr>)}</tbody></table>)}
            {prop.missing.length > 0 && <div style={{ marginTop: 6, color: "#8a5a00" }}>Still needed from you: {prop.missing.join("; ")}</div>}
            {prop.notes && <div style={{ marginTop: 4, color: "#52606d" }}>{prop.notes}</div>}
            {Object.keys(prop.proposals).length > 0 && <div style={{ marginTop: 6 }}><button style={{ ...btn, background: "#2b579a", color: "#fff" }} onClick={() => { onApply(prop.proposals); setProp(null); }}>Apply to the form</button> <button style={btn} onClick={() => setProp(null)}>Discard</button></div>}
            <div style={{ marginTop: 4, fontSize: 11, color: "#7b8794" }}>Review each value. The calculation only runs on values you confirm.</div>
          </div>)}
        </>)}
      </div>
    </div>)}
  </>);
}
