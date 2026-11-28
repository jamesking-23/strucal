// Parameter inspector: module identity, mode and design basis, presets, then the module's inputs as tightly
// grouped collapsible panels of UnitInput rows.
"use client";
import { DISCIPLINES, fieldGroups, parseNumber, disciplineOf, standardOf } from "../../lib/form";
import { presetsFor, matchingOption } from "../../lib/presets";
import type { FormValues, ModDef, ParamSet } from "../../lib/types";
import { Segmented, SectionHeader } from "../ui/primitives";
import { UnitInput, SelectInput } from "../ui/UnitInput";

type Props = {
  defs: ModDef[];
  def: ModDef;
  values: FormValues;
  paramSets: ParamSet[];
  errors: Record<string, string>;
  collapsed: Record<string, boolean>;
  onToggle: (group: string) => void;
  onChange: (patch: Partial<FormValues>) => void;
  onModule: (id: string) => void;
};

export default function Inspector({ defs, def, values, paramSets, errors, collapsed, onToggle, onChange, onModule }: Props) {
  const groups = fieldGroups(def, values.mode);
  const presets = presetsFor(def);
  const changedIn = (keys: string[]) => keys.filter((k) => { const f = def.fields.find((x) => x.key === k)!; const v = parseNumber(values[k]); return v !== null && Math.abs(v - f.default) > 1e-12; }).length;
  const ps = paramSets.find((p) => p.id === values.param_set);
  return (
    <div className="insp">
      <div className="insp-id">
        <span className="eyebrow">{disciplineOf(def.id)}</span>
        <label className="sr-only" htmlFor="insp-module">Module</label>
        <select id="insp-module" className="insp-module" value={def.id} onChange={(e) => onModule(e.target.value)}>
          {DISCIPLINES.map((g) => { const ds = defs.filter((d) => disciplineOf(d.id) === g); return ds.length ? <optgroup key={g} label={g}>{ds.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}</optgroup> : null; })}
        </select>
        <span className="insp-std mono">{standardOf(def)}</span>
        <div className="insp-ctl">
          <Segmented label="Mode" block value={values.mode} onChange={(m) => onChange({ mode: m })} options={[
            { value: "design", label: "Design" },
            { value: "check", label: "Check", disabled: !def.has_check_mode, title: def.has_check_mode ? "Verify the member as drawn" : "This module has no Check Mode" },
          ]} />
          <label className="sr-only" htmlFor="insp-basis">Parameter set</label>
          <select id="insp-basis" className="select" value={values.param_set} onChange={(e) => onChange({ param_set: e.target.value })} title={ps?.status}>
            {paramSets.map((p) => <option key={p.id} value={p.id}>{p.label}</option>)}
          </select>
          {ps && /NOT/.test(ps.status) && <span className="insp-flag"><i className="dot warn" /> {ps.status.toLowerCase()}</span>}
        </div>
      </div>

      {presets.length > 0 && (
        <div className="insp-grp">
          <SectionHeader title="Presets" meta="fills inputs" open={!collapsed.__presets} onToggle={() => onToggle("__presets")} id="grp-presets" />
          {!collapsed.__presets && (
            <div id="grp-presets">
              {presets.map((g) => {
                const cur = matchingOption(g, values);
                return (
                  <SelectInput key={g.id} id={`pre-${g.id}`} label={g.title} hint={g.source} value={cur}
                    options={[[-1, cur === -1 ? "Custom values" : "—"], ...g.options.map((o, i) => [i, `${o.label}${o.hint ? `  ·  ${o.hint}` : ""}`] as [number, string])]}
                    onChange={(v) => { const i = Number(v); if (i >= 0) onChange(g.options[i].values); }} />
                );
              })}
            </div>
          )}
        </div>
      )}

      {groups.map(({ group, fields }) => {
        const keys = fields.map((f) => f.key);
        const nErr = keys.filter((k) => errors[k]).length;
        const nCh = changedIn(keys);
        const open = !collapsed[group] || nErr > 0;
        const id = `grp-${def.id}-${group.replace(/\W+/g, "-")}`;
        return (
          <div className="insp-grp" key={id}>
            <SectionHeader title={<>{group}{fields.some((f) => f.check_only) && <span className="insp-tag">as drawn</span>}</>} open={open} onToggle={() => onToggle(group)} id={id}
              meta={nErr > 0 ? <span className="t-fail">{nErr} invalid</span> : nCh > 0 ? <span className="t-signal">{nCh} changed</span> : String(fields.length)} />
            {open && (
              <div id={id}>
                {fields.map((f) => f.options
                  ? <SelectInput key={f.key} id={`in-${f.key}`} label={f.label} value={values[f.key]} defaultValue={f.default} options={f.options} onChange={(v) => onChange({ [f.key]: Number(v) })} />
                  : <UnitInput key={f.key} id={`in-${f.key}`} label={f.label} unit={f.unit === "-" ? "" : f.unit} value={values[f.key]} defaultValue={f.default} error={errors[f.key]}
                      hint={`Default ${f.default}${f.unit && f.unit !== "-" ? ` ${f.unit}` : ""}`} onChange={(v) => onChange({ [f.key]: v })} onReset={() => onChange({ [f.key]: f.default })} />)}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
