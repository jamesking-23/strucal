// Verification status vocabulary shared by every view.
//   pass  — check satisfied, utilisation below 0.85        (emerald)
//   warn  — check satisfied, utilisation ≥ 0.85 (review)   (amber)
//   fail  — check not satisfied / overstressed             (crimson)
import type { Check, ModResult } from "./types";

export const REVIEW_THRESHOLD = 0.85;
export type Status = "pass" | "warn" | "fail";
export type Verdict = Status | "results" | "none";

export function statusOf(pass: boolean, utilisation: number): Status {
  if (!pass) return "fail";
  return utilisation >= REVIEW_THRESHOLD ? "warn" : "pass";
}
export const checkStatus = (c: Check): Status => statusOf(c.pass, c.utilisation);

/** Overall verdict of a result; "results" for modules that give design values without checks. */
export function verdictOf(r: ModResult | null): Verdict {
  if (!r || !r.ok) return "none";
  if (r.checks.length === 0) return "results";
  if (!r.overall_pass) return "fail";
  return r.checks.some((c) => c.utilisation >= REVIEW_THRESHOLD) ? "warn" : "pass";
}

/** The check with the highest utilisation (the governing limit state). */
export function governing(r: ModResult | null): Check | null {
  if (!r || !r.ok || r.checks.length === 0) return null;
  return r.checks.reduce((a, b) => (b.utilisation > a.utilisation ? b : a));
}

/** Utilisation as shown to engineers; the engine saturates unbounded ratios at 9.99. */
export const fmtUtil = (u: number) => (u >= 9.99 ? "n/a" : u.toFixed(2));
export const fmtPct = (u: number) => (u >= 9.99 ? "n/a" : `${Math.round(u * 100)}%`);

export const VERDICT_WORD: Record<Verdict, string> = { pass: "PASS", warn: "PASS", fail: "FAIL", results: "RESULTS", none: "—" };
