// Property-grid fields for engineering parameters: label · value · unit, with validation feedback,
// a mark for values changed from the module default and a per-field reset.
"use client";
import { parseNumber } from "../../lib/form";

type Common = { id: string; label: string; hint?: string };

/** Splits a label like "Width b" or "Moment M Ed" into words and a trailing symbol for typographic emphasis. */
function LabelText({ text }: { text: string }) {
  // A symbol is a short token (≤ 3 chars) optionally followed by a short subscript token, e.g. "b", "M Ed", "Q k,1", "φ'".
  const m = text.match(/^(.*?)(\s)([A-Za-zφγδαβψξλμρτ][A-Za-z0-9,'′]{0,2}(?: [A-Za-z0-9,'′]{1,4})?)$/);
  if (!m || m[1].length < 3) return <>{text}</>;
  return <>{m[1]} <span className="sym">{m[3]}</span></>;
}

export function NumberRow({ id, label, hint, unit, value, defaultValue, error, onChange, onReset }: Common & { unit: string; value: string | number; defaultValue: number; error?: string; onChange: (v: string) => void; onReset: () => void }) {
  const parsed = parseNumber(value);
  const modified = parsed !== null && Math.abs(parsed - defaultValue) > 1e-12;
  return (
    <div className={`prop${error ? " invalid" : modified ? " modified" : ""}`} title={hint}>
      <label htmlFor={id}><LabelText text={label} /></label>
      <input id={id} className="val" type="text" inputMode="decimal" autoComplete="off" spellCheck={false} value={String(value)}
        aria-invalid={!!error} aria-describedby={error ? `${id}-msg` : undefined}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => { if (e.key === "Escape" && modified) { e.preventDefault(); onReset(); } }} />
      <span className="u">{unit}</span>
      {modified && !error && <button type="button" className="reset" title={`Reset to default (${defaultValue} ${unit})`} aria-label={`Reset ${label} to ${defaultValue}`} onClick={onReset}>↺</button>}
      {error && <span className="msg" id={`${id}-msg`}>{error}</span>}
    </div>
  );
}

export function SelectRow({ id, label, hint, value, defaultValue, options, onChange }: Common & { value: string | number; defaultValue: number; options: [number, string][]; onChange: (v: number) => void }) {
  const modified = Number(value) !== defaultValue;
  return (
    <div className={`prop wide${modified ? " modified" : ""}`} title={hint}>
      <label htmlFor={id}>{label}</label>
      <select id={id} className="val" value={String(value)} onChange={(e) => onChange(Number(e.target.value))}>
        {options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
      </select>
    </div>
  );
}
