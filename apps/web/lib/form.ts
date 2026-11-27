// Form helpers shared by the workspace and project pages. They convert between what the user types and
// the numeric JSON the engine expects. They never compute engineering values.
import type { Field, FormValues, ModDef, Mode } from "./types";
import { DEFAULT_PARAM_SET } from "./engine";

/** Fields that are shown (and sent) in the given mode. */
export const visibleFields = (def: ModDef, mode: Mode): Field[] => def.fields.filter((f) => !f.check_only || mode === "check");

/** Field groups in definition order, for the given mode. */
export function fieldGroups(def: ModDef, mode: Mode): { group: string; fields: Field[] }[] {
  const out: { group: string; fields: Field[] }[] = [];
  for (const f of visibleFields(def, mode)) {
    const g = out.find((x) => x.group === f.group);
    if (g) g.fields.push(f); else out.push({ group: f.group, fields: [f] });
  }
  return out;
}

export function initialValues(def: ModDef): FormValues {
  const v: FormValues = { mode: "design", param_set: DEFAULT_PARAM_SET };
  for (const f of def.fields) v[f.key] = f.default;
  return v;
}

/** Restores saved values onto the current definition: unknown keys are dropped, new fields get defaults. */
export function mergeValues(def: ModDef, saved: unknown, paramSetIds: string[]): FormValues {
  const base = initialValues(def);
  if (!saved || typeof saved !== "object") return base;
  const s = saved as Record<string, unknown>;
  if (s.mode === "check" && def.has_check_mode) base.mode = "check";
  if (typeof s.param_set === "string" && paramSetIds.includes(s.param_set)) base.param_set = s.param_set;
  for (const f of def.fields) {
    const x = s[f.key];
    if (typeof x === "number" && Number.isFinite(x)) base[f.key] = x;
    else if (typeof x === "string") base[f.key] = x;
  }
  return base;
}

/** Parses a typed value. Returns null when it is blank or not a finite number. */
export function parseNumber(v: string | number | undefined): number | null {
  if (typeof v === "number") return Number.isFinite(v) ? v : null;
  if (v === undefined) return null;
  const t = v.trim().replace(",", ".");
  if (t === "" || t === "-" || t === ".") return null;
  const n = Number(t);
  return Number.isFinite(n) ? n : null;
}

/** Field-level entry problems (blank or not a number). Engineering validity is left to the engine. */
export function entryErrors(def: ModDef, values: FormValues): Record<string, string> {
  const errs: Record<string, string> = {};
  for (const f of visibleFields(def, values.mode)) {
    if (parseNumber(values[f.key]) === null) errs[f.key] = values[f.key] === "" ? "Required" : "Not a number";
  }
  return errs;
}

/** The engine payload: mode, parameter set and every field as a number. */
export function toPayload(def: ModDef, values: FormValues): Record<string, unknown> {
  const p: Record<string, unknown> = { mode: values.mode, param_set: values.param_set };
  for (const f of def.fields) p[f.key] = parseNumber(values[f.key]) ?? f.default;
  return p;
}

/** Discipline used to group modules in navigation, derived from the module id. */
export function disciplineOf(id: string): string {
  if (id === "combos") return "Actions";
  if (["steelbeam", "steelcol", "bolted", "weld"].includes(id)) return "Structural steel";
  if (["timber", "timbercol"].includes(id)) return "Timber";
  if (["pad", "strip", "raft", "retaining"].includes(id)) return "Foundations & geotechnics";
  return "Reinforced concrete";
}

export const DISCIPLINES = ["Actions", "Reinforced concrete", "Structural steel", "Timber", "Foundations & geotechnics"];

/** Primary standard named in a module subtitle (text before the first "·"). */
export const standardOf = (def: ModDef) => def.subtitle.split("·")[0].trim();
