// Landing hero instrument: the real WASM engine running the RC beam module in the visitor's browser.
// Moving a slider re-runs the deterministic calculation; nothing here is simulated.
"use client";
import { useMemo, useState } from "react";
import Link from "next/link";
import { useEngine } from "../lib/useEngine";
import { DEFAULT_PARAM_SET } from "../lib/engine";
import { checkStatus, fmtUtil, governing, verdictOf, VERDICT_WORD } from "../lib/status";
import SectionDrawing from "./workspace/SectionDrawing";
import { UtilizationBar } from "./ui/primitives";

const BASE = { mode: "design", param_set: DEFAULT_PARAM_SET, b: 300, cover: 30, link_dia: 8, bar_dia: 20, fck: 30, fyk: 500, v_ed: 120 };

function Slider({ id, label, unit, min, max, step, value, onChange }: { id: string; label: string; unit: string; min: number; max: number; step: number; value: number; onChange: (v: number) => void }) {
  return (
    <div className="lp-sl">
      <label htmlFor={id}>{label}</label>
      <input id={id} type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(Number(e.target.value))} />
      <output htmlFor={id} className="mono">{value} <span>{unit}</span></output>
    </div>
  );
}

export default function LivePreview() {
  const { engine } = useEngine();
  const [m, setM] = useState(200);
  const [h, setH] = useState(500);
  const res = useMemo(() => (engine ? engine.run("beam", { ...BASE, h, m_ed: m }) : null), [engine, m, h]);
  const verdict = verdictOf(res);
  const gov = governing(res);
  return (
    <figure className="lp" aria-label="Live calculation: reinforced concrete beam">
      <div className="lp-bar"><span>RC beam · EN 1992-1-1:2004</span><span className="mono">{res?.ok ? `trace ${res.trace_hash.slice(0, 10)}` : engine ? "" : "loading engine…"}</span></div>
      <div className="lp-grid">
        <div className="lp-vp">{res?.ok && res.drawing ? <SectionDrawing d={res.drawing} /> : <span className="spinner" />}</div>
        <div className="lp-out">
          <div className={`lp-verdict v-${verdict}`}><b className="mono">{VERDICT_WORD[verdict]}</b>{gov && <span className="mono">η<sub>max</sub> {fmtUtil(gov.utilisation)}</span>}</div>
          {res?.ok ? (
            <ul className="lp-checks">
              {res.checks.map((c) => { const st = checkStatus(c); return (
                <li key={c.name}><span className="n">{c.name}</span><b className={`mono t-${st}`}>{fmtUtil(c.utilisation)}</b><UtilizationBar thin value={c.utilisation} status={st} label={`${c.name} utilisation`} /></li>
              ); })}
            </ul>
          ) : res ? <p className="lp-err">{res.errors[0]}</p> : <p className="lp-err">Starting the WebAssembly engine…</p>}
        </div>
      </div>
      <div className="lp-ctl">
        <Slider id="lp-m" label="Design moment M Ed" unit="kNm" min={40} max={420} step={10} value={m} onChange={setM} />
        <Slider id="lp-h" label="Section depth h" unit="mm" min={350} max={800} step={25} value={h} onChange={setH} />
      </div>
      <figcaption>Live — b 300 mm · C30/37 · B500 · V<sub>Ed</sub> 120 kN · computed by the engine in your browser. <Link href="/workspace?m=beam">Open in workbench →</Link></figcaption>
    </figure>
  );
}
