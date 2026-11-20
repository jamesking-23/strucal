import Link from "next/link";

const REL = "https://github.com/jamesking-23/strucal/releases/latest";

export default function Home() {
  return (
    <>
      <header className="nav"><div className="wrap">
        <div className="brand" aria-label="StruCal"><span className="mark">S</span></div>
        <nav><a href="#platform">Platform</a><a href="#trust">Verification</a><a href="#scope">Coverage</a><a href="#roadmap">Roadmap</a><a href="#desktop">Desktop</a></nav>
        <Link href="/account" className="btn ghost" style={{marginRight:10}}>Sign in</Link><Link href="/workspace" className="btn gold">Open workspace</Link>
      </div></header>

      <section className="hero"><div className="wrap">
        <div>
          <div className="eyebrow">Structural calculation &amp; verification platform</div>
          <h1>Calculations you can defend in front of a checking engineer.</h1>
          <p className="lead">StruCal produces deterministic, fully traceable Eurocode calculations with clause references, substituted values and independent Check Mode. No black boxes. No invented numbers.</p>
          <div className="cta"><Link href="/workspace" className="btn gold">Open the workspace</Link><a href="#desktop" className="btn ghost">Get the desktop edition</a></div>
        </div>
        <div className="sheet" aria-label="Sample calculation sheet">
          <div className="h"><span>RC BEAM - FLEXURE</span><span>EN 1992-1-1</span></div>
          <div className="r th"><div>Reference</div><div>Calculation</div><div>Output</div></div>
          <div className="r"><div>cl. 3.1.6(1)P</div><div>f<sub>cd</sub> = 1.0 x 30 / 1.5</div><div>20.00 MPa</div></div>
          <div className="r"><div>cl. 6.1</div><div>mu = M<sub>Ed</sub> / (b d<sup>2</sup> f<sub>cd</sub>)</div><div>0.1632</div></div>
          <div className="r"><div>cl. 6.1</div><div>As = M<sub>Ed</sub> / (f<sub>yd</sub> z)</div><div>1118 mm<sup>2</sup></div></div>
          <div className="r"><div>cl. 9.2.1.1</div><div>As,min check</div><div className="pass">PASS</div></div>
        </div>
      </div></section>

      <div className="strip"><div className="wrap">
        <div><b>100%</b><span>of numbers from a deterministic engine</span></div>
        <div><b>3-column</b><span>Reference / Calculation / Output sheets</span></div>
        <div><b>SHA-256</b><span>trace hash on every execution</span></div>
        <div><b>Web + Desktop</b><span>one engine, two native experiences</span></div>
      </div></div>

      <section className="s" id="platform"><div className="wrap">
        <div className="sh"><h2>Built for design offices, checkers and approvers</h2><p>One platform that carries a member from inputs to an issued, auditable calculation report.</p></div>
        <div className="grid3">
          <div className="card"><h3>Deterministic engine</h3><p>Unit-aware computation. A force can never be added to a moment. Identical inputs, code edition and parameters always return identical results.</p></div>
          <div className="card"><h3>Full traceability</h3><p>Every step shows its clause, formula, substituted values and result. Reports are generated from the trace, never reconstructed.</p></div>
          <div className="card"><h3>Independent Check Mode</h3><p>Verify the member on the drawing rather than redesigning it. Required vs provided capacity, utilisation and the governing check.</p></div>
          <div className="card"><h3>National parameter packs</h3><p>Nationally determined parameters are controlled data. Missing or unverified parameters are flagged, never silently substituted.</p></div>
          <div className="card"><h3>Code-edition aware</h3><p>Designed for coexisting first and second generation Eurocodes, with controlled migration rather than one hard-coded edition.</p></div>
          <div className="card"><h3>Honest scope</h3><p>Every module states what it checks and what it does not. A PASS applies to the listed checks only, and is never an engineer's approval.</p></div>
        </div>
      </div></section>

      <section className="s alt" id="trust"><div className="wrap">
        <div className="sh"><h2>AI assists. It never calculates.</h2><p>Language models help prepare inputs and explain results. All design numbers come from the deterministic engine, and AI-written code must clear verification gates before release.</p></div>
        <div className="grid2">
          <div className="card"><h3>Release gates</h3><p>Approved specification, verified sources, reviewed formulas, independently derived reference tests, report-trace validation and documented limitations.</p></div>
          <div className="card"><h3>Professional accountability</h3><p>Signatures and approvals belong to authorised human signatories. The software never certifies a design on its own.</p></div>
        </div>
      </div></section>

      <section className="s" id="scope"><div className="wrap">
        <div className="sh"><h2>Coverage</h2><p>Reinforced concrete, steel, timber and foundations, delivered module by module as each clears verification.</p></div>
        <div className="grid3">
          <div className="card"><h3>Reinforced concrete</h3><p>Beams, slabs, columns, walls, stairs, footings, pile caps, retaining walls.</p></div>
          <div className="card"><h3>Structural steel</h3><p>Beams, columns, beam-columns, bolted and welded connections, and the project dependency engine, base plates.</p></div>
          <div className="card"><h3>Timber &amp; foundations</h3><p>Joists, rafters, trusses, bearing, sliding, overturning and settlement.</p></div>
        </div>
      </div></section>

      <section className="s alt" id="roadmap"><div className="wrap">
        <div className="sh"><h2>Where we are</h2><p>Transparent status. Nothing is shown as available until it exists.</p></div>
        <div className="road">
          <div><b>Stage 0-1</b><span>Repository, unit-safe engine, formula registry, parameter framework, calculation trace</span><span className="pill live">In preview</span></div>
          <div><b>Stage 2</b><span>RC beam: flexure, shear, min/max steel, Check Mode, calculation sheet</span><span className="pill live">In preview</span></div>
          <div><b>Stage 3</b><span>Slabs, columns, walls, stairs, pad and strip footings</span><span className="pill next">In progress</span></div>
          <div><b>Stage 5</b><span>Accounts, organisations, roles, cloud projects, check and approval workflow, tamper-evident audit trail (web and desktop)</span><span className="pill next">In preview</span></div>
          <div><b>Stage 6</b><span>Engineering assistant: explains traces, proposes inputs for confirmation, summarises revision changes. Never calculates or approves</span><span className="pill next">Built, needs API key</span></div>
          <div><b>Stage 7-8</b><span>BIM and analysis integrations, continuous improvement</span><span className="pill plan">Planned</span></div>
        </div>
      </div></section>

      <section className="s" id="desktop"><div className="wrap">
        <div className="sh"><h2>Desktop edition for Windows</h2><p>A native application built on the same engine, with the same capabilities. It does not run in a browser, and it updates itself from Settings with a single Check updates action.</p></div>
        <div className="cta"><a className="btn dark" href={REL}>Download for Windows</a><Link className="btn line" href="/workspace">Use the web version</Link></div>
      </div></section>

      <footer><div className="wrap"><div><b style={{color:"#fff",letterSpacing:".14em"}}>STRUCAL</b><br/>Engineering results require review by a registered engineer.</div><div>Preview build. Not for issue on live projects.</div></div></footer>
    </>
  );
}
