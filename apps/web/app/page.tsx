import Link from "next/link";
import "./home.css";

const RELEASES = "https://github.com/jamesking-23/strucal/releases/latest";

const PLATFORM: [string, string][] = [
  ["Deterministic engine", "Unit-aware computation in Rust. A force can never be added to a moment. Identical inputs, code edition and parameters always return identical results."],
  ["Full traceability", "Every step shows its clause, formula, substituted values and result. Reports are generated from the trace, never reconstructed."],
  ["Independent Check Mode", "Verify the member on the drawing rather than redesigning it: required against provided capacity, utilisation and the governing check."],
  ["National parameter sets", "Nationally determined parameters are controlled data. Missing or unverified sets are flagged, never silently substituted."],
  ["Linked projects", "Pass one calculation's design value to the next input, see what a change affects downstream, and compare against saved revisions."],
  ["Honest scope", "Every module states what it checks and what it does not. A PASS applies to the listed checks only and is never an engineer's approval."],
];

const COVERAGE: { title: string; items: [string, string][] }[] = [
  { title: "Reinforced concrete", items: [["Beams", "Available"], ["One-way slabs", "Available"], ["Columns", "Available"], ["Walls", "Available"], ["Staircases", "Available"], ["Two-way slabs", "Planned"]] },
  { title: "Steel & timber", items: [["Steel beams", "Available"], ["Steel columns", "Available"], ["Bolted connections", "Available"], ["Fillet welds", "Available"], ["Timber beams", "Available"], ["Timber columns", "Available"]] },
  { title: "Foundations & actions", items: [["Pad footings", "Available"], ["Strip footings", "Available"], ["Raft foundations", "Available"], ["Retaining walls", "Available"], ["Load combinations", "Available"], ["Base plates", "Planned"]] },
];

const ROADMAP: [string, string, string][] = [
  ["Stage 0–2", "Unit-safe engine, formula registry, parameter framework, calculation trace, RC beam with Check Mode", "Preview"],
  ["Stage 3", "Slabs, columns, walls, stairs, pad and strip footings", "Preview"],
  ["Stage 4", "Load combinations, steel members and connections, timber, retaining walls, rafts, project dependencies", "Preview"],
  ["Stage 5", "Accounts, organisations, roles, cloud projects, check and approval workflow, tamper-evident audit trail", "Preview"],
  ["Stage 6", "Engineering assistant: explains traces and proposes inputs for confirmation. Never calculates or approves", "Needs API key"],
  ["Stage 7–8", "BIM and analysis integrations, continuous improvement", "Planned"],
];

function Section({ n, id, title, children }: { n: string; id: string; title: string; children: React.ReactNode }) {
  return (
    <section className="sec" id={id} aria-labelledby={`${id}-h`}>
      <div className="wrap g12">
        <div className="sec-label"><span className="n">{n}</span><h2 id={`${id}-h`}>{title}</h2></div>
        <div className="sec-body">{children}</div>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <div className="home">
      <header className="h-nav">
        <div className="wrap">
          <Link href="/" className="h-word" aria-label="StruCal home"><span className="brand" aria-hidden>S</span>StruCal</Link>
          <nav className="h-links" aria-label="Page sections">
            <a href="#platform">Platform</a><a href="#trust">Verification</a><a href="#coverage">Coverage</a><a href="#roadmap">Roadmap</a><a href="#desktop">Desktop</a>
          </nav>
          <div className="h-actions">
            <Link href="/account" className="btn btn-ghost">Sign in</Link>
            <Link href="/workspace" className="btn btn-primary">Open workspace</Link>
          </div>
        </div>
      </header>

      <main>
        <section className="hero">
          <div className="wrap g12">
            <div className="hero-copy">
              <span className="eyebrow">Structural calculation &amp; verification</span>
              <h1>Calculations you can defend to a checking engineer.</h1>
              <p className="lead">Deterministic, fully traceable Eurocode calculations with clause references, substituted values and an independent Check Mode. No black boxes. No invented numbers.</p>
              <div className="cta">
                <Link href="/workspace" className="btn btn-primary">Open the workspace</Link>
                <a href="#desktop" className="btn">Desktop edition</a>
              </div>
            </div>
            <div className="sheet" aria-label="Excerpt of a calculation sheet">
              <div className="sheet-h"><span>RC beam · flexure</span><span>EN 1992-1-1</span></div>
              <div className="sheet-r"><span>cl. 3.1.6(1)P</span><span>f<sub>cd</sub> = 1.0 × 30 / 1.5</span><span>20.00 MPa</span></div>
              <div className="sheet-r"><span>cl. 6.1</span><span>μ = M<sub>Ed</sub> / (b·d²·f<sub>cd</sub>)</span><span>0.1632</span></div>
              <div className="sheet-r"><span>cl. 6.1</span><span>A<sub>s</sub> = M<sub>Ed</sub> / (f<sub>yd</sub>·z)</span><span>1118 mm²</span></div>
              <div className="sheet-r"><span>cl. 9.2.1.1</span><span>A<sub>s,min</sub> check</span><span className="pass">PASS</span></div>
              <div className="sheet-foot">SHA-256 c42598cc… · engine v0.16</div>
            </div>
          </div>
        </section>

        <div className="facts">
          <div className="wrap g12">
            <div><b>1 engine</b><span>Rust, shared by the web and desktop editions</span></div>
            <div><b>3 columns</b><span>Reference, calculation, output on every sheet</span></div>
            <div><b>SHA-256</b><span>Trace hash on every execution</span></div>
            <div><b>0</b><span>Design numbers produced by AI</span></div>
          </div>
        </div>

        <Section n="01" id="platform" title="Platform">
          <p className="intro">One platform that carries a member from inputs to an issued, auditable calculation.</p>
          <div className="items">{PLATFORM.map(([h, p]) => <div className="item" key={h}><h3>{h}</h3><p>{p}</p></div>)}</div>
        </Section>

        <Section n="02" id="trust" title="Verification">
          <p className="intro">AI assists. It never calculates.</p>
          <div className="items two">
            <div className="item"><h3>Release gates</h3><p>Approved specification, verified sources, reviewed formulas, independently derived reference tests, report-trace validation and documented limitations.</p></div>
            <div className="item"><h3>Professional accountability</h3><p>Signatures and approvals belong to authorised human signatories. The software never certifies a design on its own.</p></div>
          </div>
          <div><Link href="/standards" className="btn">Browse the formula registry</Link></div>
        </Section>

        <Section n="03" id="coverage" title="Coverage">
          <div className="cover">
            {COVERAGE.map((c) => (
              <div className="cover-col" key={c.title}>
                <h3>{c.title}</h3>
                <ul>{c.items.map(([name, st]) => <li key={name}><span>{name}</span><span className={`badge ${st === "Available" ? "ok" : ""}`}>{st}</span></li>)}</ul>
              </div>
            ))}
          </div>
        </Section>

        <Section n="04" id="roadmap" title="Roadmap">
          <div className="road tbl-wrap">
            <table className="tbl">
              <thead><tr><th>Stage</th><th>Scope</th><th className="r">Status</th></tr></thead>
              <tbody>{ROADMAP.map(([s, d, st]) => <tr key={s}><td>{s}</td><td>{d}</td><td className="r"><span className={`badge ${st === "Preview" ? "ok" : st === "Planned" ? "" : "warn"}`}>{st}</span></td></tr>)}</tbody>
            </table>
          </div>
        </Section>

        <Section n="05" id="desktop" title="Desktop">
          <p className="intro">A native Windows application on the same engine. No browser inside, and it updates itself from Settings.</p>
          <div className="dl"><a className="btn btn-primary" href={RELEASES}>Download for Windows</a><Link className="btn" href="/workspace">Use the web version</Link></div>
        </Section>
      </main>

      <footer className="h-foot">
        <div className="wrap"><span><b>StruCal</b> · Engineering results require review by a registered engineer.</span><span>Preview build. Not for issue on live projects.</span></div>
      </footer>
    </div>
  );
}
