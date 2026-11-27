// Client for the "assistant" Supabase edge function. The assistant explains results; it never calculates.
import { supabase } from "./supabase";

export type AssistantTask = "summarise" | "governing" | "explain" | "missing" | "chat" | "propose" | "compare";

/** Calls the assistant and returns its text answer, or throws an Error with a readable message. */
export async function askAssistant(task: AssistantTask, question: string, context: unknown): Promise<string> {
  const { data, error } = await supabase.functions.invoke("assistant", { body: { task, question, context } });
  if (error) {
    let msg = error.message;
    try {
      const body = await (error as { context?: { json?: () => Promise<{ error?: string }> } }).context?.json?.();
      if (body?.error) msg = body.error;
    } catch { /* keep the transport message */ }
    throw new Error(msg);
  }
  if (data?.error) throw new Error(String(data.error));
  if (typeof data?.answer !== "string" || !data.answer.trim()) throw new Error("The assistant returned an empty answer.");
  return data.answer;
}

/** Parses a "propose" answer into numeric proposals restricted to the allowed field keys. */
export function parseProposal(answer: string, allowed: Set<string>): { proposals: Record<string, number>; missing: string[]; notes: string } {
  const txt = answer.trim().replace(/^```(?:json)?/i, "").replace(/```$/, "").trim();
  let j: { proposals?: Record<string, unknown>; missing?: unknown; notes?: unknown };
  try { j = JSON.parse(txt); } catch { throw new Error("The assistant's proposal could not be read. Try rephrasing the description."); }
  const proposals: Record<string, number> = {};
  for (const [k, v] of Object.entries(j.proposals ?? {})) if (allowed.has(k) && typeof v === "number" && Number.isFinite(v)) proposals[k] = v;
  return { proposals, missing: Array.isArray(j.missing) ? j.missing.map(String) : [], notes: j.notes ? String(j.notes) : "" };
}
