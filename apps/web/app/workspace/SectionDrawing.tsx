// Dimensioned schematic drawings generated from the engine's `drawing` payload (geometry only, never results).
// Drawn proportionally to scale; dimension values are the input values in mm. Colours come from material tokens.
import type { ReactNode } from "react";
import type { Drawing } from "../../lib/types";

const M = 56; // margin around the member, room for dimensions
const TICK = 4;

type Side = "below" | "above" | "left" | "right";

/** Architectural dimension: extension lines, dimension line with 45° ticks, centred label. */
function Dim({ a, b, side, off = 22, label, show }: { a: [number, number]; b: [number, number]; side: Side; off?: number; label: ReactNode; show: boolean }) {
  if (!show) return null;
  const horiz = side === "below" || side === "above";
  const s = side === "below" || side === "right" ? 1 : -1;
  if (horiz) {
    const y = (side === "below" ? Math.max(a[1], b[1]) : Math.min(a[1], b[1])) + s * off;
    const [x1, x2] = [Math.min(a[0], b[0]), Math.max(a[0], b[0])];
    return (
      <g className="dim">
        <line x1={a[0]} y1={a[1] + s * 4} x2={a[0]} y2={y + s * 4} className="dim-ext" />
        <line x1={b[0]} y1={b[1] + s * 4} x2={b[0]} y2={y + s * 4} className="dim-ext" />
        <line x1={x1} y1={y} x2={x2} y2={y} className="dim-line" />
        <line x1={x1 - TICK} y1={y + TICK} x2={x1 + TICK} y2={y - TICK} className="dim-tick" />
        <line x1={x2 - TICK} y1={y + TICK} x2={x2 + TICK} y2={y - TICK} className="dim-tick" />
        <text x={(x1 + x2) / 2} y={y + (s > 0 ? 13 : -5)} className="dim-txt" textAnchor="middle">{label}</text>
      </g>
    );
  }
  const x = (side === "right" ? Math.max(a[0], b[0]) : Math.min(a[0], b[0])) + s * off;
  const [y1, y2] = [Math.min(a[1], b[1]), Math.max(a[1], b[1])];
  const tx = x + (s > 0 ? 13 : -5);
  return (
    <g className="dim">
      <line x1={a[0] + s * 4} y1={a[1]} x2={x + s * 4} y2={a[1]} className="dim-ext" />
      <line x1={b[0] + s * 4} y1={b[1]} x2={x + s * 4} y2={b[1]} className="dim-ext" />
      <line x1={x} y1={y1} x2={x} y2={y2} className="dim-line" />
      <line x1={x - TICK} y1={y1 + TICK} x2={x + TICK} y2={y1 - TICK} className="dim-tick" />
      <line x1={x - TICK} y1={y2 + TICK} x2={x + TICK} y2={y2 - TICK} className="dim-tick" />
      <text x={tx} y={(y1 + y2) / 2} className="dim-txt" textAnchor="middle" transform={`rotate(-90 ${tx} ${(y1 + y2) / 2})`}>{label}</text>
    </g>
  );
}

const Note = ({ x, y, children, anchor = "middle" }: { x: number; y: number; children: ReactNode; anchor?: "start" | "middle" | "end" }) => (
  <text x={x} y={y} className="note-txt" textAnchor={anchor}>{children}</text>
);

const range = (n: number) => Array.from({ length: Math.max(0, Math.min(200, Math.floor(n))) }, (_, i) => i);
const fmt = (v: number) => (Number.isInteger(v) ? String(v) : v.toFixed(1));

/** True when every numeric property is finite and non-negative, so the drawing can be scaled. */
function drawable(d: Drawing): boolean {
  return Object.values(d).every((v) => typeof v === "string" || (typeof v === "number" && Number.isFinite(v) && v >= 0));
}

/** Fits a w × h model (mm) into a box of maxW × maxH px. */
const fit = (w: number, h: number, maxW: number, maxH: number) => Math.min(maxW / Math.max(w, 1), maxH / Math.max(h, 1));

function Svg({ w, h, label, children }: { w: number; h: number; label: string; children: ReactNode }) {
  return <svg width={Math.ceil(w)} height={Math.ceil(h)} viewBox={`0 0 ${w} ${h}`} role="img" aria-label={label} preserveAspectRatio="xMidYMid meet">{children}</svg>;
}

function Body({ d, dims }: { d: Drawing; dims: boolean }) {
  switch (d.kind) {
    case "Beam":
    case "Column": {
      const sc = fit(d.b, d.h, 220, 260), W = d.b * sc, H = d.h * sc, ox = M, oy = 24;
      const off = (d.cover + d.link_dia / 2) * sc, ed = (d.cover + d.link_dia + d.bar_dia / 2) * sc, br = Math.max(2.4, (d.bar_dia * sc) / 2);
      const n = Math.max(2, d.kind === "Beam" ? d.n_bars : d.n_face);
      const rows = d.kind === "Beam" ? [oy + H - ed] : [oy + ed, oy + H - ed];
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label={`${d.kind} section ${d.b} by ${d.h} millimetres`}>
          <rect x={ox} y={oy} width={W} height={H} className="m-concrete m-edge" />
          <rect x={ox + off} y={oy + off} width={W - 2 * off} height={H - 2 * off} className="m-link" />
          {rows.map((y, r) => range(n).map((i) => <circle key={`${r}-${i}`} cx={ox + ed + ((W - 2 * ed) * i) / (n - 1)} cy={y} r={br} className="m-bar" />))}
          {d.kind === "Beam" && [0, 1].map((i) => <circle key={i} cx={ox + ed + (W - 2 * ed) * i} cy={oy + ed} r={Math.max(2.2, 6 * sc)} className="m-bar-2" />)}
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`b = ${fmt(d.b)}`} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label={`h = ${fmt(d.h)}`} />
          {dims && <Note x={ox + W + 8} y={rows[rows.length - 1] + 4} anchor="start">{d.kind === "Beam" ? `${n}φ${fmt(d.bar_dia)}` : `${n}φ${fmt(d.bar_dia)}/face`}</Note>}
        </Svg>
      );
    }
    case "Slab": {
      const span = 1000, sc = fit(span, d.h, 300, 140), W = span * sc, H = d.h * sc, ox = M, oy = 24, bars: number[] = [];
      if (d.spacing > 0) for (let x = d.spacing / 2; x < span && bars.length < 200; x += d.spacing) bars.push(x);
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label={`Slab section ${d.h} millimetres thick, 1 metre strip`}>
          <rect x={ox} y={oy} width={W} height={H} className="m-concrete m-edge" />
          {bars.map((x, i) => <circle key={i} cx={ox + x * sc} cy={oy + H - (d.cover + d.bar_dia / 2) * sc} r={Math.max(2.2, (d.bar_dia * sc) / 2)} className="m-bar" />)}
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label={`h = ${fmt(d.h)}`} />
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label="1000 (design strip)" />
          {dims && <Note x={ox + W / 2} y={oy - 8}>φ{fmt(d.bar_dia)} @ {fmt(d.spacing)} bottom</Note>}
        </Svg>
      );
    }
    case "Pad": {
      const sc = fit(d.bx, d.by, 220, 220), W = d.bx * sc, H = d.by * sc, ox = M, oy = 24;
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label="Pad footing plan">
          <rect x={ox} y={oy} width={W} height={H} className="m-concrete m-edge" />
          <rect x={ox + (W - d.cx * sc) / 2} y={oy + (H - d.cy * sc) / 2} width={d.cx * sc} height={d.cy * sc} className="m-solid" />
          <line x1={ox + W / 2} y1={oy - 8} x2={ox + W / 2} y2={oy + H + 8} className="m-axis" />
          <line x1={ox - 8} y1={oy + H / 2} x2={ox + W + 8} y2={oy + H / 2} className="m-axis" />
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`B = ${fmt(d.bx)}`} />
          <Dim show={dims} a={[ox + W, oy]} b={[ox + W, oy + H]} side="right" label={`L = ${fmt(d.by)}`} />
          {dims && <Note x={ox + 4} y={oy - 8} anchor="start">plan · h = {fmt(d.h)}</Note>}
        </Svg>
      );
    }
    case "Bolts": {
      const len = 2 * d.e1 + (d.n1 - 1) * d.p1, sc = fit(len, d.w, 280, 180), W = len * sc, H = d.w * sc, ox = M, oy = 40;
      const y0 = oy + ((d.w - (d.n2 - 1) * d.p2) / 2) * sc;
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label={`${d.n1} by ${d.n2} bolt group`}>
          <rect x={ox} y={oy} width={W} height={H} className="m-plate m-edge" />
          {range(d.n1).map((i) => range(d.n2).map((j) => <circle key={`${i}-${j}`} cx={ox + (d.e1 + i * d.p1) * sc} cy={y0 + j * d.p2 * sc} r={Math.max(3, (d.d * sc) / 2)} className="m-hole" />))}
          <Dim show={dims} a={[ox, oy]} b={[ox + d.e1 * sc, oy]} side="above" off={14} label={fmt(d.e1)} />
          {d.n1 > 1 && <Dim show={dims} a={[ox + d.e1 * sc, oy]} b={[ox + (d.e1 + d.p1) * sc, oy]} side="above" off={14} label={fmt(d.p1)} />}
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={fmt(len)} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label={`w = ${fmt(d.w)}`} />
          {dims && <Note x={ox + W + 8} y={oy + H / 2} anchor="start">{d.n1}×{d.n2} M{fmt(d.d)}</Note>}
        </Svg>
      );
    }
    case "Weld": {
      const W = 260, ox = M, oy = 40, t = 18;
      return (
        <Svg w={W + 2 * M} h={150} label="Fillet weld">
          <rect x={ox} y={oy} width={W} height={t} className="m-plate m-edge" />
          <rect x={ox} y={oy + t} width={W} height={t} className="m-plate-2 m-edge" />
          <polygon points={`${ox},${oy + 2 * t} ${ox},${oy + 2 * t + 14} ${ox + 14},${oy + 2 * t}`} className="m-weld" />
          <Dim show={dims} a={[ox, oy + 2 * t]} b={[ox + W, oy + 2 * t]} side="below" off={30} label={`L = ${fmt(d.lw)}`} />
          {dims && <Note x={ox + 22} y={oy + 2 * t + 13} anchor="start">a = {fmt(d.a)} (throat)</Note>}
        </Svg>
      );
    }
    case "Raft": {
      const sc = fit(d.l, d.w, 300, 160), W = d.l * sc, H = d.w * sc, ox = M, oy = 24;
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label="Raft plan">
          <rect x={ox} y={oy} width={W} height={H} className="m-concrete m-edge" />
          <line x1={ox - 8} y1={oy + H / 2} x2={ox + W + 8} y2={oy + H / 2} className="m-axis" />
          {range(d.n).map((i) => <rect key={i} x={ox + (d.edge + i * d.spacing) * sc - 5} y={oy + H / 2 - 5} width="10" height="10" className="m-solid" />)}
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`L = ${fmt(Math.round(d.l))}`} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label={`w = ${fmt(d.w)}`} />
          {d.n > 1 && <Dim show={dims} a={[ox + d.edge * sc, oy]} b={[ox + (d.edge + d.spacing) * sc, oy]} side="above" off={12} label={fmt(d.spacing)} />}
        </Svg>
      );
    }
    case "Retain": {
      const sc = fit(d.b, d.hs + d.tb, 240, 240), W = d.b * sc, H = (d.hs + d.tb) * sc, ox = M, oy = 24;
      const stemX = ox + d.lt * sc, heelX = stemX + d.ts * sc;
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label="Retaining wall section">
          <rect x={heelX} y={oy} width={Math.max(0, ox + W - heelX)} height={d.hs * sc} className="m-soil" />
          <rect x={ox} y={oy + d.hs * sc} width={W} height={d.tb * sc} className="m-concrete m-edge" />
          <rect x={stemX} y={oy} width={d.ts * sc} height={d.hs * sc} className="m-concrete m-edge" />
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`B = ${fmt(d.b)}`} />
          <Dim show={dims} a={[ox, oy + d.hs * sc]} b={[ox, oy + H]} side="left" off={14} label={fmt(d.tb)} />
          <Dim show={dims} a={[stemX, oy]} b={[stemX, oy + d.hs * sc]} side="left" off={stemX - ox + 30} label={`Hs = ${fmt(d.hs)}`} />
          <Dim show={dims} a={[stemX, oy]} b={[heelX, oy]} side="above" off={12} label={fmt(d.ts)} />
          {dims && <Note x={heelX + 6} y={oy + 14} anchor="start">retained soil</Note>}
        </Svg>
      );
    }
    case "ISection": {
      const sc = fit(d.b, d.h, 200, 260), W = d.b * sc, H = d.h * sc, ox = M, oy = 24, tw = Math.max(1.5, d.tw * sc), tf = Math.max(1.5, d.tf * sc);
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label={`I-section ${d.h} by ${d.b} millimetres`}>
          <rect x={ox} y={oy} width={W} height={tf} className="m-steel" />
          <rect x={ox} y={oy + H - tf} width={W} height={tf} className="m-steel" />
          <rect x={ox + (W - tw) / 2} y={oy} width={tw} height={H} className="m-steel" />
          <line x1={ox - 8} y1={oy + H / 2} x2={ox + W + 8} y2={oy + H / 2} className="m-axis" />
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`b = ${fmt(d.b)}`} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label={`h = ${fmt(d.h)}`} />
          {dims && <Note x={ox + W / 2 + tw / 2 + 6} y={oy + H / 2 - 6} anchor="start">tw {fmt(d.tw)} · tf {fmt(d.tf)}</Note>}
        </Svg>
      );
    }
    case "Rect": {
      const sc = fit(d.b, d.h, 200, 260), W = d.b * sc, H = d.h * sc, ox = M, oy = 24;
      return (
        <Svg w={W + 2 * M} h={H + oy + M} label={`Timber section ${d.b} by ${d.h} millimetres`}>
          <rect x={ox} y={oy} width={W} height={H} className="m-timber m-edge" />
          {[0.3, 0.5, 0.7].map((t) => <path key={t} d={`M${ox + 6} ${oy + H * t} C ${ox + W * 0.35} ${oy + H * t - 8}, ${ox + W * 0.65} ${oy + H * t + 8}, ${ox + W - 6} ${oy + H * t}`} className="m-grain" />)}
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`b = ${fmt(d.b)}`} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label={`h = ${fmt(d.h)}`} />
        </Svg>
      );
    }
    case "Wall": {
      const len = 1000, sc = fit(d.h, len, 160, 260), W = d.h * sc, H = len * sc, ox = M + 20, oy = 24, ys: number[] = [];
      if (d.spacing > 0) for (let y = d.spacing / 2; y < len && ys.length < 200; y += d.spacing) ys.push(y);
      const inset = (d.cover + d.bar_dia / 2) * sc;
      return (
        <Svg w={W + 2 * M + 80} h={H + oy + M} label={`Wall section ${d.h} millimetres thick`}>
          <rect x={ox} y={oy} width={W} height={H} className="m-concrete m-edge" />
          {ys.map((y, i) => [ox + inset, ox + W - inset].map((x, k) => <circle key={`${i}-${k}`} cx={x} cy={oy + y * sc} r={Math.max(2.2, (d.bar_dia * sc) / 2)} className="m-bar" />))}
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`h = ${fmt(d.h)}`} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" label="1000 (per m)" />
          {dims && <Note x={ox + W + 8} y={oy + 20} anchor="start">φ{fmt(d.bar_dia)} @ {fmt(d.spacing)} EF</Note>}
        </Svg>
      );
    }
    case "Strip": {
      const sc = fit(d.width, d.h + 600, 280, 220), W = d.width * sc, H = d.h * sc, T = d.t * sc, ox = M, oy = 24, wallH = 600 * sc;
      return (
        <Svg w={W + 2 * M} h={wallH + H + oy + M} label="Strip footing section">
          <rect x={ox + (W - T) / 2} y={oy} width={T} height={wallH} className="m-concrete m-edge" />
          <rect x={ox} y={oy + wallH} width={W} height={H} className="m-concrete m-edge" />
          <line x1={ox + 8} x2={ox + W - 8} y1={oy + wallH + H - 8} y2={oy + wallH + H - 8} className="m-rebar" />
          <Dim show={dims} a={[ox, oy + wallH + H]} b={[ox + W, oy + wallH + H]} side="below" label={`B = ${fmt(d.width)}`} />
          <Dim show={dims} a={[ox + W, oy + wallH]} b={[ox + W, oy + wallH + H]} side="right" label={`h = ${fmt(d.h)}`} />
          <Dim show={dims} a={[ox + (W - T) / 2, oy]} b={[ox + (W + T) / 2, oy]} side="above" off={10} label={`t = ${fmt(d.t)}`} />
        </Svg>
      );
    }
    case "Plate": {
      const sc = fit(d.b, Math.max(d.t, 1), 360, 60), W = d.b * sc, H = Math.max(10, d.t * sc), ox = M, oy = 40;
      const holes = range(d.n_holes).map((k) => ox + (W * (k + 1)) / (d.n_holes + 1));
      return (
        <Svg w={W + 2 * M} h={H + oy + M + 20} label={`Plate ${d.b} by ${d.t} millimetres with ${d.n_holes} holes`}>
          <rect x={ox} y={oy} width={W} height={H} className="m-steel m-edge" />
          {holes.map((x, i) => <rect key={i} x={x - (d.d0 * sc) / 2} y={oy - 1} width={d.d0 * sc} height={H + 2} className="m-hole" />)}
          {holes.map((x, i) => <line key={`a${i}`} x1={x} x2={x} y1={oy - 12} y2={oy + H + 12} className="m-axis" />)}
          <Dim show={dims} a={[ox, oy + H]} b={[ox + W, oy + H]} side="below" label={`b = ${fmt(d.b)}`} />
          <Dim show={dims} a={[ox + W, oy]} b={[ox + W, oy + H]} side="right" off={14} label={`t = ${fmt(d.t)}`} />
          {dims && d.n_holes > 0 && <Note x={ox + W / 2} y={oy - 18}>{d.n_holes} × d0 = {fmt(d.d0)} · section through holes</Note>}
        </Svg>
      );
    }
    case "Punching": {
      const span = Math.max(d.c1, d.c2) + 4 * d.d + 200, sc = fit(span, span, 300, 300);
      const C1 = d.c1 * sc, C2 = d.c2 * sc, R = 2 * d.d * sc, ox = M + 20, oy = 24, size = span * sc;
      const cx = d.pos === 1 ? ox + size / 2 : ox + 30 + C1 / 2, cy = d.pos === 3 ? oy + 30 + C2 / 2 : oy + size / 2;
      const left = cx - C1 / 2, top = cy - C2 / 2;
      const edgeX = d.pos === 1 ? null : left, edgeY = d.pos === 3 ? top : null;
      return (
        <Svg w={size + 2 * M + 40} h={size + oy + M} label={`Punching control perimeter around a ${d.c1} by ${d.c2} column`}>
          <rect x={ox} y={oy} width={size} height={size} className="m-plate-2" />
          {edgeX !== null && <line x1={edgeX} x2={edgeX} y1={oy} y2={oy + size} className="m-edge" />}
          {edgeY !== null && <line x1={ox} x2={ox + size} y1={edgeY} y2={edgeY} className="m-edge" />}
          <rect x={left - R} y={top - R} width={C1 + 2 * R} height={C2 + 2 * R} rx={R} className="m-perim" clipPath={undefined} />
          {edgeX !== null && <rect x={ox} y={oy} width={Math.max(0, edgeX - ox)} height={size} className="m-void" />}
          {edgeY !== null && <rect x={ox} y={oy} width={size} height={Math.max(0, edgeY - oy)} className="m-void" />}
          <rect x={left} y={top} width={C1} height={C2} className="m-concrete m-edge" />
          <Dim show={dims} a={[left, top + C2]} b={[left + C1, top + C2]} side="below" off={R + 14} label={`c1 = ${fmt(d.c1)}`} />
          <Dim show={dims} a={[left + C1, top]} b={[left + C1, top + C2]} side="right" off={R + 14} label={`c2 = ${fmt(d.c2)}`} />
          {dims && <Note x={left + C1 + R * 0.72 + 6} y={top - R * 0.72 - 4} anchor="start">u1 at 2d = {fmt(2 * d.d)}</Note>}
          {dims && <Note x={ox + size / 2} y={oy + size + 18}>plan · h = {fmt(d.h)} · d = {fmt(d.d)} · {d.pos === 1 ? "internal" : d.pos === 2 ? "edge" : "corner"} column</Note>}
        </Svg>
      );
    }
    case "Masonry": {
      const sc = fit(d.t, d.h, 120, 280), W = Math.max(14, d.t * sc), H = d.h * sc, ox = M + 60, oy = 50, course = Math.max(4, 75 * sc);
      const courses = range(H / course);
      return (
        <Svg w={W + 2 * M + 140} h={H + oy + M} label={`Masonry wall ${d.t} millimetres thick, ${d.h} high`}>
          <rect x={ox - 30} y={oy - 14} width={W + 60} height={14} className="m-plate-2 m-edge" />
          <rect x={ox - 30} y={oy + H} width={W + 60} height={14} className="m-plate-2 m-edge" />
          <rect x={ox} y={oy} width={W} height={H} className="m-masonry m-edge" />
          {courses.map((i) => <line key={i} x1={ox} x2={ox + W} y1={oy + (i + 1) * course} y2={oy + (i + 1) * course} className="m-mortar" />)}
          <line x1={ox + W / 2} x2={ox + W / 2} y1={oy - 30} y2={oy + H + 30} className="m-axis" />
          <line x1={ox + W / 2 + d.e * sc} x2={ox + W / 2 + d.e * sc} y1={oy - 46} y2={oy - 18} className="m-load" />
          <polygon points={`${ox + W / 2 + d.e * sc},${oy - 16} ${ox + W / 2 + d.e * sc - 4},${oy - 24} ${ox + W / 2 + d.e * sc + 4},${oy - 24}`} className="m-load-head" />
          <Dim show={dims} a={[ox, oy + H + 14]} b={[ox + W, oy + H + 14]} side="below" label={`t = ${fmt(d.t)}`} />
          <Dim show={dims} a={[ox, oy]} b={[ox, oy + H]} side="left" off={40} label={`h = ${fmt(d.h)}`} />
          {dims && <Note x={ox + W / 2 + d.e * sc + 8} y={oy - 34} anchor="start">N Ed at e = {fmt(d.e)}</Note>}
        </Svg>
      );
    }
    default:
      return null;
  }
}

export default function SectionDrawing({ d, dims = true }: { d: Drawing | null; dims?: boolean }) {
  if (!d || !drawable(d)) return null;
  return <Body d={d} dims={dims} />;
}
