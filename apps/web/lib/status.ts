// Verification status vocabulary shared by every view. Thresholds mirror the engine's observations:
// a passing check above 0.85 is flagged for review; any failing check is "fail".
import type { Check, ModResult } from "./types";

export const REVIEW_THRESHOLD = 0.85;
export type Status = "ok" | "warn" | "fail";
export type Verdict = "ok" | "warn" | "fail" | "results" | "none";

export function checkStatus(c: Check): Status {
  if (!c.pass) return "fail";
  return c.utilisation > REVIEW_THRESHOLD ? "warn" : "ok";
}

/** Overall verdict of a result: "results" for modules that give design values without checks. */
export function verdictOf(r: ModResult | null): Verdict {
  if (!r || !r.ok) return "none";
  if (r.checks.length === 0) return "results";
  if (!r.overall_pass) return "fail";
  return r.checks.some((c) => c.utilisation > REVIEW_THRESHOLD) ? "warn" : "ok";
}

/** The check with the highest utilisation (the governing limit state). */
export function governing(r: ModResult | null): Check | null {
  if (!r || !r.ok || r.checks.length === 0) return null;
  return r.checks.reduce((a, b) => (b.utilisation > a.utilisation ? b : a));
}

/** Utilisation as shown to engineers: two decimals, "n/a" for the engine's saturated sentinel. */
export const fmtUtil = (u: number) => (u >= 9.99 ? "n/a" : u.toFixed(2));
