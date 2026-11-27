// Schematic section drawings generated from the engine's `drawing` payload. Geometry only, never results.
// Colours come from CSS classes (sd-*) so the drawing follows the active theme.
import type { Drawing } from "../../lib/types";

const Dim = ({ x, y, children, rotate }: { x: number; y: number; children: React.ReactNode; rotate?: boolean }) => (
  <text x={x} y={y} className="sd-dim" textAnchor="middle" transform={rotate ? `rotate(-90 ${x} ${y})` : undefined}>{children}</text>
);

const range = (n: number) => Array.from({ length: Math.max(0, Math.min(200, Math.floor(n))) }, (_, i) => i);

/** True when every numeric property is finite and the drawing can be scaled. */
function drawable(d: Drawing): boolean {
  return Object.values(d).every((v) => typeof v === "string" || (typeof v === "number" && Number.isFinite(v) && v >= 0));
}

function Body({ d }: { d: Drawing }) {
  switch (d.kind) {
    case "Beam":
    case "Column": {
      const sc = Math.min((d.kind === "Beam" ? 150 : 170) / d.b, 250 / d.h), W = d.b * sc, H = d.h * sc, ox = 28, oy = 14;
      const off = (d.cover + d.link_dia / 2) * sc, ed = (d.cover + d.link_dia + d.bar_dia / 2) * sc, br = Math.max(2.4, (d.bar_dia * sc) / 2);
      const n = Math.max(2, d.kind === "Beam" ? d.n_bars : d.n_face);
      const rows = d.kind === "Beam" ? [oy + H - ed] : [oy + ed, oy + H - ed];
      return (
        <svg viewBox={`0 0 ${W + 56} ${H + 44}`} role="img" aria-label={`${d.kind === "Beam" ? "Beam" : "Column"} section ${d.b} by ${d.h} mm`}>
          <rect x={ox} y={oy} width={W} height={H} className="sd-fill sd-line" />
          <rect x={ox + off} y={oy + off} width={W - 2 * off} height={H - 2 * off} className="sd-link" />
          {rows.map((y, r) => range(n).map((i) => <circle key={`${r}-${i}`} cx={ox + ed + ((W - 2 * ed) * i) / (n - 1)} cy={y} r={br} className="sd-bar" />))}
          {d.kind === "Beam" && [0, 1].map((i) => <circle key={i} cx={ox + ed + (W - 2 * ed) * i} cy={oy + ed} r={Math.max(2.2, 6 * sc)} className="sd-bar-light" />)}
          <Dim x={ox + W / 2} y={oy + H + 24}>{d.kind === "Beam" ? `b = ${d.b}` : `${d.b} × ${d.h}`}</Dim>
          {d.kind === "Beam" && <Dim x={12} y={oy + H / 2} rotate>h = {d.h}</Dim>}
        </svg>
      );
    }
    case "Bolts": {
      const len = 2 * d.e1 + (d.n1 - 1) * d.p1, sc = 200 / Math.max(len, d.w), W = len * sc, H = d.w * sc, ox = 20, oy = 14, y0 = oy + ((d.w - (d.n2 - 1) * d.p2) / 2) * sc;
      return (
        <svg viewBox={`0 0 ${W + 40} ${H + 50}`} role="img" aria-label={`${d.n1} by ${d.n2} bolt group`}>
          <rect x={ox} y={oy} width={W} height={H} className="sd-fill sd-line" />
          {range(d.n1).map((i) => range(d.n2).map((j) => <circle key={`${i}-${j}`} cx={ox + (d.e1 + i * d.p1) * sc} cy={y0 + j * d.p2 * sc} r={Math.max(3, (d.d * sc) / 2)} className="sd-hole" />))}
          <Dim x={ox + W / 2} y={oy + H + 24}>{d.n1} × {d.n2} bolts · plate {d.w}</Dim>
        </svg>
      );
    }
    case "Weld":
      return (
        <svg viewBox="0 0 220 110" role="img" aria-label="Fillet weld">
          <rect x="20" y="25" width="180" height="16" className="sd-steel" />
          <rect x="20" y="41" width="180" height="16" className="sd-steel-light" />
          <polygon points="20,57 20,70 33,57" className="sd-bar" />
          <Dim x={110} y={95}>a = {d.a} · L = {d.lw}</Dim>
        </svg>
      );
    case "Raft": {
      const sc = 200 / Math.max(d.l, d.w), W = d.l * sc, H = d.w * sc, ox = 20, oy = 14;
      return (
        <svg viewBox={`0 0 ${W + 40} ${H + 50}`} role="img" aria-label="Raft plan">
          <rect x={ox} y={oy} width={W} height={H} className="sd-fill sd-line" />
          {range(d.n).map((i) => <rect key={i} x={ox + (d.edge + i * d.spacing) * sc - 5} y={oy + H / 2 - 5} width="10" height="10" className="sd-steel" />)}
          <Dim x={ox + W / 2} y={oy + H + 24}>L = {Math.round(d.l)} · w = {d.w} (plan)</Dim>
        </svg>
      );
    }
    case "Retain": {
      const sc = 190 / Math.max(d.hs + d.tb, d.b), W = d.b * sc, H = (d.hs + d.tb) * sc, ox = 20, oy = 14;
      return (
        <svg viewBox={`0 0 ${W + 40} ${H + 50}`} role="img" aria-label="Retaining wall section">
          <rect x={ox + (d.lt + d.ts) * sc} y={oy} width={Math.max(0, W - (d.lt + d.ts) * sc)} height={d.hs * sc} className="sd-soil" />
          <rect x={ox} y={oy + d.hs * sc} width={W} height={d.tb * sc} className="sd-concrete" />
          <rect x={ox + d.lt * sc} y={oy} width={d.ts * sc} height={d.hs * sc} className="sd-concrete" />
          <Dim x={ox + W / 2} y={oy + H + 24}>H = {d.hs + d.tb} · B = {d.b}</Dim>
        </svg>
      );
    }
    case "Rect": {
      const sc = Math.min(150 / d.b, 250 / d.h), W = d.b * sc, H = d.h * sc, ox = 28, oy = 14;
      return (
        <svg viewBox={`0 0 ${W + 56} ${H + 44}`} role="img" aria-label={`Timber section ${d.b} by ${d.h} mm`}>
          <rect x={ox} y={oy} width={W} height={H} className="sd-timber sd-line" />
          {[0.25, 0.5, 0.75].map((t) => <path key={t} d={`M${ox + W * t - 8} ${oy + H * 0.5} q8 -${Math.min(18, H * 0.3)} 16 0`} className="sd-grain" />)}
          <Dim x={ox + W / 2} y={oy + H + 24}>{d.b} × {d.h}</Dim>
        </svg>
      );
    }
    case "ISection": {
      const sc = Math.min(160 / d.b, 250 / d.h), W = d.b * sc, H = d.h * sc, ox = 28, oy = 14, tw = d.tw * sc, tf = d.tf * sc;
      return (
        <svg viewBox={`0 0 ${W + 56} ${H + 44}`} role="img" aria-label={`I-section ${d.h} by ${d.b} mm`}>
          <rect x={ox} y={oy} width={W} height={tf} className="sd-steel" />
          <rect x={ox} y={oy + H - tf} width={W} height={tf} className="sd-steel" />
          <rect x={ox + (W - tw) / 2} y={oy} width={tw} height={H} className="sd-steel" />
          <Dim x={ox + W / 2} y={oy + H + 24}>{d.h} × {d.b} × {d.tw} / {d.tf}</Dim>
        </svg>
      );
    }
    case "Wall": {
      const sc = Math.min(120 / d.h, 1), W = d.h * sc, H = 260, ox = 60, oy = 14, ys: number[] = [];
      if (d.spacing > 0) for (let y = d.spacing / 2; y < 400 && ys.length < 200; y += d.spacing) ys.push(y);
      return (
        <svg viewBox={`0 0 ${W + 120} ${H + 44}`} role="img" aria-label={`Wall section ${d.h} mm thick`}>
          <rect x={ox} y={oy} width={W} height={H} className="sd-fill sd-line" />
          {ys.map((y, i) => [ox + (d.cover + 10) * sc, ox + W - (d.cover + 10) * sc].map((x, k) => <circle key={`${i}-${k}`} cx={x} cy={oy + (y / 400) * H} r={Math.max(2.2, (d.bar_dia * sc) / 2)} className="sd-bar" />))}
          <Dim x={ox + W / 2} y={oy + H + 24}>h = {d.h} · T{d.bar_dia} @ {d.spacing}</Dim>
        </svg>
      );
    }
    case "Strip": {
      const sc = 190 / d.width, W = d.width * sc, H = d.h * sc, ox = 20, oy = 14, T = d.t * sc;
      return (
        <svg viewBox={`0 0 ${W + 40} ${H + 100}`} role="img" aria-label="Strip footing section">
          <rect x={ox + (W - T) / 2} y={oy} width={T} height={60} className="sd-concrete" />
          <rect x={ox} y={oy + 60} width={W} height={H} className="sd-fill sd-line" />
          <line x1={ox + 8} x2={ox + W - 8} y1={oy + 60 + H - 8} y2={oy + 60 + H - 8} className="sd-rebar" />
          <Dim x={ox + W / 2} y={oy + 60 + H + 24}>B = {d.width} · h = {d.h}</Dim>
        </svg>
      );
    }
    case "Slab": {
      const sc = 0.55, W = 400 * sc, H = d.h * sc, ox = 20, oy = 20, bars: number[] = [];
      if (d.spacing > 0) for (let x = d.spacing / 2; x < 400 && bars.length < 200; x += d.spacing) bars.push(x);
      return (
        <svg viewBox={`0 0 ${W + 40} ${H + 60}`} role="img" aria-label={`Slab section ${d.h} mm thick`}>
          <rect x={ox} y={oy} width={W} height={H} className="sd-fill sd-line" />
          {bars.map((x, i) => <circle key={i} cx={ox + x * sc} cy={oy + H - (d.cover + d.bar_dia / 2) * sc} r={Math.max(2.2, (d.bar_dia * sc) / 2)} className="sd-bar" />)}
          <Dim x={ox + W / 2} y={oy + H + 22}>T{d.bar_dia} @ {d.spacing}</Dim>
          <Dim x={ox + W / 2} y={oy + H + 38}>h = {d.h} · 1 m strip</Dim>
        </svg>
      );
    }
    case "Pad": {
      const sc = 170 / Math.max(d.bx, d.by), W = d.bx * sc, H = d.by * sc, ox = 25, oy = 14;
      return (
        <svg viewBox={`0 0 ${W + 50} ${H + 50}`} role="img" aria-label="Pad footing plan">
          <rect x={ox} y={oy} width={W} height={H} className="sd-fill sd-line" />
          {range(6).map((i) => <line key={i} x1={ox + 6} x2={ox + W - 6} y1={oy + (H * (i + 1)) / 7} y2={oy + (H * (i + 1)) / 7} className="sd-hatch" />)}
          <rect x={ox + (W - d.cx * sc) / 2} y={oy + (H - d.cy * sc) / 2} width={d.cx * sc} height={d.cy * sc} className="sd-concrete" />
          <Dim x={ox + W / 2} y={oy + H + 24}>{d.bx} × {d.by} (plan) · h = {d.h}</Dim>
        </svg>
      );
    }
    default:
      return null;
  }
}

export default function SectionDrawing({ d }: { d: Drawing | null }) {
  if (!d || !drawable(d)) return null;
  return <figure className="sd">{<Body d={d} />}</figure>;
}
