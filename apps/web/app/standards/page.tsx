"use client";
import { useMemo, useState } from "react";
import Link from "next/link";
import { useEngine } from "../../lib/useEngine";
import "../workspace/workspace.css";
import "../project/project.css";

export default function Standards() {
  const { engine: eng, error, retry } = useEngine();
  const [q, setQ] = useState("");
  const rows = useMemo(() => {
    const t = q.trim().toLowerCase();
    return (eng?.formulas ?? []).filter((f) => !t || [f.id, f.name, f.standard, f.clause, f.verification].some((x) => x.toLowerCase().includes(t)));
  }, [eng, q]);

  return (
    <div className="pj">
      <div className="wk-title"><Link href="/" className="wk-logo" aria-label="StruCal home">S</Link><div className="wk-wt">StruCal — Standards and formula registry</div></div>
      <div className="wk-tabs"><span className="file">STANDARDS</span><Link href="/workspace" style={{ padding: "0 14px", lineHeight: "24px" }}>Workspace</Link><Link href="/project" style={{ padding: "0 14px", lineHeight: "24px" }}>Project</Link></div>
      <div style={{ padding: 8, display: "grid", gap: 8, maxWidth: 1100, width: "100%", margin: "0 auto" }}>
        {!eng && (error ? <div className="pj-err">{error} <button className="pj-btn" onClick={retry}>Try again</button></div> : <div className="pj-empty">Loading calculation engine…</div>)}
        {eng && (<>
          <div className="pj-card">
            <div className="pj-ch"><b>Parameter sets</b><span className="pj-tag">{eng.paramSets.length}</span></div>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}><thead><tr><th align="left">Set</th><th>γc</th><th>γs</th><th>αcc</th><th>γG</th><th>γQ</th><th align="left">Status</th></tr></thead><tbody>
              {eng.paramSets.map((p) => <tr key={p.id} style={{ borderTop: "1px solid #e4e7eb" }}><td style={{ padding: "6px 10px" }}><b>{p.label}</b><div style={{ color: "#52606d" }}>{p.source}</div></td><td align="center">{p.gamma_c}</td><td align="center">{p.gamma_s}</td><td align="center">{p.alpha_cc}</td><td align="center">{p.gamma_g}</td><td align="center">{p.gamma_q}</td><td>{p.status}</td></tr>)}
            </tbody></table>
          </div>
          <div className="pj-card">
            <div className="pj-ch"><b>Formula registry</b><span className="pj-tag">{rows.length} of {eng.formulas.length}</span><span style={{ flex: 1 }} /><input className="fac" style={{ width: 240, textAlign: "left", height: 26 }} placeholder="Filter by id, clause or status" aria-label="Filter formulas" value={q} onChange={(e) => setQ(e.target.value)} /></div>
            {rows.length === 0 ? <div className="pj-empty">No formulas match “{q}”.</div> : (
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}><thead><tr><th align="left">ID</th><th align="left">Name</th><th align="left">Standard</th><th align="left">Clause</th><th align="left">Verification</th></tr></thead><tbody>
                {rows.map((f) => <tr key={f.id} style={{ borderTop: "1px solid #e4e7eb" }}><td style={{ padding: "6px 10px", fontFamily: "IBM Plex Mono, monospace" }}>{f.id}</td><td>{f.name}</td><td>{f.standard}</td><td>{f.clause}</td><td>{f.verification}</td></tr>)}
              </tbody></table>)}
          </div>
          <div className="pj-warn">Engine v{eng.version}. Verification states are shown exactly as recorded in the engine; see docs/verification/VERIFICATION_REGISTER.md for reference cases.</div>
        </>)}
      </div>
    </div>
  );
}
