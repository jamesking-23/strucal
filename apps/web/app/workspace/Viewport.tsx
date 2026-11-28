// Section viewport: drafting-grid canvas around the dimensioned schematic, with dimension toggle and zoom.
"use client";
import { useState } from "react";
import type { Drawing } from "../../lib/types";
import SectionDrawing from "./SectionDrawing";

const ZOOMS = [0.75, 1, 1.25, 1.5, 2];

export default function Viewport({ drawing, title, state }: { drawing: Drawing | null; title: string; state: "ready" | "invalid" | "none" }) {
  const [zi, setZi] = useState(1);
  const [dims, setDims] = useState(true);
  const zoom = ZOOMS[zi];
  const hasGeom = state === "ready" && !!drawing;
  return (
    <section className="vp" aria-label="Section viewport">
      <div className="vp-bar">
        <span className="vp-title">{title}</span>
        <span className="vp-note">{hasGeom ? "schematic · proportional · dimensions in mm" : ""}</span>
        <span className="spacer" />
        <button type="button" className="vp-btn" aria-pressed={dims} disabled={!hasGeom} onClick={() => setDims(!dims)} title="Show or hide dimensions">Dims</button>
        <span className="vp-sep" />
        <button type="button" className="vp-btn" disabled={!hasGeom || zi === 0} onClick={() => setZi(zi - 1)} aria-label="Zoom out">−</button>
        <button type="button" className="vp-btn vp-zoom" disabled={!hasGeom} onClick={() => setZi(1)} title="Fit">{Math.round(zoom * 100)}%</button>
        <button type="button" className="vp-btn" disabled={!hasGeom || zi === ZOOMS.length - 1} onClick={() => setZi(zi + 1)} aria-label="Zoom in">+</button>
      </div>
      <div className="vp-canvas">
        {hasGeom ? (
          <div className="vp-stage" style={{ transform: `scale(${zoom})` }}><SectionDrawing d={drawing} dims={dims} /></div>
        ) : (
          <div className="vp-empty">
            <span className="vp-cross" aria-hidden />
            <span>{state === "invalid" ? "Geometry appears when every input is valid." : "This module works on action effects; it has no section geometry."}</span>
          </div>
        )}
        <span className="vp-axes" aria-hidden><i>x</i><i>y</i></span>
      </div>
    </section>
  );
}
