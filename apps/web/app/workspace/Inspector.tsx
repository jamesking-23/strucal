// Parameter inspector: the module's inputs as a property grid (label · value · unit) in collapsible groups.
// Shows which values differ from the module defaults, field-level validation and per-field reset.
"use client";
import { fieldGroups, parseNumber } from "../../lib/form";
import type { FormValues, ModDef } from "../../lib/types";
import { Collapsible } from "../ui/primitives";
import { NumberRow, SelectRow } from "../ui/fields";

type Props = {
  def: ModDef;
  values: FormValues;
  errors: Record<string, string>;
  collapsed: Record<string, boolean>;
  onToggle: (group: string, open: boolean) => void;
  onChange: (patch: Partial<FormValues>) => void;
};

export default function Inspector({ def, values, errors, collapsed, onToggle, onChange }: Props) {
  const groups = fieldGroups(def, values.mode);
  const isModified = (key: string, dflt: number) => { const v = parseNumber(values[key]); return v !== null && Math.abs(v - dflt) > 1e-12; };
  return (
    <div className="insp">
      {groups.map(({ group, fields }) => {
        const nMod = fields.filter((f) => isModified(f.key, f.default)).length;
        const nErr = fields.filter((f) => errors[f.key]).length;
        const drawing = fields.some((f) => f.check_only);
        return (
          <Collapsible key={`${def.id}-${group}`} open={!collapsed[group] || nErr > 0} onToggle={(o) => onToggle(group, o)}
            title={<>{group}{drawing && <span className="insp-tag">drawing</span>}</>}
            meta={nErr > 0 ? <span className="t-fail">{nErr} invalid</span> : nMod > 0 ? <span className="insp-mod">{nMod} changed</span> : `${fields.length}`}>
            <div className="props">
              {fields.map((f) => f.options
                ? <SelectRow key={f.key} id={`in-${f.key}`} label={f.label} value={values[f.key]} defaultValue={f.default} options={f.options} onChange={(v) => onChange({ [f.key]: v })} />
                : <NumberRow key={f.key} id={`in-${f.key}`} label={f.label} unit={f.unit} value={values[f.key]} defaultValue={f.default} error={errors[f.key]}
                    hint={`Default ${f.default}${f.unit ? ` ${f.unit}` : ""}`} onChange={(v) => onChange({ [f.key]: v })} onReset={() => onChange({ [f.key]: f.default })} />)}
            </div>
            {drawing && <p className="insp-note">Values as shown on the drawing. Check Mode verifies them instead of designing.</p>}
          </Collapsible>
        );
      })}
    </div>
  );
}
