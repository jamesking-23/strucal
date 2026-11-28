// Engineering parameter inputs: label on the left, value with an inline right-aligned unit on the right.
// Validation is instant (blank / not a number), values changed from the module default are marked, and Escape
// or the corner button restores the default.
"use client";
import { parseNumber } from "../../lib/form";

/** Splits "Width b" / "Moment M Ed" into words and a trailing symbol set in italics. */
function LabelText({ text }: { text: string }) {
  const m = text.match(/^(.*?)(\s)([A-Za-zφγδαβψξλμρτ][A-Za-z0-9,'′]{0,2}(?: [A-Za-z0-9,'′]{1,4})?)$/);
  if (!m || m[1].length < 3) return <>{text}</>;
  return <>{m[1]} <span className="sym">{m[3]}</span></>;
}

type Common = { id: string; label: string; hint?: string };

export function UnitInput({ id, label, hint, unit, value, defaultValue, error, readOnly, onChange, onReset }: Common & {
  unit: string; value: string | number; defaultValue?: number; error?: string; readOnly?: boolean; onChange?: (v: string) => void; onReset?: () => void;
}) {
  const parsed = parseNumber(value);
  const changed = !readOnly && defaultValue !== undefined && parsed !== null && Math.abs(parsed - defaultValue) > 1e-12;
  return (
    <div className={`ui-row${error ? " invalid" : changed ? " changed" : ""}${readOnly ? " linked" : ""}`} title={hint}>
      <label htmlFor={id}><LabelText text={label} /></label>
      <div className="ui-box">
        <input id={id} type="text" inputMode="decimal" autoComplete="off" spellCheck={false} value={String(value)} readOnly={readOnly}
          aria-invalid={!!error} aria-describedby={error ? `${id}-msg` : undefined}
          onChange={(e) => onChange?.(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Escape" && changed && onReset) { e.preventDefault(); onReset(); } }} />
        {unit && <span className="u">{unit}</span>}
      </div>
      {changed && onReset && <button type="button" className="reset" title={`Reset to ${defaultValue} ${unit}`} aria-label={`Reset ${label} to ${defaultValue}`} onClick={onReset}>↺</button>}
      {error && <span className="msg" id={`${id}-msg`}>{error}</span>}
    </div>
  );
}

export function SelectInput({ id, label, hint, value, defaultValue, options, onChange }: Common & { value: string | number; defaultValue?: number; options: [number | string, string][]; onChange: (v: string) => void }) {
  const changed = defaultValue !== undefined && Number(value) !== defaultValue;
  return (
    <div className={`ui-row wide${changed ? " changed" : ""}`} title={hint}>
      <label htmlFor={id}>{label}</label>
      <div className="ui-box">
        <select id={id} value={String(value)} onChange={(e) => onChange(e.target.value)}>
          {options.map(([v, l]) => <option key={String(v)} value={String(v)}>{l}</option>)}
        </select>
      </div>
    </div>
  );
}
