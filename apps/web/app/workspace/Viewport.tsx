// Technical viewport: Section and Loading drawings on a drafting grid. Drag to pan, wheel or buttons to zoom,
// Fit to reset; the grid moves with the drawing.
"use client";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Drawing } from "../../lib/types";
import SectionDrawing from "./SectionDrawing";
import LoadingDiagram from "./LoadingDiagram";
import { Button, Tabs } from "../ui/primitives";
import { IconFit, IconMinus, IconPlus } from "../ui/icons";

type View = "section" | "loading";

export default function Viewport({ moduleId, drawing, values, state }: { moduleId: string; drawing: Drawing | null; values: Record<string, unknown>; state: "ready" | "invalid" }) {
  const [view, setView] = useState<View>("section");
  const [dims, setDims] = useState(true);
  const [z, setZ] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const canvas = useRef<HTMLDivElement>(null);
  const drag = useRef<{ x: number; y: number; px: number; py: number } | null>(null);

  const fit = useCallback(() => { setZ(1); setPan({ x: 0, y: 0 }); }, []);
  useEffect(() => { fit(); }, [moduleId, view, fit]);
  useEffect(() => { if (moduleId === "combos" || moduleId === "beam") setView((v) => (moduleId === "combos" ? "loading" : v)); }, [moduleId]);

  // Wheel zoom needs a non-passive listener to stop the page from scrolling.
  useEffect(() => {
    const el = canvas.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => { e.preventDefault(); setZ((v) => Math.min(4, Math.max(0.4, v * (e.deltaY < 0 ? 1.1 : 1 / 1.1)))); };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  const hasSection = state === "ready" && !!drawing;
  const content = view === "section"
    ? (hasSection ? <SectionDrawing d={drawing} dims={dims} /> : null)
    : <LoadingDiagram id={moduleId} values={values} />;
  const empty = view === "section"
    ? (state === "invalid" ? "Geometry appears when every input is valid." : "This module works on action effects and has no cross-section. See Loading.")
    : "";

  return (
    <section className="vp" aria-label="Viewport">
      <Tabs label="Viewport" value={view} onChange={setView} tabs={[{ value: "section", label: "Section" }, { value: "loading", label: "Loading" }]} tools={<>
        <Button size="sm" aria-pressed={dims} onClick={() => setDims(!dims)} disabled={view !== "section" || !hasSection} title="Dimensions">Dims</Button>
        <Button size="sm" icon onClick={() => setZ((v) => Math.max(0.4, v / 1.2))} aria-label="Zoom out"><IconMinus width={14} height={14} /></Button>
        <span className="vp-z mono">{Math.round(z * 100)}%</span>
        <Button size="sm" icon onClick={() => setZ((v) => Math.min(4, v * 1.2))} aria-label="Zoom in"><IconPlus width={14} height={14} /></Button>
        <Button size="sm" icon onClick={fit} aria-label="Fit to view" title="Fit"><IconFit width={14} height={14} /></Button>
      </>} />
      <div ref={canvas} className={`vp-canvas${drag.current ? " dragging" : ""}`}
        style={{ backgroundPosition: `${pan.x}px ${pan.y}px, ${pan.x}px ${pan.y}px, ${pan.x}px ${pan.y}px, ${pan.x}px ${pan.y}px`, backgroundSize: `${80 * z}px ${80 * z}px, ${80 * z}px ${80 * z}px, ${16 * z}px ${16 * z}px, ${16 * z}px ${16 * z}px` }}
        onPointerDown={(e) => { (e.target as HTMLElement).setPointerCapture?.(e.pointerId); drag.current = { x: e.clientX, y: e.clientY, px: pan.x, py: pan.y }; }}
        onPointerMove={(e) => { const d = drag.current; if (d) setPan({ x: d.px + e.clientX - d.x, y: d.py + e.clientY - d.y }); }}
        onPointerUp={() => { drag.current = null; }} onPointerCancel={() => { drag.current = null; }}
        onDoubleClick={fit}>
        {content ? <div className="vp-stage" style={{ transform: `translate(${pan.x}px, ${pan.y}px) scale(${z})` }}>{content}</div> : <div className="vp-empty"><span className="vp-cross" aria-hidden />{empty}</div>}
        <div className="vp-legend mono" aria-hidden>{view === "section" ? "SECTION · proportional · mm" : "LOADING · schematic · inputs only"}</div>
        <div className="vp-hint" aria-hidden>drag to pan · wheel to zoom · double-click to fit</div>
      </div>
    </section>
  );
}
