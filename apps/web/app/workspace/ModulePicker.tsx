// Module picker: a searchable list grouped by discipline, fully keyboard operable (↑ ↓ Enter Esc, / to focus).
"use client";
import { useEffect, useMemo, useRef, useState } from "react";
import { DISCIPLINES, disciplineOf, standardOf } from "../../lib/form";
import type { ModDef, Upcoming } from "../../lib/types";

export default function ModulePicker({ defs, upcoming, value, onChange }: { defs: ModDef[]; upcoming: Upcoming[]; value: string; onChange: (id: string) => void }) {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const [hi, setHi] = useState(0);
  const root = useRef<HTMLDivElement>(null);
  const search = useRef<HTMLInputElement>(null);
  const current = defs.find((d) => d.id === value);

  const list = useMemo(() => {
    const t = q.trim().toLowerCase();
    const all = DISCIPLINES.flatMap((g) => defs.filter((d) => disciplineOf(d.id) === g).map((d) => ({ d, g })));
    if (!t) return all;
    // Rank: name starts with query, name contains it, title contains it, then standard/subtitle or id.
    const score = (d: ModDef) => { const n = d.name.toLowerCase(); return n.startsWith(t) ? 0 : n.includes(t) ? 1 : d.title.toLowerCase().includes(t) ? 2 : [d.subtitle, d.id].some((x) => x.toLowerCase().includes(t)) ? 3 : 9; };
    return all.map((x, i) => ({ ...x, s: score(x.d), i })).filter((x) => x.s < 9).sort((a, b) => a.s - b.s || a.i - b.i).map(({ d, g }) => ({ d, g }));
  }, [defs, q]);

  useEffect(() => { if (open) setHi(Math.max(0, list.findIndex((x) => x.d.id === value))); }, [open]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => { setHi(0); }, [q]);
  useEffect(() => {
    if (!open) return;
    const down = (e: MouseEvent) => { if (root.current && !root.current.contains(e.target as Node)) setOpen(false); };
    document.addEventListener("mousedown", down);
    return () => document.removeEventListener("mousedown", down);
  }, [open]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      if (e.key === "/" && !open && !["INPUT", "SELECT", "TEXTAREA"].includes(t.tagName)) { e.preventDefault(); setOpen(true); }
    };
    document.addEventListener("keydown", key);
    return () => document.removeEventListener("keydown", key);
  }, [open]);

  const choose = (id: string) => { onChange(id); setOpen(false); setQ(""); };
  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") { e.preventDefault(); setHi((h) => Math.min(list.length - 1, h + 1)); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setHi((h) => Math.max(0, h - 1)); }
    else if (e.key === "Enter" && list[hi]) { e.preventDefault(); choose(list[hi].d.id); }
    else if (e.key === "Escape") { e.preventDefault(); setOpen(false); }
  };

  let lastGroup = "";
  return (
    <div className="mp" ref={root}>
      <button type="button" className="mp-trigger" aria-haspopup="listbox" aria-expanded={open} onClick={() => setOpen(!open)} title="Choose module ( / )">
        <span className="mp-disc">{current ? disciplineOf(current.id) : "Module"}</span>
        <span className="mp-name">{current?.name ?? "Choose a module"}</span>
        <span className="mp-std">{current ? standardOf(current) : ""}</span>
        <span className="mp-caret" aria-hidden />
      </button>
      {open && (
        <div className="mp-pop" role="dialog" aria-label="Choose module">
          <input ref={search} autoFocus className="input" placeholder="Search modules" aria-label="Search modules" value={q} onChange={(e) => setQ(e.target.value)} onKeyDown={onKey} aria-controls="mp-list" aria-activedescendant={list[hi] ? `mp-${list[hi].d.id}` : undefined} />
          <ul className="mp-list" id="mp-list" role="listbox" aria-label="Modules">
            {list.length === 0 && <li className="mp-none">No module matches “{q}”.</li>}
            {list.map(({ d, g }, i) => {
              const head = g !== lastGroup; lastGroup = g;
              return (
                <li key={d.id} role="presentation">
                  {head && <div className="mp-group" role="presentation">{g}</div>}
                  <div id={`mp-${d.id}`} role="option" aria-selected={d.id === value} className={`mp-opt${i === hi ? " hi" : ""}`} onMouseEnter={() => setHi(i)} onClick={() => choose(d.id)}>
                    <span className="n">{d.name}</span><span className="s">{standardOf(d)}</span>
                    {d.has_check_mode && <span className="c" title="Has Check Mode">CHK</span>}
                  </div>
                </li>
              );
            })}
            {!q && upcoming.length > 0 && <li role="presentation"><div className="mp-group">Planned</div>{upcoming.map((u) => <div key={u.name} className="mp-opt dis" aria-disabled><span className="n">{u.name}</span><span className="s">{u.note}</span></div>)}</li>}
          </ul>
          <div className="mp-foot"><kbd>↑</kbd><kbd>↓</kbd> move <kbd>↵</kbd> open <kbd>esc</kbd> close</div>
        </div>
      )}
    </div>
  );
}
