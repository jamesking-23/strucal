// Command palette (⌘K / Ctrl+K): jump to any module, page or action from the keyboard.
"use client";
import { useEffect, useMemo, useRef, useState } from "react";

export type Command = { id: string; group: string; title: string; hint?: string; keywords?: string; run: () => void };

export default function CommandPalette({ open, onClose, commands }: { open: boolean; onClose: () => void; commands: Command[] }) {
  const [q, setQ] = useState("");
  const [hi, setHi] = useState(0);
  const list = useRef<HTMLUListElement>(null);

  const items = useMemo(() => {
    const t = q.trim().toLowerCase();
    if (!t) return commands;
    const score = (c: Command) => {
      const title = c.title.toLowerCase();
      if (title.startsWith(t)) return 0;
      if (title.includes(t)) return 1;
      if ((c.keywords ?? "").toLowerCase().includes(t) || (c.hint ?? "").toLowerCase().includes(t)) return 2;
      return 9;
    };
    return commands.map((c, i) => ({ c, s: score(c), i })).filter((x) => x.s < 9).sort((a, b) => a.s - b.s || a.i - b.i).map((x) => x.c);
  }, [commands, q]);

  useEffect(() => { if (open) { setQ(""); setHi(0); } }, [open]);
  useEffect(() => { setHi(0); }, [q]);
  useEffect(() => { list.current?.querySelector(".hi")?.scrollIntoView({ block: "nearest" }); }, [hi]);
  if (!open) return null;

  const run = (c: Command) => { onClose(); c.run(); };
  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") { e.preventDefault(); setHi((h) => Math.min(items.length - 1, h + 1)); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setHi((h) => Math.max(0, h - 1)); }
    else if (e.key === "Enter" && items[hi]) { e.preventDefault(); run(items[hi]); }
    else if (e.key === "Escape") { e.preventDefault(); onClose(); }
  };
  let last = "";
  return (<>
    <div className="backdrop" onClick={onClose} aria-hidden />
    <div className="palette" role="dialog" aria-modal="true" aria-label="Command palette">
      <input autoFocus placeholder="Type a module, page or action…" aria-label="Search commands" value={q} onChange={(e) => setQ(e.target.value)} onKeyDown={onKey}
        role="combobox" aria-expanded="true" aria-controls="pal-list" aria-activedescendant={items[hi] ? `pal-${items[hi].id}` : undefined} />
      <ul ref={list} id="pal-list" role="listbox">
        {items.length === 0 && <li className="none">Nothing matches “{q}”.</li>}
        {items.map((c, i) => {
          const head = c.group !== last; last = c.group;
          return (
            <li key={c.id} role="presentation">
              {head && <div className="grp">{c.group}</div>}
              <div id={`pal-${c.id}`} role="option" aria-selected={i === hi} className={`opt${i === hi ? " hi" : ""}`} onMouseEnter={() => setHi(i)} onClick={() => run(c)}>
                <span className="t">{c.title}</span>{c.hint && <span className="h">{c.hint}</span>}
              </div>
            </li>
          );
        })}
      </ul>
      <div className="foot"><span><kbd>↑</kbd> <kbd>↓</kbd> select</span><span><kbd>↵</kbd> run</span><span><kbd>esc</kbd> close</span></div>
    </div>
  </>);
}
