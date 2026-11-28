// Loading and support diagrams. Each drawing states exactly the model the engine module uses (from its module
// statement) and is annotated with the input values only — never with computed results.
import type { ReactNode } from "react";

type V = Record<string, unknown>;
const n = (v: V, k: string) => { const x = Number(v[k]); return Number.isFinite(x) ? x : 0; };
const f = (x: number) => (Number.isInteger(x) ? String(x) : x.toFixed(2).replace(/\.?0+$/, ""));

const W = 560, H = 300;

function Svg({ label, children }: { label: string; children: ReactNode }) {
  return <svg width={W} height={H} viewBox={`0 0 ${W} ${H}`} role="img" aria-label={label}>{children}</svg>;
}
/** Downward arrow with its tip at (x, y). */
const Arrow = ({ x, y, len = 34, up }: { x: number; y: number; len?: number; up?: boolean }) => {
  const s = up ? -1 : 1;
  return <g className="ld-load"><line x1={x} y1={y - s * len} x2={x} y2={y - s * 6} /><polygon points={`${x},${y} ${x - 4},${y - s * 8} ${x + 4},${y - s * 8}`} /></g>;
};
const HArrow = ({ x, y, len = 40, left }: { x: number; y: number; len?: number; left?: boolean }) => {
  const s = left ? -1 : 1;
  return <g className="ld-load"><line x1={x - s * len} y1={y} x2={x - s * 6} y2={y} /><polygon points={`${x},${y} ${x - s * 8},${y - 4} ${x - s * 8},${y + 4}`} /></g>;
};
/** Uniformly distributed load between x1 and x2 acting down on y. */
const Udl = ({ x1, x2, y, label }: { x1: number; x2: number; y: number; label: string }) => {
  const k = Math.max(2, Math.round((x2 - x1) / 36));
  return (
    <g>
      <line x1={x1} y1={y - 34} x2={x2} y2={y - 34} className="ld-load-line" />
      {Array.from({ length: k + 1 }, (_, i) => <Arrow key={i} x={x1 + ((x2 - x1) * i) / k} y={y - 2} len={32} />)}
      <text x={(x1 + x2) / 2} y={y - 42} className="ld-txt load" textAnchor="middle">{label}</text>
    </g>
  );
};
const Pin = ({ x, y }: { x: number; y: number }) => (<g className="ld-sup"><polygon points={`${x},${y} ${x - 10},${y + 16} ${x + 10},${y + 16}`} /><line x1={x - 16} y1={y + 16} x2={x + 16} y2={y + 16} />{[-12, -4, 4, 12].map((d) => <line key={d} x1={x + d} y1={y + 16} x2={x + d - 5} y2={y + 22} />)}</g>);
const Roller = ({ x, y }: { x: number; y: number }) => (<g className="ld-sup"><polygon points={`${x},${y} ${x - 10},${y + 13} ${x + 10},${y + 13}`} /><circle cx={x - 5} cy={y + 16} r={3} /><circle cx={x + 5} cy={y + 16} r={3} /><line x1={x - 16} y1={y + 20} x2={x + 16} y2={y + 20} /></g>);
const Dim = ({ x1, x2, y, label }: { x1: number; x2: number; y: number; label: string }) => (
  <g className="ld-dim"><line x1={x1} y1={y - 6} x2={x1} y2={y + 6} /><line x1={x2} y1={y - 6} x2={x2} y2={y + 6} /><line x1={x1} y1={y} x2={x2} y2={y} />
    <line x1={x1 - 4} y1={y + 4} x2={x1 + 4} y2={y - 4} /><line x1={x2 - 4} y1={y + 4} x2={x2 + 4} y2={y - 4} />
    <text x={(x1 + x2) / 2} y={y + 16} textAnchor="middle">{label}</text></g>
);
const VDim = ({ x, y1, y2, label }: { x: number; y1: number; y2: number; label: string }) => (
  <g className="ld-dim"><line x1={x - 6} y1={y1} x2={x + 6} y2={y1} /><line x1={x - 6} y1={y2} x2={x + 6} y2={y2} /><line x1={x} y1={y1} x2={x} y2={y2} />
    <line x1={x - 4} y1={y1 + 4} x2={x + 4} y2={y1 - 4} /><line x1={x - 4} y1={y2 + 4} x2={x + 4} y2={y2 - 4} />
    <text x={x - 10} y={(y1 + y2) / 2} textAnchor="middle" transform={`rotate(-90 ${x - 10} ${(y1 + y2) / 2})`}>{label}</text></g>
);
/** Moment arc (clockwise when cw) centred at (x, y). */
const Moment = ({ x, y, r = 16, cw, label, lx, ly }: { x: number; y: number; r?: number; cw?: boolean; label: string; lx: number; ly: number }) => {
  const a1 = cw ? -150 : -30, a2 = cw ? 150 : 210;
  const p = (a: number) => [x + r * Math.cos((a * Math.PI) / 180), y + r * Math.sin((a * Math.PI) / 180)];
  const [sx, sy] = p(a1), [ex, ey] = p(a2);
  return (<g className="ld-load"><path d={`M${sx} ${sy} A ${r} ${r} 0 1 ${cw ? 1 : 0} ${ex} ${ey}`} fill="none" /><circle cx={ex} cy={ey} r={2.5} /><text x={lx} y={ly} className="ld-txt load">{label}</text></g>);
};
const Note = ({ x, y, children, anchor = "start" }: { x: number; y: number; children: ReactNode; anchor?: "start" | "middle" | "end" }) => <text x={x} y={y} className="ld-txt" textAnchor={anchor}>{children}</text>;
const Ground = ({ x1, x2, y }: { x1: number; x2: number; y: number }) => (<g className="ld-ground"><line x1={x1} y1={y} x2={x2} y2={y} />{Array.from({ length: Math.floor((x2 - x1) / 12) }, (_, i) => <line key={i} x1={x1 + i * 12 + 4} y1={y} x2={x1 + i * 12 - 4} y2={y + 8} />)}</g>);

function SimpleSpan({ v, id }: { v: V; id: string }) {
  const L = n(v, "span"), g = n(v, "gk"), q = n(v, "qk");
  const per = id === "slab" || id === "stairs" ? "kN/m²" : "kN/m";
  const x1 = 70, x2 = 490, y = 160;
  const lcr = id === "steelbeam" ? n(v, "lcr") : 0;
  const nR = lcr > 0 && L > 0 ? Math.min(30, Math.floor(L / lcr)) : 0;
  return (
    <Svg label="Simply supported span with uniformly distributed load">
      <Udl x1={x1} x2={x2} y={y} label={`gk ${f(g)} + qk ${f(q)} ${per}`} />
      <line x1={x1} y1={y} x2={x2} y2={y} className="ld-member" />
      <Pin x={x1} y={y + 4} /><Roller x={x2} y={y + 4} />
      {Array.from({ length: Math.max(0, nR - 1) }, (_, i) => { const x = x1 + ((x2 - x1) * (i + 1)) / nR; return <g key={i} className="ld-restraint"><line x1={x} y1={y - 8} x2={x} y2={y + 8} /><line x1={x - 5} y1={y - 8} x2={x + 5} y2={y - 8} /></g>; })}
      <Dim x1={x1} x2={x2} y={y + 52} label={`L = ${f(L)} mm`} />
      <Note x={x1} y={y + 96}>{id === "slab" ? "Simply supported single span · 1 m design strip" : id === "stairs" ? "Simply supported flight · horizontal span · 1 m width" : id === "timber" ? "Simply supported · rectangular section" : `Simply supported · lateral restraints at Lcr = ${f(lcr)} mm`}</Note>
    </Svg>
  );
}

function VerticalMember({ v, id }: { v: V; id: string }) {
  const x = 280, y1 = 60, y2 = 240;
  const N = n(v, "n_ed");
  const perM = id === "wall" ? " kN/m" : " kN";
  const len = id === "column" || id === "wall" ? n(v, "len") : id === "steelcol" ? n(v, "len") : n(v, "lef_y");
  return (
    <Svg label="Braced compression member">
      <Arrow x={x} y={y1 - 4} len={40} />
      <text x={x + 10} y={y1 - 30} className="ld-txt load">N Ed = {f(N)}{perM}</text>
      <line x1={x} y1={y1} x2={x} y2={y2} className="ld-member" />
      <Pin x={x} y={y2} />
      <g className="ld-sup"><line x1={x - 14} y1={y1 - 2} x2={x + 14} y2={y1 - 2} /><circle cx={x} cy={y1} r={3} /></g>
      <line x1={x - 22} y1={y1} x2={x - 8} y2={y1} className="ld-brace" /><line x1={x - 22} y1={y2} x2={x - 8} y2={y2} className="ld-brace" />
      {(id === "column" || id === "wall") && (<>
        <Moment x={x} y={y1 + 18} cw label={`M02 = ${f(n(v, "m02"))}`} lx={x + 26} ly={y1 + 22} />
        <Moment x={x} y={y2 - 18} label={`M01 = ${f(n(v, "m01"))}`} lx={x + 26} ly={y2 - 14} />
      </>)}
      {id === "timbercol" && <Note x={x + 26} y={(y1 + y2) / 2}>My,d {f(n(v, "my_ed"))} · Mz,d {f(n(v, "mz_ed"))} kNm</Note>}
      <VDim x={x - 70} y1={y1} y2={y2} label={`${id === "timbercol" ? "lef,y" : "l"} = ${f(len)} mm`} />
      <Note x={40} y={H - 16}>{id === "steelcol" ? `Braced · effective-length factors ky = ${f(n(v, "ky"))}, kz = ${f(n(v, "kz"))}` : id === "timbercol" ? "Buckling about both axes" : `Braced · l0 = β·l with β = ${f(n(v, "beta"))}`}{id === "wall" ? " · per metre run" : ""}</Note>
    </Svg>
  );
}

function Footing({ v, id }: { v: V; id: string }) {
  const y = 150, x1 = 120, x2 = 440, depth = 40;
  const strip = id === "strip";
  return (
    <Svg label={strip ? "Strip footing on soil" : "Pad footing on soil"}>
      <rect x={280 - (strip ? 22 : 18)} y={y - 80} width={strip ? 44 : 36} height={80} className="ld-concrete" />
      <Arrow x={280} y={y - 84} len={40} />
      <text x={292} y={y - 108} className="ld-txt load">N Ed = {f(n(v, "n_ed"))} {strip ? "kN/m" : "kN"} · N k = {f(n(v, "n_k"))}</text>
      <rect x={x1} y={y} width={x2 - x1} height={depth} className="ld-concrete" />
      {Array.from({ length: 9 }, (_, i) => <Arrow key={i} x={x1 + 12 + ((x2 - x1 - 24) * i) / 8} y={y + depth + 2} len={26} up />)}
      <text x={280} y={y + depth + 46} className="ld-txt" textAnchor="middle">uniform bearing pressure · allowable q = {f(n(v, "q_allow"))} kPa</text>
      <Ground x1={60} x2={500} y={y} />
      <Note x={40} y={H - 12}>{strip ? "Concentric wall line load per metre" : "Concentric axial load"} · allowable bearing pressure from the geotechnical engineer</Note>
    </Svg>
  );
}

function Raft({ v }: { v: V }) {
  const nc = Math.max(2, Math.min(12, Math.round(n(v, "n_cols")) || 3));
  const x1 = 50, x2 = 510, y = 160;
  return (
    <Svg label="Raft strip with equal column loads">
      {Array.from({ length: nc }, (_, i) => { const x = x1 + 30 + ((x2 - x1 - 60) * i) / (nc - 1); return <g key={i}><rect x={x - 8} y={y - 50} width={16} height={50} className="ld-concrete" /><Arrow x={x} y={y - 54} len={30} /></g>; })}
      <text x={280} y={y - 92} className="ld-txt load" textAnchor="middle">{nc} × N Ed = {f(n(v, "n_ed"))} kN at {f(n(v, "spacing"))} mm</text>
      <rect x={x1} y={y} width={x2 - x1} height={30} className="ld-concrete" />
      {Array.from({ length: 12 }, (_, i) => <Arrow key={i} x={x1 + 10 + ((x2 - x1 - 20) * i) / 11} y={y + 32} len={24} up />)}
      <text x={280} y={y + 74} className="ld-txt" textAnchor="middle">uniform ground pressure · allowable q = {f(n(v, "q_allow"))} kPa · strip width {f(n(v, "width"))} mm</text>
      <Note x={40} y={H - 12}>One column-line strip · equal symmetric column loads · rigid method</Note>
    </Svg>
  );
}

function Retaining({ v }: { v: V }) {
  const hs = n(v, "hs"), tb = n(v, "tb"), bw = n(v, "bw"), lt = n(v, "lt"), ts = n(v, "ts"), hw = n(v, "hw");
  const sc = Math.min(200 / Math.max(hs + tb, 1), 300 / Math.max(bw, 1));
  const ox = 90, base = 250, top = base - (hs + tb) * sc;
  const stemX = ox + lt * sc, heelX = stemX + ts * sc, endX = ox + bw * sc;
  const tri = 70;
  return (
    <Svg label="Cantilever retaining wall with active earth pressure">
      <rect x={heelX} y={top} width={endX - heelX} height={hs * sc} className="ld-soil" />
      <rect x={ox} y={base - tb * sc} width={bw * sc} height={tb * sc} className="ld-concrete" />
      <rect x={stemX} y={top} width={ts * sc} height={hs * sc} className="ld-concrete" />
      {hw > 0 && <line x1={heelX} y1={base - hw * sc} x2={endX + tri + 20} y2={base - hw * sc} className="ld-water" />}
      <polygon points={`${endX + 6},${top} ${endX + 6},${base} ${endX + 6 + tri},${base}`} className="ld-pressure" />
      {Array.from({ length: 5 }, (_, i) => { const y = top + ((base - top) * (i + 1)) / 6; return <HArrow key={i} x={endX + 6} y={y} len={(tri * (i + 1)) / 6 + 6} left />; })}
      {n(v, "q_sur") > 0 && Array.from({ length: 4 }, (_, i) => <Arrow key={i} x={heelX + 10 + ((endX - heelX - 20) * i) / 3} y={top - 2} len={22} />)}
      <text x={endX + 10} y={top - 30} className="ld-txt load">q = {f(n(v, "q_sur"))} kPa</text>
      <text x={endX + tri + 14} y={base - 8} className="ld-txt load">Ka·γ·H</text>
      <Ground x1={40} x2={ox} y={base - tb * sc} />
      <Note x={40} y={H - 12}>Rankine active pressure · γ = {f(n(v, "gamma"))} kN/m³ · φ′ = {f(n(v, "phi"))}° · horizontal backfill{hw > 0 ? ` · water ${f(hw)} mm above base` : ""}</Note>
    </Svg>
  );
}

function Bolted({ v }: { v: V }) {
  const y = 130;
  return (
    <Svg label="Bolted lap joint in tension">
      <rect x={60} y={y - 14} width={300} height={14} className="ld-steel" />
      <rect x={200} y={y} width={300} height={14} className="ld-steel-2" />
      {Array.from({ length: Math.max(1, Math.min(12, Math.round(n(v, "n1")))) }, (_, i) => <rect key={i} x={226 + i * 34} y={y - 22} width={8} height={44} className="ld-bolt" />)}
      <HArrow x={60} y={y - 7} len={44} left />
      <HArrow x={500} y={y + 7} len={44} />
      <text x={24} y={y - 26} className="ld-txt load">N Ed = {f(n(v, "n_ed"))} kN</text>
      <text x={430} y={y + 40} className="ld-txt load">N Ed</text>
      <Note x={40} y={H - 36}>{Math.round(n(v, "n1"))} × {Math.round(n(v, "n2"))} M{f(n(v, "d"))} grade {f(n(v, "grade"))} · {Math.round(n(v, "planes"))} shear plane(s)</Note>
      <Note x={40} y={H - 14}>Concentric tension · bolts in shear and bearing · Category A (non-preloaded)</Note>
    </Svg>
  );
}

function Weld({ v }: { v: V }) {
  const y = 150;
  return (
    <Svg label="Fillet weld under concentric force">
      <rect x={100} y={y - 16} width={360} height={16} className="ld-steel" />
      <rect x={100} y={y} width={360} height={16} className="ld-steel-2" />
      <polygon points={`100,${y + 16} 100,${y + 30} 114,${y + 16}`} className="ld-weld" />
      <Arrow x={280} y={y - 18} len={50} />
      <text x={290} y={y - 54} className="ld-txt load">F⊥ = {f(n(v, "f_perp"))} kN</text>
      <HArrow x={470} y={y - 8} len={50} />
      <text x={380} y={y - 24} className="ld-txt load">F∥ = {f(n(v, "f_par"))} kN</text>
      <Dim x1={100} x2={460} y={y + 52} label={`Lw = ${f(n(v, "lw"))} mm · a = ${f(n(v, "a"))} mm`} />
      <Note x={40} y={H - 12}>Equal-leg fillet welds · directional and simplified methods</Note>
    </Svg>
  );
}

function Actions({ v }: { v: V }) {
  const items: [string, number][] = [["Gk", n(v, "gk")], ["Qk,1", n(v, "q1")], ["Qk,2", n(v, "q2")], ["Qk,3", n(v, "q3")]];
  const max = Math.max(1, ...items.map((x) => Math.abs(x[1])));
  return (
    <Svg label="Characteristic action effects">
      {items.map(([k, val], i) => {
        const h = (Math.abs(val) / max) * 170, x = 90 + i * 110;
        return <g key={k}><rect x={x} y={230 - h} width={56} height={h} className={i === 0 ? "ld-bar-g" : "ld-bar-q"} /><text x={x + 28} y={250} textAnchor="middle" className="ld-txt">{k}</text><text x={x + 28} y={222 - h} textAnchor="middle" className="ld-txt load">{f(val)}</text></g>;
      })}
      <line x1={60} y1={230} x2={520} y2={230} className="ld-member thin" />
      <Note x={40} y={H - 14}>Characteristic action effects in a common unit · combined by EN 1990 Eq 6.10, 6.10a/b and SLS</Note>
    </Svg>
  );
}

function SectionActions({ v }: { v: V }) {
  const y = 140;
  return (
    <Svg label="Design actions on the section">
      <line x1={140} y1={y} x2={420} y2={y} className="ld-member" />
      <Moment x={140} y={y} r={22} label={`M Ed = ${f(n(v, "m_ed"))} kNm (sagging)`} lx={120} ly={y - 34} />
      <Moment x={420} y={y} r={22} cw label="" lx={0} ly={0} />
      <Arrow x={140} y={y + 4} len={36} up />
      <text x={150} y={y + 46} className="ld-txt load">V Ed = {f(n(v, "v_ed"))} kN</text>
      <Note x={40} y={H - 34}>The module verifies a section for design actions supplied from the frame analysis.</Note>
      <Note x={40} y={H - 14}>No span or support model is assumed.</Note>
    </Svg>
  );
}

export default function LoadingDiagram({ id, values }: { id: string; values: V }) {
  switch (id) {
    case "steelbeam": case "timber": case "slab": case "stairs": return <SimpleSpan v={values} id={id} />;
    case "column": case "steelcol": case "timbercol": case "wall": return <VerticalMember v={values} id={id} />;
    case "pad": case "strip": return <Footing v={values} id={id} />;
    case "raft": return <Raft v={values} />;
    case "retaining": return <Retaining v={values} />;
    case "bolted": return <Bolted v={values} />;
    case "weld": return <Weld v={values} />;
    case "combos": return <Actions v={values} />;
    case "beam": return <SectionActions v={values} />;
    default: return null;
  }
}
