// Input presets: standard values that fill a module's existing inputs in one step. They never change the
// engine or its formulas; the engineer can edit every value afterwards. Each preset group is offered only when
// the module has all of the input keys it sets.
import type { FormValues, ModDef } from "./types";

export type PresetOption = { label: string; hint?: string; values: Record<string, number> };
export type PresetGroup = { id: string; title: string; source: string; options: PresetOption[] };

const CONCRETE: [string, number][] = [["C20/25", 20], ["C25/30", 25], ["C28/35", 28], ["C30/37", 30], ["C32/40", 32], ["C35/45", 35], ["C40/50", 40], ["C45/55", 45], ["C50/60", 50]];

// EN 1993-1-1 Table 3.1, nominal values for t ≤ 40 mm.
const STEEL: [string, number, number][] = [["S235", 235, 360], ["S275", 275, 430], ["S355", 355, 490]];

// Hot-rolled sections, EN 10365 dimensions: h, b, tw, tf, r (mm).
const SECTIONS: [string, number, number, number, number, number][] = [
  ["IPE 160", 160, 82, 5.0, 7.4, 9], ["IPE 180", 180, 91, 5.3, 8.0, 9], ["IPE 200", 200, 100, 5.6, 8.5, 12], ["IPE 220", 220, 110, 5.9, 9.2, 12],
  ["IPE 240", 240, 120, 6.2, 9.8, 15], ["IPE 270", 270, 135, 6.6, 10.2, 15], ["IPE 300", 300, 150, 7.1, 10.7, 15], ["IPE 330", 330, 160, 7.5, 11.5, 18],
  ["IPE 360", 360, 170, 8.0, 12.7, 18], ["IPE 400", 400, 180, 8.6, 13.5, 21], ["IPE 450", 450, 190, 9.4, 14.6, 21], ["IPE 500", 500, 200, 10.2, 16.0, 21],
  ["HEB 100", 100, 100, 6.0, 10, 12], ["HEB 120", 120, 120, 6.5, 11, 12], ["HEB 140", 140, 140, 7.0, 12, 12], ["HEB 160", 160, 160, 8.0, 13, 15],
  ["HEB 180", 180, 180, 8.5, 14, 15], ["HEB 200", 200, 200, 9.0, 15, 18], ["HEB 220", 220, 220, 9.5, 16, 18], ["HEB 240", 240, 240, 10.0, 17, 21],
  ["HEB 260", 260, 260, 10.0, 17.5, 24], ["HEB 280", 280, 280, 10.5, 18, 24], ["HEB 300", 300, 300, 11.0, 19, 27],
];

// EN 1990 Table A1.1 recommended ψ factors for buildings.
const PSI: [string, number, number, number][] = [
  ["Cat. A — domestic, residential", 0.7, 0.5, 0.3], ["Cat. B — offices", 0.7, 0.5, 0.3], ["Cat. C — congregation areas", 0.7, 0.7, 0.6],
  ["Cat. D — shopping areas", 0.7, 0.7, 0.6], ["Cat. E — storage areas", 1.0, 0.9, 0.8], ["Cat. F — traffic, vehicle ≤ 30 kN", 0.7, 0.7, 0.6],
  ["Cat. G — traffic, 30–160 kN", 0.7, 0.5, 0.3], ["Cat. H — roofs", 0, 0, 0], ["Snow (site altitude ≤ 1000 m)", 0.5, 0.2, 0],
  ["Wind", 0.6, 0.2, 0], ["Temperature (non-fire)", 0.6, 0.5, 0],
];

function has(def: ModDef, keys: string[]) {
  const set = new Set(def.fields.map((f) => f.key));
  return keys.every((k) => set.has(k));
}

/** Preset groups applicable to a module. */
export function presetsFor(def: ModDef): PresetGroup[] {
  const out: PresetGroup[] = [];
  if (has(def, ["h", "b", "tw", "tf", "r"])) {
    out.push({ id: "section", title: "Rolled section", source: "EN 10365", options: SECTIONS.map(([n, h, b, tw, tf, r]) => ({ label: n, hint: `${h}×${b}`, values: { h, b, tw, tf, r } })) });
  }
  if (has(def, ["fck"])) {
    out.push({ id: "concrete", title: "Concrete class", source: "EN 1992-1-1 Table 3.1", options: CONCRETE.map(([n, fck]) => ({ label: n, hint: `fck ${fck}`, values: { fck } })) });
  }
  if (has(def, ["fy", "fu"])) {
    out.push({ id: "steel", title: "Steel grade", source: "EN 1993-1-1 Table 3.1 (t ≤ 40 mm)", options: STEEL.map(([n, fy, fu]) => ({ label: n, hint: `fy ${fy} · fu ${fu}`, values: { fy, fu } })) });
  } else if (has(def, ["fy"])) {
    out.push({ id: "steel", title: "Steel grade", source: "EN 1993-1-1 Table 3.1 (t ≤ 40 mm)", options: STEEL.map(([n, fy]) => ({ label: n, hint: `fy ${fy}`, values: { fy } })) });
  }
  for (const i of [1, 2, 3]) {
    if (has(def, [`p0${i}`, `p1${i}`, `p2${i}`])) {
      out.push({ id: `psi${i}`, title: `ψ factors · variable action ${i}`, source: "EN 1990 Table A1.1", options: PSI.map(([n, a, b, c]) => ({ label: n, hint: `${a} · ${b} · ${c}`, values: { [`p0${i}`]: a, [`p1${i}`]: b, [`p2${i}`]: c } })) });
    }
  }
  return out;
}

/** Index of the option whose values all match the current inputs, or -1. */
export function matchingOption(group: PresetGroup, values: FormValues): number {
  return group.options.findIndex((o) => Object.entries(o.values).every(([k, v]) => Number(values[k]) === v));
}
