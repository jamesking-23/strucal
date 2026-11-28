// Typed access to the WebAssembly build of crates/engine (served from /public/wasm).
// All calculation happens inside the engine; this module only loads it, parses its JSON and contains failures.
import type { Formula, ModDef, ModResult, ParamSet, ProjectNode, ProjectResult, NodeResult, Upcoming } from "./types";

interface RawEngine {
  default: (init: { module_or_path: string }) => Promise<unknown>;
  modules(): string;
  upcoming(): string;
  run_module(id: string, input: string): string;
  report_html(id: string, input: string): string;
  param_sets(): string;
  formula_registry(): string;
  engine_version(): string;
  analyse_project(project: string, previous: string): string;
}

export interface Engine {
  version: string;
  defs: ModDef[];
  upcoming: Upcoming[];
  paramSets: ParamSet[];
  formulas: Formula[];
  /** Runs a module. Never throws: failures come back as `ok: false` with errors. */
  run(id: string, input: Record<string, unknown>): ModResult;
  /** Full calculation sheet as a standalone HTML document. */
  report(id: string, input: Record<string, unknown>): string;
  /** Project dependency analysis. Never throws. */
  analyse(nodes: ProjectNode[], baseline: NodeResult[] | null): ProjectResult;
}

export const DEFAULT_PARAM_SET = "EN1992-1-1:2004/RECOMMENDED";
// The engine files are fetched at run time, so they need the site's base path when it is served from a sub-path.
const BASE = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
const WASM_JS = `${BASE}/wasm/strucal_wasm.js`;
const WASM_BIN = `${BASE}/wasm/strucal_wasm_bg.wasm`;
const CRASH = "The calculation engine stopped unexpectedly. Reload the page to restart it; your inputs are kept.";

export function failedResult(errors: string[], version = ""): ModResult {
  return { outputs: [], ok: false, errors, warnings: [], param_status: "", summary: [], checks: [], max_util: 0, overall_pass: false, observations: [], trace: { steps: [] }, trace_hash: "", engine_version: version, scope_notes: [], drawing: null };
}

function failedProject(errors: string[], version: string): ProjectResult {
  return { ok: false, errors, order: [], nodes: [], overall_pass: false, engine_version: version, impact: { has_previous: false, edited: [], affected: [], newly_failing: [], newly_passing: [], added: [], removed: [], diffs: [], review_required: false, summary: "" } };
}

const message = (e: unknown) => (e instanceof Error ? e.message : String(e));

function wrap(raw: RawEngine): Engine {
  const version = raw.engine_version();
  // A WebAssembly trap leaves the instance in an undefined state; refuse further calls until reload.
  let crashed = false;
  const guard = <T>(fallback: (msg: string) => T, call: () => T): T => {
    if (crashed) return fallback(CRASH);
    try { return call(); }
    catch (e) {
      if (e instanceof WebAssembly.RuntimeError) { crashed = true; return fallback(CRASH); }
      return fallback(`Engine error: ${message(e)}`);
    }
  };
  return {
    version,
    defs: JSON.parse(raw.modules()) as ModDef[],
    upcoming: JSON.parse(raw.upcoming()) as Upcoming[],
    paramSets: JSON.parse(raw.param_sets()) as ParamSet[],
    formulas: JSON.parse(raw.formula_registry()) as Formula[],
    run: (id, input) => guard((m) => failedResult([m], version), () => JSON.parse(raw.run_module(id, JSON.stringify(input))) as ModResult),
    report: (id, input) => guard((m) => `<!DOCTYPE html><html><body><p>${m}</p></body></html>`, () => raw.report_html(id, JSON.stringify(input))),
    analyse: (nodes, baseline) => guard((m) => failedProject([m], version), () => JSON.parse(raw.analyse_project(JSON.stringify({ nodes }), baseline ? JSON.stringify(baseline) : "null")) as ProjectResult),
  };
}

let ready: Promise<Engine> | null = null;

/** Loads the engine once per page. A failed load is not cached, so calling again retries. */
export function loadEngine(): Promise<Engine> {
  if (!ready) {
    ready = (async () => {
      if (typeof WebAssembly === "undefined") throw new Error("This browser does not support WebAssembly, which the calculation engine needs.");
      const mod = (await import(/* webpackIgnore: true */ WASM_JS)) as RawEngine;
      await mod.default({ module_or_path: WASM_BIN });
      return wrap(mod);
    })().catch((e) => {
      ready = null;
      throw new Error(`Could not load the calculation engine: ${message(e)}`);
    });
  }
  return ready;
}
