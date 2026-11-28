import Link from "next/link";
import SectionDrawing from "./workspace/SectionDrawing";
import { UtilBar } from "./ui/primitives";
import { DISCIPLINES } from "../lib/form";
import { REVIEW_THRESHOLD } from "../lib/status";
import "./home.css";

const RELEASES = "https://github.com/jamesking-23/strucal/releases/latest";

// Example shown in the hero: the RC beam module on its default inputs, exactly as the engine returns it
// (engine v0.16.0, trace c42598cc9d59). Static so the landing page needs no WebAssembly.
const EXAMPLE_CHECKS: [string, string, string, string, number][] = [
  ["Flexural resistance", "6.1", "200.0 kNm", "222.1 kNm", 0.9],
  ["Minimum reinforcement", "9.2.1.1(1)", "204 mm²", "1257 mm²", 0.16],
  ["Maximum reinforcement", "9.2.1.1(3)", "1257 mm²", "6000 mm²", 0.21],
  ["Bar spacing", "8.2(2)", "25 mm min.", "48 mm clear", 0.52],
  ["Shear: compression strut", "6.2.3 Eq 6.9", "120.0 kN", "444.4 kN", 0.27],
  ["Shear reinforcement", "6.2.3 Eq 6.8", "0.271 mm²/mm", "0.309 mm²/mm", 0.88],
  ["Link spacing limit", "9.2.2(6)", "325 mm", "339 mm max", 0.96],
];

// Module coverage as registered in the engine (crates/engine/src/modules.rs).
const MODULES: { discipline: string; name: string; standard: string; check: boolean }[] = [
  { discipline: "Actions", name: "Load combinations", standard: "EN 1990:2002", check: false },
  { discipline: "Reinforced concrete", name: "RC beam", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Reinforced concrete", name: "RC column", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Reinforced concrete", name: "RC wall", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Reinforced concrete", name: "One-way slab", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Reinforced concrete", name: "Staircase", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Structural steel", name: "Steel beam", standard: "EN 1993-1-1:2005", check: false },
  { discipline: "Structural steel", name: "Steel column", standard: "EN 1993-1-1:2005", check: false },
  { discipline: "Structural steel", name: "Bolted connection", standard: "EN 1993-1-8:2005", check: false },
  { discipline: "Structural steel", name: "Fillet weld", standard: "EN 1993-1-8:2005", check: false },
  { discipline: "Timber", name: "Timber beam", standard: "Eurocode 5 (UK NA)", check: false },
  { discipline: "Timber", name: "Timber column", standard: "Eurocode 5 (UK NA)", check: false },
  { discipline: "Foundations & geotechnics", name: "Pad footing", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Foundations & geotechnics", name: "Strip footing", standard: "EN 1992-1-1:2004", check: true },
  { discipline: "Foundations & geotechnics", name: "Raft foundation", standard: "EN 1992-1-1:2004", check: false },
  { discipline: "Foundations & geotechnics", name: "Retaining wall", standard: "EN 1997-1 / EN 1992-1-1", check: false },
];

const FLOW: [string, string, string][] = [
  ["01", "Parameters", "Geometry, materials and design actions in a property grid with units, defaults and field-level validation. A parameter set fixes the partial factors; unverified sets are flagged."],
  ["02", "Section", "A dimensioned, proportional schematic of the member as defined, redrawn on every change. The drawing reflects inputs; it never produces results."],
  ["03", "Calculation sheet", "Every step with its clause, formula, substituted values and output, grouped by limit state and reproducible to the byte by its SHA-256 trace hash."],
  ["04", "Verification", "Each implemented check as E_d / R_d with utilisation, the governing limit state, observations and an explicit statement of scope."],
];

const PRINCIPLES: [string, string][] = [
  ["Deterministic engine", "One Rust engine, compiled natively for the desktop and to WebAssembly for the browser. Identical inputs, code edition and parameters always return identical results."],
  ["Dimensionally typed", "Quantities carry units through the calculation. A force cannot be added to a moment; a mismatch is an error, not a number."],
  ["Independent Check Mode", "Verify the member as drawn — provided reinforcement, sizes and spacing — instead of redesigning it."],
  ["No invented values", "National parameters are controlled data. Out-of-range or physically meaningless inputs are rejected with a reason."],
  ["AI never calculates", "The optional assistant explains a finished trace or proposes inputs for confirmation. It cannot change a result or approve a design."],
  ["Human sign-off", "Revisions move from designer to checker to approver, each a different person, with a tamper-evident audit trail."],
];

const tone = (u: number) => (u > 1 ? "fail" : u > REVIEW_THRESHOLD ? "warn" : "ok");

function Sec({ n, id, title, children }: { n: string; id: string; title: string; children: React.ReactNode }) {
  return (
    <section className="h-sec" id={id} aria-labelledby={`${id}-h`}>
      <div className="wrap h-sec-grid">
        <header className="h-sec-label"><span className="mono">{n}</span><h2 id={`${id}-h`}>{title}</h2></header>
        <div className="h-sec-body">{children}</div>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <div className="home">
      <header className="h-nav">
        <div className="wrap h-nav-row">
          <Link href="/" className="wordmark" aria-label="StruCal home"><span className="brand" aria-hidden>S</span>StruCal</Link>
          <nav className="h-links" aria-label="Page sections">
            <a href="#workbench">Workbench</a><a href="#principles">Verification</a><a href="#coverage">Coverage</a><a href="#desktop">Desktop</a>
          </nav>
          <div className="h-actions">
            <Link href="/standards" className="btn btn-ghost">Formula registry</Link>
            <Link href="/account" className="btn btn-ghost">Sign in</Link>
            <Link href="/workspace" className="btn btn-primary">Open workbench</Link>
          </div>
        </div>
      </header>

      <main>
        <section className="h-hero">
          <div className="wrap h-hero-grid">
            <div className="h-hero-copy">
              <span className="eyebrow">Structural calculation and verification</span>
              <h1>Eurocode calculations a checking engineer can follow line by line.</h1>
              <p className="lead">StruCal is a deterministic calculation instrument for structural engineers: dimensioned sections, clause-referenced working and explicit limit-state verification, from the same engine on the web and on the desktop.</p>
              <div className="h-cta">
                <Link href="/workspace" className="btn btn-primary btn-lg">Open the workbench</Link>
                <a href={RELEASES} className="btn btn-lg">Windows desktop edition</a>
              </div>
              <dl className="h-spec">
                <div><dt>Modules</dt><dd>{MODULES.length}</dd></div>
                <div><dt>Standards</dt><dd>EN 1990 · 1992 · 1993 · 1995 · 1997</dd></div>
                <div><dt>Trace</dt><dd>SHA-256 per run</dd></div>
              </dl>
            </div>

            <figure className="h-panel" aria-label="Example: RC beam on default inputs">
              <div className="h-panel-bar"><span>RC beam · EN 1992-1-1:2004</span><span className="mono">trace c42598cc</span></div>
              <div className="h-panel-grid">
                <div className="h-panel-vp"><SectionDrawing d={{ kind: "Beam", b: 300, h: 500, cover: 30, link_dia: 8, bar_dia: 20, n_bars: 4 }} /></div>
                <div className="h-panel-checks">
                  <div className="h-verdict"><b>PASS</b><span className="mono">η<sub>max</sub> 0.96</span></div>
                  <ul>
                    {EXAMPLE_CHECKS.map(([name, cl, ed, rd, u]) => (
                      <li key={name}>
                        <div className="r1"><span>{name}</span><b className={`mono t-${tone(u)}`}>{u.toFixed(2)}</b></div>
                        <UtilBar value={u} status={tone(u)} thin label={`${name} utilisation`} />
                        <div className="r2 mono"><span>cl. {cl}</span><span>{ed} / {rd}</span></div>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
              <figcaption>Example output: RC beam module on its default inputs (b 300, h 500, C30/37, M<sub>Ed</sub> 200 kNm, V<sub>Ed</sub> 120 kN).</figcaption>
            </figure>
          </div>
        </section>

        <Sec n="01" id="workbench" title="The workbench">
          <p className="intro">Four panes, one flow: from parameters to a verified, reproducible calculation sheet.</p>
          <ol className="h-flow">{FLOW.map(([n, h, p]) => <li key={n}><span className="mono">{n}</span><h3>{h}</h3><p>{p}</p></li>)}</ol>
          <div><Link href="/workspace" className="btn btn-primary">Open the workbench</Link></div>
        </Sec>

        <Sec n="02" id="principles" title="Verification">
          <p className="intro">Results you can defend: every number comes from the engine, and every check states its scope.</p>
          <div className="h-principles">{PRINCIPLES.map(([h, p]) => <div key={h}><h3>{h}</h3><p>{p}</p></div>)}</div>
          <div className="row-actions"><Link href="/standards" className="btn">Formula registry and parameter sets</Link></div>
        </Sec>

        <Sec n="03" id="coverage" title="Coverage">
          <p className="intro">{MODULES.length} modules in the current engine. A PASS applies to the implemented checks listed on each sheet.</p>
          <div className="tbl-wrap">
            <table className="tbl h-cov">
              <thead><tr><th>Discipline</th><th>Module</th><th>Standard</th><th>Design</th><th>Check Mode</th></tr></thead>
              <tbody>
                {DISCIPLINES.flatMap((g) => MODULES.filter((m) => m.discipline === g).map((m, i) => (
                  <tr key={m.name} className={i === 0 ? "first" : undefined}>
                    <td className="disc">{i === 0 ? g : ""}</td>
                    <td>{m.name}</td>
                    <td className="m muted">{m.standard}</td>
                    <td><span className="badge ok">Yes</span></td>
                    <td>{m.check ? <span className="badge ok">Yes</span> : <span className="faint m">—</span>}</td>
                  </tr>
                )))}
              </tbody>
            </table>
          </div>
          <p className="h-fine">Planned: two-way slabs (awaiting a verified method source), base plates, beam-columns.</p>
        </Sec>

        <Sec n="04" id="desktop" title="Desktop">
          <p className="intro">A native Windows application on the same engine. No browser inside; it updates itself from Settings.</p>
          <div className="row-actions"><a className="btn btn-primary btn-lg" href={RELEASES}>Download for Windows</a><Link className="btn btn-lg" href="/workspace">Use the web workbench</Link></div>
        </Sec>
      </main>

      <footer className="h-foot">
        <div className="wrap h-foot-row"><span><b>StruCal</b> · Engineering results require review by a registered engineer.</span><span className="mono">Preview build · not for issue on live projects</span></div>
      </footer>
    </div>
  );
}
