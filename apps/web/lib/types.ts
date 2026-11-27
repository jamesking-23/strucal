// Types mirroring the JSON contracts of crates/engine (module.rs, project.rs, params.rs, registry.rs).
// The engine is the source of truth; these types only describe what it serialises.

export type Mode = "design" | "check";

/** A numeric input field. `options` turns it into a choice list of [value, label] pairs. */
export interface Field {
  key: string;
  label: string;
  unit: string;
  group: string;
  default: number;
  check_only: boolean;
  options?: [number, string][];
}

export interface ModDef {
  id: string;
  name: string;
  title: string;
  subtitle: string;
  available: boolean;
  has_check_mode: boolean;
  fields: Field[];
}

export interface Upcoming { name: string; note: string }

export interface Output { key: string; label: string; value: number; unit: string }
export interface KV { label: string; value: string }
export interface Check { name: string; reference: string; demand: string; capacity: string; utilisation: number; pass: boolean }
export type ObsLevel = "RED" | "AMBER" | "INFO";
export interface Obs { id: string; level: ObsLevel | string; text: string }

export interface Step {
  section: string;
  title: string;
  reference: string;
  formula: string;
  substitution: string;
  result: string;
  comment: string;
  /** Plain-text renderings (no sub/superscript markup). */
  calculation: string;
  output: string;
}

/** Section drawing. The engine tags variants with their Rust names ("Beam", "ISection", ...). */
export type Drawing =
  | { kind: "Beam"; b: number; h: number; cover: number; link_dia: number; bar_dia: number; n_bars: number }
  | { kind: "Slab"; h: number; cover: number; bar_dia: number; spacing: number }
  | { kind: "Pad"; bx: number; by: number; cx: number; cy: number; h: number }
  | { kind: "Bolts"; w: number; n1: number; n2: number; e1: number; p1: number; e2: number; p2: number; d: number }
  | { kind: "Weld"; lw: number; a: number }
  | { kind: "Raft"; l: number; w: number; n: number; spacing: number; edge: number }
  | { kind: "Retain"; hs: number; tb: number; ts: number; lt: number; b: number }
  | { kind: "ISection"; h: number; b: number; tw: number; tf: number }
  | { kind: "Rect"; b: number; h: number }
  | { kind: "Wall"; h: number; cover: number; bar_dia: number; spacing: number }
  | { kind: "Strip"; width: number; t: number; h: number }
  | { kind: "Column"; b: number; h: number; cover: number; link_dia: number; bar_dia: number; n_face: number };

export interface ModResult {
  outputs: Output[];
  ok: boolean;
  errors: string[];
  warnings: string[];
  param_status: string;
  summary: KV[];
  checks: Check[];
  max_util: number;
  overall_pass: boolean;
  observations: Obs[];
  trace: { steps: Step[] };
  trace_hash: string;
  engine_version: string;
  scope_notes: string[];
  drawing: Drawing | null;
}

export interface ParamSet {
  id: string;
  label: string;
  gamma_c: number;
  gamma_s: number;
  alpha_cc: number;
  alpha_cw: number;
  gamma_g: number;
  gamma_q: number;
  gamma_m0: number;
  gamma_m1: number;
  status: string;
  source: string;
}

export interface Formula {
  id: string;
  name: string;
  standard: string;
  clause: string;
  module_version: string;
  verification: string;
}

/** Raw form values: strings while the user types, numbers once parsed. */
export type FormValues = Record<string, string | number> & { mode: Mode; param_set: string };

// ---------- project dependency engine ----------

export interface Link { field: string; from: string; output: string; factor: number }
export interface ProjectNode {
  id: string;
  name: string;
  module: string;
  mode: Mode;
  param_set: string;
  values: Record<string, number>;
  links: Link[];
}
export interface Resolved { field: string; from: string; output: string; value: number }
export type NodeStatus = "calculated" | "blocked" | "error";
export interface NodeResult {
  id: string;
  name: string;
  module: string;
  status: NodeStatus;
  errors: string[];
  overall_pass: boolean;
  has_verdict: boolean;
  max_util: number;
  trace_hash: string;
  own_hash: string;
  outputs: Output[];
  resolved: Resolved[];
}
export interface Diff { node: string; output: string; old: number; new: number }
export interface Impact {
  has_previous: boolean;
  edited: string[];
  affected: string[];
  newly_failing: string[];
  newly_passing: string[];
  added: string[];
  removed: string[];
  diffs: Diff[];
  review_required: boolean;
  summary: string;
}
export interface ProjectResult {
  ok: boolean;
  errors: string[];
  order: string[];
  nodes: NodeResult[];
  overall_pass: boolean;
  impact: Impact;
  engine_version: string;
}
export interface Revision {
  id: string;
  name: string;
  savedAt: string;
  project: ProjectNode[];
  results: NodeResult[];
  status?: string;
}
