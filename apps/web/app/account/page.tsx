"use client";
import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { supabase, ROLES, ROLE_LABEL, STATUS_LABEL } from "../../lib/supabase";
import "../workspace/workspace.css";
import "../project/project.css";

const msg = (e: any) => (e?.message ?? String(e)).replace(/^.*?(ERROR:\s*)?/, "");

export default function Account() {
  const [session, setSession] = useState<any>(null);
  const [ready, setReady] = useState(false);
  const [mode, setMode] = useState<"in" | "up">("in");
  const [email, setEmail] = useState(""); const [pw, setPw] = useState(""); const [dn, setDn] = useState("");
  const [note, setNote] = useState(""); const [err, setErr] = useState("");
  const [orgs, setOrgs] = useState<any[]>([]); const [org, setOrg] = useState<string>("");
  const [members, setMembers] = useState<any[]>([]); const [projects, setProjects] = useState<any[]>([]);
  const [proj, setProj] = useState<string>(""); const [revs, setRevs] = useState<any[]>([]); const [reviews, setReviews] = useState<any[]>([]);
  const [audit, setAudit] = useState<any[]>([]); const [chain, setChain] = useState<string>("");
  const [newOrg, setNewOrg] = useState(""); const [newProj, setNewProj] = useState(""); const [addEmail, setAddEmail] = useState(""); const [addRole, setAddRole] = useState("designer");
  const [comment, setComment] = useState(""); const [aal, setAal] = useState<any>(null); const [stepCode, setStepCode] = useState(""); const [factors, setFactors] = useState<any[]>([]); const [enroll, setEnroll] = useState<any>(null); const [code, setCode] = useState("");
  const uid = session?.user?.id;
  const myRole = orgs.find((o) => o.id === org)?.role as string | undefined;
  const isAdmin = myRole === "owner" || myRole === "administrator";
  const canAudit = isAdmin || myRole === "auditor";
  const names: Record<string, string> = Object.fromEntries(members.map((m) => [m.user_id, m.display_name || m.email]));

  useEffect(() => { supabase.auth.getSession().then(({ data }) => { setSession(data.session); setReady(true); }); const { data: sub } = supabase.auth.onAuthStateChange((_e, s) => setSession(s)); return () => sub.subscription.unsubscribe(); }, []);

  const run = async (fn: () => Promise<any>, ok = "") => { setErr(""); setNote(""); try { const r = await fn(); if (r?.error) throw r.error; if (ok) setNote(ok); return r; } catch (e) { setErr(msg(e)); return null; } };

  const loadOrgs = useCallback(async () => {
    if (!uid) return;
    const { data } = await supabase.from("memberships").select("role, organisations(id,name,require_mfa_approval)").eq("user_id", uid);
    const list = (data ?? []).map((m: any) => ({ id: m.organisations.id, name: m.organisations.name, role: m.role, mfa: m.organisations.require_mfa_approval }));
    setOrgs(list); setOrg((cur) => (cur && list.some((o) => o.id === cur) ? cur : list[0]?.id ?? ""));
  }, [uid]);
  const loadOrg = useCallback(async () => {
    if (!org) { setMembers([]); setProjects([]); return; }
    const { data: ms } = await supabase.from("memberships").select("user_id, role").eq("org_id", org);
    const ids = (ms ?? []).map((m: any) => m.user_id);
    const { data: ps } = ids.length ? await supabase.from("profiles").select("id, display_name, email").in("id", ids) : { data: [] as any[] };
    setMembers((ms ?? []).map((m: any) => { const p = (ps ?? []).find((x: any) => x.id === m.user_id); return { ...m, display_name: p?.display_name ?? "", email: p?.email ?? "" }; }));
    const { data: pr } = await supabase.from("projects").select("id, name, description, created_at").eq("org_id", org).order("created_at", { ascending: false });
    setProjects(pr ?? []); setProj((c) => (c && (pr ?? []).some((p: any) => p.id === c) ? c : pr?.[0]?.id ?? ""));
    if (canAudit) { const { data: au } = await supabase.from("audit_events").select("id, actor, action, entity, detail, created_at").eq("org_id", org).order("id", { ascending: false }).limit(40); setAudit(au ?? []); } else setAudit([]);
  }, [org, canAudit]);
  const loadRevs = useCallback(async () => {
    if (!proj) { setRevs([]); setReviews([]); return; }
    const { data } = await supabase.from("revisions").select("id, number, name, status, created_by, created_at, content_hash, engine_version").eq("project_id", proj).order("number", { ascending: false });
    setRevs(data ?? []);
    const ids = (data ?? []).map((r: any) => r.id);
    const { data: rv } = ids.length ? await supabase.from("reviews").select("id, revision_id, reviewer, kind, comment, created_at").in("revision_id", ids).order("created_at") : { data: [] as any[] };
    setReviews(rv ?? []);
  }, [proj]);
  const loadFactors = useCallback(async () => { const { data } = await supabase.auth.mfa.listFactors(); setFactors(data?.totp ?? []); }, []);
  const loadAal = useCallback(async () => { const { data } = await supabase.auth.mfa.getAuthenticatorAssuranceLevel(); setAal(data); }, []);
  useEffect(() => { loadOrgs(); loadFactors(); loadAal(); }, [loadOrgs, loadFactors, loadAal, uid]);
  const needStepUp = aal && aal.nextLevel === "aal2" && aal.currentLevel !== "aal2";
  const stepUp = () => run(async () => { const f = factors.find((x) => x.status === "verified"); if (!f) return { error: { message: "No authenticator app is set up" } }; const ch = await supabase.auth.mfa.challenge({ factorId: f.id }); if (ch.error) return ch; const v = await supabase.auth.mfa.verify({ factorId: f.id, challengeId: ch.data.id, code: stepCode }); if (!v.error) { setStepCode(""); await loadAal(); } return v; }, "Two-factor verified for this session.");
  useEffect(() => { loadOrg(); }, [loadOrg]);
  useEffect(() => { loadRevs(); }, [loadRevs]);

  const signIn = () => run(async () => { const r = mode === "in" ? await supabase.auth.signInWithPassword({ email, password: pw }) : await supabase.auth.signUp({ email, password: pw, options: { data: { display_name: dn } } }); if (!r.error && mode === "up" && !r.data.session) { const r2 = await supabase.auth.signInWithPassword({ email, password: pw }); if (r2.error) setNote("Account created. If email confirmation is switched on for this project, click the link in the email, then sign in."); return r2.error ? { error: null } : r2; } return r; });
  const rpc = (name: string, args: any, ok: string, after: () => void) => run(async () => { const r = await supabase.rpc(name, args); if (!r.error) after(); return r; }, ok);

  if (!ready) return <div className="pj"><div className="pj-empty">Loading...</div></div>;
  return (
    <div className="pj">
      <div className="wk-title"><Link href="/" className="wk-logo">S</Link><div className="wk-wt">StruCal — Account and teams</div></div>
      <div className="wk-tabs"><span className="file">ACCOUNT</span><Link href="/workspace" style={{ padding: "0 14px", lineHeight: "24px" }}>Workspace</Link><Link href="/project" style={{ padding: "0 14px", lineHeight: "24px" }}>Project</Link></div>
      <div style={{ padding: 8, display: "grid", gap: 8, maxWidth: 1100, width: "100%", margin: "0 auto" }}>
        {err && <div className="pj-err">{err}</div>}{note && <div className="pj-warn">{note}</div>}
        {!session && (
          <div className="pj-card" style={{ maxWidth: 420 }}>
            <div className="pj-ch"><b>{mode === "in" ? "Sign in" : "Create account"}</b><span style={{ flex: 1 }} /><button className="pj-btn" onClick={() => { setMode(mode === "in" ? "up" : "in"); setErr(""); setNote(""); }}>{mode === "in" ? "Need an account?" : "Have an account?"}</button></div>
            <div className="pj-grid" style={{ gridTemplateColumns: "1fr" }}>
              {mode === "up" && <div className="pj-f"><label>Display name</label><input className="fac" style={{ width: "100%", textAlign: "left", height: 28 }} value={dn} onChange={(e) => setDn(e.target.value)} /></div>}
              <div className="pj-f"><label>Email</label><input className="fac" type="email" style={{ width: "100%", textAlign: "left", height: 28 }} value={email} onChange={(e) => setEmail(e.target.value)} /></div>
              <div className="pj-f"><label>Password (at least 8 characters)</label><input className="fac" type="password" style={{ width: "100%", textAlign: "left", height: 28 }} value={pw} onChange={(e) => setPw(e.target.value)} onKeyDown={(e) => e.key === "Enter" && signIn()} /></div>
              <button className="pj-btn pri" onClick={signIn} style={{ height: 30 }}>{mode === "in" ? "Sign in" : "Create account"}</button>
            </div>
          </div>
        )}
        {session && (<>
          <div className="pj-card"><div className="pj-ch"><b>{session.user.user_metadata?.display_name || session.user.email}</b><span className="pj-tag">{session.user.email}</span><span style={{ flex: 1 }} /><button className="pj-btn" onClick={() => supabase.auth.signOut()}>Sign out</button></div>
            <div className="pj-grid" style={{ gridTemplateColumns: "1fr" }}>
              <div><b>Two-factor authentication (authenticator app)</b>
                {factors.filter((f) => f.status === "verified").length > 0 ? <div style={{ marginTop: 4 }}>Enabled. <button className="pj-btn" onClick={() => run(async () => { for (const f of factors) await supabase.auth.mfa.unenroll({ factorId: f.id }); await loadFactors(); return {}; }, "Two-factor authentication removed.")}>Remove</button></div>
                  : enroll ? (<div style={{ marginTop: 6 }}><div dangerouslySetInnerHTML={{ __html: enroll.qr }} style={{ width: 160 }} /><div style={{ fontSize: 11, color: "#52606d" }}>Or enter this secret: {enroll.secret}</div>
                    <input className="fac" style={{ width: 120, textAlign: "left", height: 28 }} placeholder="6-digit code" value={code} onChange={(e) => setCode(e.target.value)} /> <button className="pj-btn pri" onClick={() => run(async () => { const ch = await supabase.auth.mfa.challenge({ factorId: enroll.id }); if (ch.error) return ch; const v = await supabase.auth.mfa.verify({ factorId: enroll.id, challengeId: ch.data.id, code }); if (!v.error) { setEnroll(null); setCode(""); loadFactors(); } return v; }, "Two-factor authentication enabled.")}>Verify and enable</button></div>)
                  : <div style={{ marginTop: 4 }}><button className="pj-btn" onClick={() => run(async () => { const r = await supabase.auth.mfa.enroll({ factorType: "totp" }); if (!r.error) setEnroll({ id: r.data.id, qr: r.data.totp.qr_code, secret: r.data.totp.secret }); return r; })}>Set up authenticator app</button></div>}
              </div></div></div>

          {needStepUp && (<div className="pj-card"><div className="pj-ch"><b>Verify two-factor authentication</b></div><div className="pj-out"><span>Enter the code from your authenticator app to check or approve revisions in this session.</span><input className="fac" style={{ width: 120, textAlign: "left", height: 26 }} placeholder="6-digit code" value={stepCode} onChange={(e) => setStepCode(e.target.value)} onKeyDown={(e) => e.key === "Enter" && stepUp()} /><button className="pj-btn pri" onClick={stepUp}>Verify</button></div></div>)}
          <div className="pj-card"><div className="pj-ch"><b>Organisations</b><select value={org} onChange={(e) => setOrg(e.target.value)}>{orgs.map((o) => <option key={o.id} value={o.id}>{o.name} ({ROLE_LABEL[o.role]})</option>)}</select><span style={{ flex: 1 }} />
            <input className="fac" style={{ width: 200, textAlign: "left", height: 26 }} placeholder="New organisation name" value={newOrg} onChange={(e) => setNewOrg(e.target.value)} />
            <button className="pj-btn pri" onClick={() => rpc("create_organisation", { p_name: newOrg }, "Organisation created.", () => { setNewOrg(""); loadOrgs(); })}>Create</button></div>
            {!org && <div className="pj-empty">Create an organisation to start. You become its owner.</div>}</div>

          {org && (<>
            <div className="pj-card"><div className="pj-ch"><b>Members and roles</b><span className="pj-tag">Your role: {ROLE_LABEL[myRole ?? ""]}</span></div>
              <table style={{ width: "100%", borderCollapse: "collapse" }}><tbody>
                {members.map((m) => (<tr key={m.user_id} style={{ borderTop: "1px solid #e4e7eb" }}><td style={{ padding: "6px 10px" }}>{m.display_name || "—"}</td><td>{m.email}</td>
                  <td><select disabled={!isAdmin} value={m.role} onChange={(e) => rpc("set_member_role", { p_org: org, p_user: m.user_id, p_role: e.target.value }, "Role updated.", loadOrg)}>{ROLES.map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}</select></td>
                  <td style={{ textAlign: "right", paddingRight: 10 }}>{(isAdmin || m.user_id === uid) && <button className="pj-btn" onClick={() => rpc("remove_member", { p_org: org, p_user: m.user_id }, "Member removed.", () => { loadOrg(); if (m.user_id === uid) loadOrgs(); })}>{m.user_id === uid ? "Leave" : "Remove"}</button>}</td></tr>))}
              </tbody></table>
              {isAdmin && <div className="pj-out"><input className="fac" style={{ width: 240, textAlign: "left", height: 26 }} placeholder="Email of a registered user" value={addEmail} onChange={(e) => setAddEmail(e.target.value)} /><select value={addRole} onChange={(e) => setAddRole(e.target.value)}>{ROLES.map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}</select><button className="pj-btn pri" onClick={() => rpc("add_member", { p_org: org, p_email: addEmail, p_role: addRole }, "Member added.", () => { setAddEmail(""); loadOrg(); })}>Add member</button></div>}
            </div>

            <div className="pj-card"><div className="pj-ch"><b>Security and data</b></div><div className="pj-out" style={{ flexDirection: "column", alignItems: "flex-start", gap: 8 }}>
              <label><input type="checkbox" disabled={!isAdmin} checked={!!orgs.find((o) => o.id === org)?.mfa} onChange={(e) => rpc("set_org_security", { p_org: org, p_require_mfa: e.target.checked }, "Security setting updated.", loadOrgs)} /> Require two-factor authentication to check or approve revisions</label>
              {canAudit && <button className="pj-btn" onClick={() => run(async () => { const r = await supabase.rpc("export_organisation", { p_org: org }); if (!r.error) { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([JSON.stringify(r.data, null, 2)], { type: "application/json" })); a.download = `strucal-${orgs.find((o) => o.id === org)?.name ?? "organisation"}-export.json`; a.click(); } return r; }, "Organisation data exported.")}>Export all organisation data (.json)</button>}
            </div></div>
            <div className="pj-card"><div className="pj-ch"><b>Projects</b><select value={proj} onChange={(e) => setProj(e.target.value)}>{projects.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}</select>
              {proj && <Link className="pj-btn" href={`/project?p=${proj}`} style={{ textDecoration: "none", lineHeight: "24px" }}>Open in Project page</Link>}<span style={{ flex: 1 }} />
              <input className="fac" style={{ width: 200, textAlign: "left", height: 26 }} placeholder="New project name" value={newProj} onChange={(e) => setNewProj(e.target.value)} />
              <button className="pj-btn pri" onClick={() => rpc("create_project", { p_org: org, p_name: newProj, p_description: "" }, "Project created.", () => { setNewProj(""); loadOrg(); })}>Create</button></div>
              {projects.length === 0 && <div className="pj-empty">No projects yet.</div>}
              {proj && (<div style={{ padding: 8 }}>
                <b>Revisions and approval</b>
                <div style={{ color: "#52606d", fontSize: 12, margin: "2px 0 6px" }}>Designer submits, a different person with the Checker role reviews, and a third person with the Approver role signs off. Approved revisions are permanently locked.</div>
                {revs.length === 0 && <div style={{ color: "#52606d" }}>No revisions. Save one from the Project page.</div>}
                {revs.map((r) => {
                  const rvs = reviews.filter((x) => x.revision_id === r.id);
                  const mine = r.created_by === uid;
                  const canSubmit = (r.status === "draft" || r.status === "changes_requested") && ["owner", "administrator", "project_manager", "designer", "student"].includes(myRole ?? "");
                  return (<div key={r.id} style={{ border: "1px solid #c9ced6", marginBottom: 6 }}>
                    <div className="pj-ch"><b>{r.name}</b><span className={`pj-b ${r.status === "approved" ? "pass" : r.status === "rejected" ? "fail" : r.status === "checked" ? "res" : "blk"}`}>{STATUS_LABEL[r.status]}</span><span className="pj-tag" title={r.content_hash}>hash {r.content_hash.slice(0, 10)}</span><span style={{ color: "#52606d" }}>by {names[r.created_by] ?? "?"} · {new Date(r.created_at).toLocaleString()}</span><span style={{ flex: 1 }} />
                      {canSubmit && <button className="pj-btn pri" onClick={() => rpc("submit_revision", { p_rev: r.id }, "Submitted for checking.", loadRevs)}>Submit for checking</button>}</div>
                    {rvs.length > 0 && <div style={{ padding: "4px 10px", fontSize: 12 }}>{rvs.map((x) => <div key={x.id}><b>{names[x.reviewer] ?? "?"}</b> · {x.kind.replace("_", " ")} · {new Date(x.created_at).toLocaleString()}{x.comment ? ` — ${x.comment}` : ""}</div>)}</div>}
                    {((myRole === "checker" && !mine && (r.status === "submitted" || r.status === "checked")) || (myRole === "approver" && !mine && r.status === "checked")) && (
                      <div className="pj-out"><input className="fac" style={{ width: 280, textAlign: "left", height: 26 }} placeholder="Comment (required to reject or request changes)" value={comment} onChange={(e) => setComment(e.target.value)} />
                        {myRole === "checker" && (<>{["comment", "recommend_approval", "request_changes", "reject"].map((o) => <button key={o} className="pj-btn" onClick={() => rpc("check_revision", { p_rev: r.id, p_outcome: o, p_comment: comment }, "Review recorded.", () => { setComment(""); loadRevs(); })}>{o === "recommend_approval" ? "Recommend approval" : o === "request_changes" ? "Request changes" : o === "reject" ? "Reject" : "Comment"}</button>)}</>)}
                        {myRole === "approver" && (<><button className="pj-btn pri" onClick={() => rpc("approve_revision", { p_rev: r.id, p_approve: true, p_comment: comment }, "Revision approved and locked.", () => { setComment(""); loadRevs(); })}>Approve</button><button className="pj-btn" onClick={() => rpc("approve_revision", { p_rev: r.id, p_approve: false, p_comment: comment }, "Revision rejected.", () => { setComment(""); loadRevs(); })}>Reject</button></>)}
                      </div>)}
                  </div>);
                })}
              </div>)}
            </div>

            {canAudit && (<div className="pj-card"><div className="pj-ch"><b>Audit trail</b><span style={{ flex: 1 }} />{chain && <span className={`pj-b ${chain.startsWith("Chain intact") ? "pass" : "fail"}`}>{chain}</span>}
              <button className="pj-btn" onClick={() => run(async () => { const r = await supabase.rpc("verify_audit_chain", { p_org: org }); const row = Array.isArray(r.data) ? r.data[0] : r.data; if (!r.error) setChain(row?.ok ? `Chain intact (${row.checked} events)` : `TAMPERING DETECTED at event ${row?.broken_at}`); return r; })}>Verify integrity</button></div>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}><tbody>{audit.map((a) => <tr key={a.id} style={{ borderTop: "1px solid #e4e7eb" }}><td style={{ padding: "4px 10px", whiteSpace: "nowrap" }}>{new Date(a.created_at).toLocaleString()}</td><td>{names[a.actor] ?? "?"}</td><td><b>{a.action}</b></td><td style={{ color: "#52606d" }}>{JSON.stringify(a.detail)}</td></tr>)}</tbody></table></div>)}
          </>)}
        </>)}
        {session && <div className="pj-card"><div className="pj-ch"><b>Close my account</b></div><div className="pj-out"><span>Removes you from all organisations and disables sign-in. Engineering records you created are kept for audit, shown as a closed account. You must not be the only owner of an organisation.</span><button className="pj-btn" onClick={async () => { if (!confirm("Close your account? This cannot be undone.")) return; const ok = await run(async () => await supabase.rpc("close_account")); if (ok) await supabase.auth.signOut(); }}>Close account</button></div></div>}
        <div className="pj-warn">Preview backend. Single sign-on (SAML) is not enabled. Data is held in a development database on a free plan with no uptime guarantee.</div>
      </div>
    </div>
  );
}
