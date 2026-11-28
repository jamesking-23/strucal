// Core UI primitives of the workbench design system. Styling lives in app/globals.css; these components fix
// structure, semantics and accessibility so every screen composes the same parts.
"use client";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import type { Status, Verdict } from "../../lib/status";

type BtnVariant = "default" | "primary" | "ghost" | "danger" | "accent";
type BtnSize = "sm" | "md" | "lg";

export function Button({ variant = "default", size = "md", icon, className, children, ...rest }: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: BtnVariant; size?: BtnSize; icon?: boolean }) {
  const cls = ["btn", variant !== "default" && `btn-${variant}`, size !== "md" && `btn-${size}`, icon && "btn-icon", className].filter(Boolean).join(" ");
  return <button type="button" className={cls} {...rest}>{children}</button>;
}

export function Segmented<T extends string>({ label, options, value, onChange, full }: { label: string; options: { value: T; label: string; disabled?: boolean; title?: string }[]; value: T; onChange: (v: T) => void; full?: boolean }) {
  return (
    <div className={`seg${full ? " full" : ""}`} role="group" aria-label={label}>
      {options.map((o) => (
        <button key={o.value} type="button" aria-pressed={o.value === value} disabled={o.disabled} title={o.title} onClick={() => onChange(o.value)}>{o.label}</button>
      ))}
    </div>
  );
}

const VERDICT_TEXT: Record<Verdict, string> = { ok: "Pass", warn: "Pass · review", fail: "Fail", results: "Results", none: "Not calculated" };

export function StatusBadge({ status, children }: { status: Verdict | Status; children?: ReactNode }) {
  const cls = status === "ok" ? "ok" : status === "warn" ? "warn" : status === "fail" ? "fail" : status === "results" ? "info" : "";
  return <span className={`badge ${cls}`}>{children ?? VERDICT_TEXT[status]}</span>;
}

export function StatusDot({ status }: { status: Verdict | Status }) {
  const cls = status === "ok" || status === "warn" || status === "fail" ? status : "";
  return <i className={`dot ${cls}`} aria-hidden />;
}

/** Utilisation bar: 0–100 % fill, a tick at the 0.85 review threshold, colour by status. */
export function UtilBar({ value, status, thin, label }: { value: number; status: Status; thin?: boolean; label?: string }) {
  const pct = Math.max(0, Math.min(100, value * 100));
  return (
    <div className={`util ${status === "ok" ? "" : status}${thin ? " thin" : ""}`} role="meter" aria-valuemin={0} aria-valuemax={1} aria-valuenow={Math.min(value, 9.99)} aria-label={label ?? "Utilisation"}>
      <i style={{ width: `${pct}%` }} />
    </div>
  );
}

/** Collapsible section on native <details>, so it works without JS and with find-in-page. */
export function Collapsible({ title, meta, open, children, id, onToggle }: { title: ReactNode; meta?: ReactNode; open?: boolean; children: ReactNode; id?: string; onToggle?: (open: boolean) => void }) {
  return (
    <details className="coll" open={open} id={id} onToggle={(e) => onToggle?.((e.currentTarget as HTMLDetailsElement).open)}>
      <summary>{title}{meta !== undefined && <span className="meta">{meta}</span>}</summary>
      <div className="coll-b">{children}</div>
    </details>
  );
}

/** Ruled workbench pane with a header strip. */
export function Pane({ title, meta, actions, className, children, label }: { title: ReactNode; meta?: ReactNode; actions?: ReactNode; className?: string; children: ReactNode; label?: string }) {
  return (
    <section className={`pane ${className ?? ""}`} aria-label={label ?? (typeof title === "string" ? title : undefined)}>
      <div className="pane-h">{title}{meta !== undefined && <span className="meta">{meta}</span>}{actions}</div>
      <div className="pane-b">{children}</div>
    </section>
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

export function Notice({ tone, title, children, onDismiss }: { tone: "info" | "warn" | "error" | "ok"; title?: string; children?: ReactNode; onDismiss?: () => void }) {
  return (
    <div className={`alert ${tone}`} role={tone === "error" ? "alert" : "status"}>
      <div className="grow">{title && <b>{title} </b>}{children}</div>
      {onDismiss && <Button size="sm" variant="ghost" onClick={onDismiss}>Dismiss</Button>}
    </div>
  );
}

/** WebAssembly engine start-up: progress while loading, the reason and a retry when it fails. */
export function EngineBoot({ error, onRetry }: { error?: string; onRetry?: () => void }) {
  if (error) {
    return (
      <div className="loading" role="alert">
        <EmptyState glyph="!" title="The calculation engine did not start" action={onRetry && <Button variant="primary" onClick={onRetry}>Try again</Button>}>{error}</EmptyState>
      </div>
    );
  }
  return (
    <div className="loading" role="status" aria-live="polite">
      <div className="boot">
        <span className="eyebrow">Starting calculation engine</span>
        <div className="bar" />
        <div className="steps"><span>strucal_wasm_bg.wasm · deterministic Rust engine</span><span>module definitions · parameter sets · formula registry</span></div>
      </div>
    </div>
  );
}

export function Kbd({ children }: { children: ReactNode }) {
  return <kbd>{children}</kbd>;
}
