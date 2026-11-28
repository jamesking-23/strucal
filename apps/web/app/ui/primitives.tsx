// Core UI primitives. Styling is in app/globals.css; these components fix structure, semantics and accessibility.
"use client";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import type { Status, Verdict } from "../../lib/status";
import { VERDICT_WORD } from "../../lib/status";

type BtnVariant = "default" | "primary" | "ghost" | "danger";

export function Button({ variant = "default", size, icon, className, children, ...rest }: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: BtnVariant; size?: "sm" | "lg"; icon?: boolean }) {
  const cls = ["btn", variant !== "default" && variant, size, icon && "icon", className].filter(Boolean).join(" ");
  return <button type="button" className={cls} {...rest}>{children}</button>;
}

export function Segmented<T extends string>({ label, options, value, onChange, block, xs }: { label: string; options: { value: T; label: string; disabled?: boolean; title?: string }[]; value: T; onChange: (v: T) => void; block?: boolean; xs?: boolean }) {
  return (
    <div className={`seg${block ? " block" : ""}${xs ? " xs" : ""}`} role="group" aria-label={label}>
      {options.map((o) => <button key={o.value} type="button" aria-pressed={o.value === value} disabled={o.disabled} title={o.title} onClick={() => onChange(o.value)}>{o.label}</button>)}
    </div>
  );
}

const BADGE_CLASS: Record<Verdict, string> = { pass: "pass", warn: "warn", fail: "fail", results: "info", none: "" };

/** Status badge. Without children it shows the standard word for the status. */
export function StatusBadge({ status, children }: { status: Verdict | Status; children?: ReactNode }) {
  const word = status === "warn" ? "≥ 0.85" : VERDICT_WORD[status as Verdict];
  return <span className={`badge ${BADGE_CLASS[status as Verdict]}`}>{children ?? word}</span>;
}

export function StatusDot({ status }: { status: Verdict | Status }) {
  return <i className={`dot ${BADGE_CLASS[status as Verdict] === "info" ? "" : BADGE_CLASS[status as Verdict]}`} aria-hidden />;
}

/** Horizontal utilisation bar, 0–100 %, marks at 0.85 (review) and 1.00 (limit). */
export function UtilizationBar({ value, status, thin, label }: { value: number; status: Status; thin?: boolean; label: string }) {
  const pct = Math.max(0, Math.min(100, value * 100));
  return (
    <div className={`ubar${status === "pass" ? "" : ` ${status}`}${thin ? " thin" : ""}`} role="meter" aria-label={label} aria-valuemin={0} aria-valuemax={1} aria-valuenow={Math.min(value, 9.99)}>
      <i style={{ width: `${pct}%` }} />
    </div>
  );
}

/** Section header. With `onToggle` it becomes a disclosure button. */
export function SectionHeader({ title, meta, open, onToggle, id }: { title: ReactNode; meta?: ReactNode; open?: boolean; onToggle?: () => void; id?: string }) {
  if (onToggle) {
    return (
      <button type="button" className="sh" aria-expanded={open} aria-controls={id} onClick={onToggle}>
        <span className="chev" aria-hidden />{title}{meta !== undefined && <span className="meta">{meta}</span>}
      </button>
    );
  }
  return <div className="sh">{title}{meta !== undefined && <span className="meta">{meta}</span>}</div>;
}

/** Workbench pane: a ruled region with a header strip and a scrolling body. */
export function WorkbenchPane({ title, meta, tools, className, children, label }: { title: ReactNode; meta?: ReactNode; tools?: ReactNode; className?: string; children: ReactNode; label?: string }) {
  return (
    <section className={`pane ${className ?? ""}`} aria-label={label ?? (typeof title === "string" ? title : undefined)}>
      <div className="pane-h">{title}{meta !== undefined && <span className="meta">{meta}</span>}{tools}</div>
      <div className="pane-b">{children}</div>
    </section>
  );
}

export function Tabs<T extends string>({ label, tabs, value, onChange, tools }: { label: string; tabs: { value: T; label: ReactNode }[]; value: T; onChange: (v: T) => void; tools?: ReactNode }) {
  return (
    <div className="tabs" role="tablist" aria-label={label}>
      {tabs.map((t) => <button key={t.value} type="button" role="tab" aria-selected={t.value === value} onClick={() => onChange(t.value)}>{t.label}</button>)}
      <span className="fill" />
      {tools && <span className="tools">{tools}</span>}
    </div>
  );
}

export function EmptyState({ glyph = "∅", title, children, action }: { glyph?: string; title: string; children?: ReactNode; action?: ReactNode }) {
  return (
    <div className="empty">
      <span className="glyph" aria-hidden>{glyph}</span>
      <h2>{title}</h2>
      {children && <p>{children}</p>}
      {action}
    </div>
  );
}

export function Notice({ tone, title, children, onDismiss, action }: { tone: "info" | "warn" | "error" | "ok"; title?: string; children?: ReactNode; onDismiss?: () => void; action?: ReactNode }) {
  return (
    <div className={`notice ${tone}`} role={tone === "error" ? "alert" : "status"}>
      <div className="grow">{title && <b>{title} </b>}{children}</div>
      {action}
      {onDismiss && <Button size="sm" variant="ghost" onClick={onDismiss}>Dismiss</Button>}
    </div>
  );
}

/** WebAssembly engine start-up: progress while loading; the reason and a retry if it fails. */
export function EngineBoot({ error, onRetry }: { error?: string; onRetry?: () => void }) {
  if (error) {
    return <div className="boot" role="alert"><EmptyState glyph="!" title="The calculation engine did not start" action={onRetry && <Button variant="primary" onClick={onRetry}>Retry</Button>}>{error}</EmptyState></div>;
  }
  return (
    <div className="boot" role="status" aria-live="polite">
      <div className="boot-card">
        <span className="eyebrow">Initialising engine</span>
        <div className="bar" />
        <div className="log"><span>› strucal_wasm_bg.wasm — deterministic Rust engine</span><span>› module definitions · parameter sets · formula registry</span></div>
      </div>
    </div>
  );
}

export function Spinner() {
  return <span className="spinner" role="status" aria-label="Working" />;
}
