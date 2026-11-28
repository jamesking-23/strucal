import Link from "next/link";
import LivePreview from "./LivePreview";
import "./landing.css";

const RELEASES = "https://github.com/jamesking-23/strucal/releases/latest";

const MODULES: [string, string, string, boolean][] = [
  ["Actions", "Load combinations", "EN 1990", false],
  ["Concrete", "RC beam", "EN 1992-1-1", true], ["Concrete", "RC column", "EN 1992-1-1", true], ["Concrete", "RC wall", "EN 1992-1-1", true],
  ["Concrete", "One-way slab", "EN 1992-1-1", true], ["Concrete", "Staircase", "EN 1992-1-1", true], ["Concrete", "Punching shear", "EN 1992-1-1", false],
  ["Steel", "Steel beam", "EN 1993-1-1", false], ["Steel", "Steel column", "EN 1993-1-1", false], ["Steel", "Steel beam-column", "EN 1993-1-1", false], ["Steel", "Steel tension member", "EN 1993-1-1", false], ["Steel", "Bolted connection", "EN 1993-1-8", false], ["Steel", "Fillet weld", "EN 1993-1-8", false],
  ["Timber", "Timber beam", "EN 1995-1-1", false], ["Timber", "Timber column", "EN 1995-1-1", false], ["Timber", "Timber tension member", "EN 1995-1-1", false],
  ["Masonry", "Masonry wall", "EN 1996-1-1", false],
  ["Foundations", "Pad footing", "EN 1992-1-1", true], ["Foundations", "Strip footing", "EN 1992-1-1", true], ["Foundations", "Raft foundation", "EN 1992-1-1", false], ["Foundations", "Retaining wall", "EN 1997-1", false],
];

export default function Home() {
  return (
    <div className="ld">
      <header className="ld-top">
        <Link href="/" className="ld-mark" aria-label="StruCal"><i>S</i><span>StruCal</span></Link>
        <nav aria-label="Sections"><a href="#engine">Engine</a><a href="#workbench">Workbench</a><a href="#coverage">Coverage</a><a href="#desktop">Desktop</a></nav>
        <div className="ld-top-r"><Link href="/account" className="btn ghost sm">Sign in</Link><Link href="/workspace" className="btn primary sm">Launch Workbench</Link></div>
      </header>

      <section className="ld-hero">
        <div className="ld-hero-l">
          <p className="eyebrow">Structural calculation · verification · v0.17</p>
          <h1>Deterministic<br />structural<br />verification.</h1>
          <p className="ld-lead">A Rust calculation engine with clause-referenced, auditable working — the same bytes in your browser through WebAssembly and on your desktop as a native application.</p>
          <div className="ld-cta">
            <Link href="/workspace" className="btn primary lg">Launch Workbench</Link>
            <a href={RELEASES} className="btn lg">Windows desktop</a>
          </div>
          <dl className="ld-spec">
            <div><dt>Modules</dt><dd className="mono">21</dd></div>
            <div><dt>Eurocodes</dt><dd className="mono">EN 1990 · 1992 · 1993 · 1995 · 1996 · 1997</dd></div>
            <div><dt>Reproducibility</dt><dd className="mono">SHA-256 trace per run</dd></div>
          </dl>
        </div>
        <div className="ld-hero-r"><LivePreview /></div>
      </section>

      <section className="ld-sec" id="engine">
        <header className="ld-sec-h"><span className="mono">01</span><h2>One engine. Two native surfaces.</h2><p>Every number on every screen comes from a single deterministic Rust crate. The browser and the desktop do not reimplement calculations; they render the engine&apos;s trace.</p></header>
        <div className="ld-arch" role="img" aria-label="Architecture: Rust engine compiled to WebAssembly for the web workbench and natively for the desktop application">
          <div className="arch-node core"><span className="mono">crates/engine</span><b>Calculation engine</b><ul><li>Rust · dimensionally typed quantities</li><li>21 Eurocode modules · formula registry</li><li>Trace: clause · formula · substitution · result</li><li>SHA-256 hash of the ordered trace</li></ul></div>
          <div className="arch-links" aria-hidden><span /><span /></div>
          <div className="arch-col">
            <div className="arch-node"><span className="mono">crates/wasm → apps/web</span><b>Web workbench</b><ul><li>wasm-bindgen · runs in the browser</li><li>No server round-trip for calculations</li></ul></div>
            <div className="arch-node"><span className="mono">apps/desktop</span><b>Desktop application</b><ul><li>Native egui · no webview</li><li>Same engine linked directly</li></ul></div>
          </div>
        </div>
        <div className="ld-facts">
          <div><b className="mono">0</b><span>design values produced by AI — the assistant only explains a finished trace</span></div>
          <div><b className="mono">1:1</b><span>identical inputs, code edition and parameters return identical results and hash</span></div>
          <div><b className="mono">N·mm</b><span>unit-typed arithmetic: a force cannot be added to a moment</span></div>
        </div>
      </section>

      <section className="ld-sec" id="workbench">
        <header className="ld-sec-h"><span className="mono">02</span><h2>An engineering workbench, not a form.</h2><p>Parameters on the left, the member in the middle, the verification ledger below and the verdict on the right — the layout of a checking engineer&apos;s desk.</p></header>
        <ol className="ld-panes">
          <li><span className="mono">A</span><b>Inspector</b><p>Grouped parameters with inline units, instant validation and presets for concrete class, steel grade, rolled sections and ψ factors.</p></li>
          <li><span className="mono">B</span><b>Viewport</b><p>A dimensioned cross-section and a loading diagram showing supports and actions — pan and zoom on a drafting grid.</p></li>
          <li><span className="mono">C</span><b>Ledger</b><p>Every step with its clause, governing equation, substituted values and result, grouped by limit state.</p></li>
          <li><span className="mono">D</span><b>Verdict</b><p>Utilisation per check — emerald below 0.85, amber at or above 0.85, crimson when overstressed — and the governing limit state.</p></li>
        </ol>
        <Link href="/workspace" className="btn primary">Launch Workbench</Link>
      </section>

      <section className="ld-sec" id="coverage">
        <header className="ld-sec-h"><span className="mono">03</span><h2>Coverage</h2><p>Twenty-one modules in the current engine. A PASS applies only to the implemented checks listed on each ledger.</p></header>
        <div className="tbl-wrap">
          <table className="tbl ld-cov">
            <thead><tr><th>Discipline</th><th>Module</th><th>Standard</th><th>Design</th><th>Check Mode</th></tr></thead>
            <tbody>{MODULES.map(([d, n, s, c]) => <tr key={n}><td className="muted">{d}</td><td>{n}</td><td className="m">{s}</td><td><span className="badge pass">yes</span></td><td>{c ? <span className="badge pass">yes</span> : <span className="faint m">—</span>}</td></tr>)}</tbody>
          </table>
        </div>
      </section>

      <section className="ld-sec" id="desktop">
        <header className="ld-sec-h"><span className="mono">04</span><h2>Desktop edition</h2><p>Native Windows application on the same engine. Signs in once and stays signed in; updates itself from Settings.</p></header>
        <div className="ld-cta"><a className="btn primary lg" href={RELEASES}>Download for Windows</a><Link className="btn lg" href="/standards">Formula registry</Link></div>
      </section>

      <footer className="ld-foot"><span><b>StruCal</b> · Engineering results require review by a registered engineer.</span><span className="mono">preview build · not for issue on live projects</span></footer>
    </div>
  );
}
