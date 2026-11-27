// WASM smoke and parity check.
// Loads a wasm-bindgen "web" build in Node, runs every module on its defaults (design and check mode)
// and prints one line per run with the trace hash, so two builds can be diffed for behavioural parity.
// Usage: node scripts/wasm-smoke.mjs [dir]   (default: apps/web/public/wasm)
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const dir = resolve(process.argv[2] ?? "apps/web/public/wasm");
const mod = await import(pathToFileURL(resolve(dir, "strucal_wasm.js")).href);
await mod.default({ module_or_path: await readFile(resolve(dir, "strucal_wasm_bg.wasm")) });

const PS = "EN1992-1-1:2004/RECOMMENDED";
const defs = JSON.parse(mod.modules());
let failures = 0;
console.log(`engine ${mod.engine_version()} · ${defs.length} modules`);
for (const d of defs) {
  for (const mode of d.has_check_mode ? ["design", "check"] : ["design"]) {
    const input = { mode, param_set: PS };
    for (const f of d.fields) input[f.key] = f.default;
    const r = JSON.parse(mod.run_module(d.id, JSON.stringify(input)));
    if (!r.ok) failures++;
    console.log(`${d.id.padEnd(10)} ${mode.padEnd(6)} ${r.ok ? "ok  " : "FAIL"} ${r.trace_hash || r.errors.join("; ")}`);
  }
}
const html = mod.report_html("beam", JSON.stringify({ mode: "design", param_set: PS, b: 300, h: 500, cover: 30, link_dia: 8, bar_dia: 20, fck: 30, fyk: 500, m_ed: 200, v_ed: 120 }));
if (!html.includes("Step-by-step calculation")) { failures++; console.log("report_html: FAIL"); }
const pj = JSON.parse(mod.analyse_project(JSON.stringify({ nodes: [] }), "null"));
if (!pj.ok) { failures++; console.log("analyse_project: FAIL"); }
if (failures) { console.error(`${failures} failure(s)`); process.exit(1); }
