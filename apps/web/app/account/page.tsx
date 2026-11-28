"use client";
import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { supabase, ROLES, ROLE_LABEL, STATUS_LABEL } from "../../lib/supabase";
import { downloadText } from "../../lib/download";
import "../workspace/workspace.css";
import "../project/project.css";

type Org = { id: string; name: string; role: string; mfa: boolean };
type Member = { user_id: string; role: string; display_name: string; email: string };
type Project = { id: string; name: string; description: string; created_at: string };
type Rev = { id: string; number: number; name: string; status: string; created_by: string; created_at: string; content_hash: string; engine_version: string };
type Review = { id: string; revision_id: string; reviewer: string; kind: string; comment: string; created_at: string };
type Audit = { id: number; actor: string; action: string; entity: string; detail: unknown; created_at: string };
type Factor = { id: string; status: string };
type Result = { error?: { message?: string } | null } | null | undefined | void;

const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const DESIGN_ROLES = ["owner", "administrator", "project_manager", "designer", "student"];
const msg = (e: unknown) => String((e as { message?: string })?.message ?? e).replace(/^\s*ERROR:\s*/i, "");
const when = (s: string) => new Date(s).toLocaleString();

export default function Account() {
  const [session, setSession] = useState<Awaited<ReturnType<typeof supabase.auth.getSession>>["data"]["session"]>(null);
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState("");
  const [mode, setMode] = useState<"in" | "up">("in");
  const [email, setEmail] = useState(""); const [pw, setPw] = useState(""); const [dn, setDn] = useState("");
  const [note, setNote] = useState(""); const [err, setErr] = useState("");
  const [orgs, setOrgs] = useState<Org[]>([]); const [org, setOrg] = useState("");
  const [members, setMembers] = useState<Member[]>([]); const [projects, setProjects] = useState<Project[]>([]);
  const [proj, setProj] = useState(""); const [revs, setRevs] = useState<Rev[]>([]); const [reviews, setReviews] = useState<Review[]>([]);
  const [audit, setAudit] = useState<Audit[]>([]); const [chain, setChain] = useState("");
  const [newOrg, setNewOrg] = useState(""); const [newProj, setNewProj] = useState(""); const [addEmail, setAddEmail] = useState(""); const [addRole, setAddRole] = useState("designer");
  const [comment, setComment] = useState(""); const [aal, setAal] = useState<{ currentLevel: string | null; nextLevel: string | null } | null>(null);
  const [stepCode, setStepCode] = useState(""); const [factors, setFactors] = useState<Factor[]>([]);
  const [enroll, setEnroll] = useState<{ id: string; qr: string; secret: string } | null>(null); const [code, setCode] = useState("");
  const uid = session?.user?.id;
  const myRole = orgs.find((o) => o.id === org)?.role;
  const isAdmin = myRole === "owner" || myRole === "administrator";
  const canAudit = isAdmin || myRole === "auditor";
  const names: Record<string, string> = Object.fromEntries(members.map((m) => [m.user_id, m.display_name || m.email]));

  useEffect(() => {
    supabase.auth.getSession().then(({ data }) => { setSession(data.session); setReady(true); }).catch((e) => { setErr(`Could not reach the account service: ${msg(e)}`); setReady(true); });
    const { data: sub } = supabase.auth.onAuthStateChange((_e, s) => setSession(s));
    return () => sub.subscription.unsubscribe();
  }, []);

  /** Runs an action with a busy marker, shows its error or success note. Returns the result, or null on error. */
  const run = async <T extends Result>(key: string, fn: () => Promise<T>, ok = ""): Promise<T | null> => {
    setErr(""); setNote(""); setBusy(key);
    try { const r = await fn(); if (r && r.error) throw r.error; if (ok) setNote(ok); return r; }
    catch (e) { setErr(msg(e)); return null; }
    finally { setBusy(""); }
  };
  const fail = (m: string) => { setNote(""); setErr(m); };

  const loadOrgs = useCallback(async () => {
    if (!uid) return;
    const { data, error } = await supabase.from("memberships").select("role, organisations(id,name,require_mfa_approval)").eq("user_id", uid);
    if (error) { setErr(`Could not load organisations: ${msg(error)}`); return; }
    const list: Org[] = (data ?? []).flatMap((m: { role: string; organisations: unknown }) => {
      const o = (Array.isArray(m.organisations) ? m.organisations[0] : m.organisations) as { id: string; name: string; require_mfa_approval: boolean } | null;
      return o ? [{ id: o.id, name: o.name, role: m.role, mfa: o.require_mfa_approval }] : [];
    });
    setOrgs(list); setOrg((cur) => (cur && list.some((o) => o.id === cur) ? cur : list[0]?.id ?? ""));
  }, [uid]);
  const loadOrg = useCallback(async () => {
    if (!org) { setMembers([]); setProjects([]); setAudit([]); return; }
    const { data: ms, error: me } = await supabase.from("memberships").select("user_id, role").eq("org_id", org);
    if (me) { setErr(`Could not load members: ${msg(me)}`); return; }
    const ids = (ms ?? []).map((m) => m.user_id as string);
    const { data: ps } = ids.length ? await supabase.from("profiles").select("id, display_name, email").in("id", ids) : { data: [] as { id: string; display_name: string; email: string }[] };
    setMembers((ms ?? []).map((m) => { const p = (ps ?? []).find((x) => x.id === m.user_id); return { user_id: m.user_id, role: m.role, display_name: p?.display_name ?? "", email: p?.email ?? "" }; }));
    const { data: pr, error: pe } = await supabase.from("projects").select("id, name, description, created_at").eq("org_id", org).order("created_at", { ascending: false });
    if (pe) setErr(`Could not load projects: ${msg(pe)}`);
    setProjects(pr ?? []); setProj((c) => (c && (pr ?? []).some((p) => p.id === c) ? c : pr?.[0]?.id ?? ""));
    if (canAudit) { const { data: au } = await supabase.from("audit_events").select("id, actor, action, entity, detail, created_at").eq("org_id", org).order("id", { ascending: false }).limit(40); setAudit(au ?? []); } else setAudit([]);
  }, [org, canAudit]);
  const loadRevs = useCallback(async () => {
    if (!proj) { setRevs([]); setReviews([]); return; }
    const { data, error } = await supabase.from("revisions").select("id, number, name, status, created_by, created_at, content_hash, engine_version").eq("project_id", proj).order("number", { ascending: false });
    if (error) { setErr(`Could not load revisions: ${msg(error)}`); return; }
    setRevs(data ?? []);
    const ids = (data ?? []).map((r) => r.id);
    const { data: rv } = ids.length ? await supabase.from("reviews").select("id, revision_id, reviewer, kind, comment, created_at").in("revision_id", ids).order("created_at") : { data: [] as Review[] };
    setReviews(rv ?? []);
  }, [proj]);
  const loadFactors = useCallback(async () => { const { data } = await supabase.auth.mfa.listFactors(); setFactors((data?.totp ?? []) as Factor[]); }, []);
  const loadAal = useCallback(async () => { const { data } = await supabase.auth.mfa.getAuthenticatorAssuranceLevel(); setAal(data); }, []);
  useEffect(() => { if (uid) { loadOrgs(); loadFactors(); loadAal(); } }, [loadOrgs, loadFactors, loadAal, uid]);
  useEffect(() => { loadOrg(); }, [loadOrg]);
  useEffect(() => { loadRevs(); }, [loadRevs]);

  const needStepUp = !!aal && aal.nextLevel === "aal2" && aal.currentLevel !== "aal2";
  const verifyCode = async (factorId: string, c: string) => {
    const ch = await supabase.auth.mfa.challenge({ factorId });
    if (ch.error) return ch;
    return supabase.auth.mfa.verify({ factorId, challengeId: ch.data.id, code: c.trim() });
  };
  const stepUp = () => {
    const f = factors.find((x) => x.status === "verified");
    if (!f) return fail("No authenticator app is set up for this account.");
    if (!/^\d{6}$/.test(stepCode.trim())) return fail("Enter the 6-digit code from your authenticator app.");
    return run("stepup", async () => { const v = await verifyCode(f.id, stepCode); if (!v.error) { setStepCode(""); await loadAal(); } return v; }, "Two-factor verified for this session.");
  };
  const enable2fa = () => {
    if (!enroll) return;
    if (!/^\d{6}$/.test(code.trim())) return fail("Enter the 6-digit code from your authenticator app.");
    return run("enroll", async () => { const v = await verifyCode(enroll.id, code); if (!v.error) { setEnroll(null); setCode(""); await loadFactors(); await loadAal(); } return v; }, "Two-factor authentication enabled.");
  };

  const signIn = () => {
    if (!EMAIL.test(email.trim())) return fail("Enter a valid email address.");
    if (pw.length < 8) return fail("The password must be at least 8 characters.");
    if (mode === "up" && !dn.trim()) return fail("Enter a display name. It is shown to your team on reviews and approvals.");
    return run("auth", async () => {
      if (mode === "in") return supabase.auth.signInWithPassword({ email: email.trim(), password: pw });
      const r = await supabase.auth.signUp({ email: email.trim(), password: pw, options: { data: { display_name: dn.trim() } } });
      if (r.error || r.data.session) return r;
      const r2 = await supabase.auth.signInWithPassword({ email: email.trim(), password: pw });
      if (r2.error) { setNote("Account created. If email confirmation is switched on, click the link in the email, then sign in."); return { error: null }; }
      return r2;
    });
  };
  const rpc = (key: string, name: string, args: Record<string, unknown>, ok: string, after: () => void) =>
    run(key, async () => { const r = await supabase.rpc(name, args); if (!r.error) after(); return r; }, ok);
  const required = (v: string, what: string) => { if (!v.trim()) { fail(`Enter ${what}.`); return false; } return true; };

  if (!ready) return <div className="pj"><div className="pj-empty" role="status">Loading…</div></div>;
  return (
    <div className="pj">
      <div className="wk-title"><Link href="/" className="wk-logo" aria-label="StruCal home">S</Link><div className="wk-wt">StruCal — Account and teams</div></div>
      <div className="wk-tabs"><span className="file">ACCOUNT</span><Link href="/workspace" style={{ padding: "0 14px", lineHeight: "24px" }}>Workspace</Link><Link href="/project" style={{ padding: "0 14px", lineHeight: "24px" }}>Project</Link></div>
      <div style={{ padding: 8, display: "grid", gap: 8, maxWidth: 1100, width: "100%", margin: "0 auto" }}>
        {err && <div className="pj-err" role="alert">{err}</div>}{note && <div className="pj-warn" role="status">{note}</div>}
        {!session && (
          <form className="pj-card" style={{ maxWidth: 420 }} onSubmit={(e) => { e.preventDefault(); signIn(); }} noValidate>
            <div className="pj-ch"><b>{mode === "in" ? "Sign in" : "Create account"}</b><span style={{ flex: 1 }} /><button type="button" className="pj-btn" onClick={() => { setMode(mode === "in" ? "up" : "in"); setErr(""); setNote(""); }}>{mode === "in" ? "Need an account?" : "Have an account?"}</button></div>
            <div className="pj-grid" style={{ gridTemplateColumns: "1fr" }}>
              {mode === "up" && <div className="pj-f"><label htmlFor="ac-dn">Display name</label><input id="ac-dn" className="fac" autoComplete="name" style={{ width: "100%", textAlign: "left", height: 28 }} value={dn} onChange={(e) => setDn(e.target.value)} /></div>}
              <div className="pj-f"><label htmlFor="ac-em">Email</label><input id="ac-em" className="fac" type="email" autoComplete="email" style={{ width: "100%", textAlign: "left", height: 28 }} value={email} onChange={(e) => setEmail(e.target.value)} /></div>
              <div className="pj-f"><label htmlFor="ac-pw">Password (at least 8 characters)</label><input id="ac-pw" className="fac" type="password" autoComplete={mode === "in" ? "current-password" : "new-password"} style={{ width: "100%", textAlign: "left", height: 28 }} value={pw} onChange={(e) => setPw(e.target.value)} /></div>
              <button type="submit" className="pj-btn pri" disabled={!!busy} style={{ height: 30 }}>{busy === "auth" ? "Please wait…" : mode === "in" ? "Sign in" : "Create account"}</button>
            </div>
          </form>
        )}
        {session && (<>
          <div className="pj-card"><div className="pj-ch"><b>{session.user.user_metadata?.display_name || session.user.email}</b><span className="pj-tag">{session.user.email}</span><span style={{ flex: 1 }} /><button className="pj-btn" onClick={() => supabase.auth.signOut()}>Sign out</button></div>
            <div className="pj-grid" style={{ gridTemplateColumns: "1fr" }}>
              <div><b>Two-factor authentication (authenticator app)</b>
                {factors.some((f) => f.status === "verified") ? <div style={{ marginTop: 4 }}>Enabled. <button className="pj-btn" disabled={!!busy} onClick={() => { if (confirm("Remove two-factor authentication? You will not be able to check or approve revisions in organisations that require it.")) run("unenroll", async () => { for (const f of factors) { const r = await supabase.auth.mfa.unenroll({ factorId: f.id }); if (r.error) return r; } await loadFactors(); await loadAal(); return {}; }, "Two-factor authentication removed."); }}>Remove</button></div>
                  : enroll ? (<div style={{ marginTop: 6 }}><div dangerouslySetInnerHTML={{ __html: enroll.qr }} style={{ width: 160 }} aria-label="QR code for your authenticator app" /><div style={{ fontSize: 11, color: "#52606d" }}>Or enter this secret: <code>{enroll.secret}</code></div>
                    <input className="fac" inputMode="numeric" autoComplete="one-time-code" maxLength={6} style={{ width: 120, textAlign: "left", height: 28 }} placeholder="6-digit code" aria-label="6-digit code" value={code} onChange={(e) => setCode(e.target.value)} onKeyDown={(e) => e.key === "Enter" && enable2fa()} /> <button className="pj-btn pri" disabled={!!busy} onClick={enable2fa}>Verify and enable</button> <button className="pj-btn" onClick={() => { setEnroll(null); setCode(""); }}>Cancel</button></div>)
                  : <div style={{ marginTop: 4 }}><button className="pj-btn" disabled={!!busy} onClick={() => run("startenroll", async () => { const r = await supabase.auth.mfa.enroll({ factorType: "totp" }); if (!r.error) setEnroll({ id: r.data.id, qr: r.data.totp.qr_code, secret: r.data.totp.secret }); return r; })}>Set up authenticator app</button></div>}
              </div></div></div>

          {needStepUp && (<div className="pj-card"><div className="pj-ch"><b>Verify two-factor authentication</b></div><div className="pj-out"><span>Enter the code from your authenticator app to check or approve revisions in this session.</span><input className="fac" inputMode="numeric" autoComplete="one-time-code" maxLength={6} style={{ width: 120, textAlign: "left", height: 26 }} placeholder="6-digit code" aria-label="6-digit code" value={stepCode} onChange={(e) => setStepCode(e.target.value)} onKeyDown={(e) => e.key === "Enter" && stepUp()} /><button className="pj-btn pri" disabled={!!busy} onClick={stepUp}>Verify</button></div></div>)}
          <div className="pj-card"><div className="pj-ch"><b>Organisations</b>{orgs.length > 0 && <select value={org} aria-label="Organisation" onChange={(e) => setOrg(e.target.value)}>{orgs.map((o) => <option key={o.id} value={o.id}>{o.name} ({ROLE_LABEL[o.role]})</option>)}</select>}<span style={{ flex: 1 }} />
            <input className="fac" style={{ width: 200, textAlign: "left", height: 26 }} placeholder="New organisation name" aria-label="New organisation name" value={newOrg} onChange={(e) => setNewOrg(e.target.value)} />
            <button className="pj-btn pri" disabled={!!busy} onClick={() => required(newOrg, "an organisation name") && rpc("org", "create_organisation", { p_name: newOrg.trim() }, "Organisation created.", () => { setNewOrg(""); loadOrgs(); })}>Create</button></div>
            {!org && <div className="pj-empty">Create an organisation to start. You become its owner.</div>}</div>

          {org && (<>
            <div className="pj-card"><div className="pj-ch"><b>Members and roles</b><span className="pj-tag">Your role: {ROLE_LABEL[myRole ?? ""]}</span></div>
              <table style={{ width: "100%", borderCollapse: "collapse" }}><tbody>
                {members.map((m) => (<tr key={m.user_id} style={{ borderTop: "1px solid #e4e7eb" }}><td style={{ padding: "6px 10px" }}>{m.display_name || "—"}</td><td>{m.email}</td>
                  <td><select disabled={!isAdmin || !!busy} value={m.role} aria-label={`Role of ${m.display_name || m.email}`} onChange={(e) => rpc("role", "set_member_role", { p_org: org, p_user: m.user_id, p_role: e.target.value }, "Role updated.", loadOrg)}>{ROLES.map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}</select></td>
                  <td style={{ textAlign: "right", paddingRight: 10 }}>{(isAdmin || m.user_id === uid) && <button className="pj-btn" disabled={!!busy} onClick={() => { const self = m.user_id === uid; if (confirm(self ? "Leave this organisation? You lose access to its projects." : `Remove ${m.display_name || m.email} from this organisation?`)) rpc("rm", "remove_member", { p_org: org, p_user: m.user_id }, self ? "You left the organisation." : "Member removed.", () => { loadOrg(); if (self) loadOrgs(); }); }}>{m.user_id === uid ? "Leave" : "Remove"}</button>}</td></tr>))}
              </tbody></table>
              {isAdmin && <div className="pj-out"><input className="fac" type="email" style={{ width: 240, textAlign: "left", height: 26 }} placeholder="Email of a registered user" aria-label="Member email" value={addEmail} onChange={(e) => setAddEmail(e.target.value)} /><select value={addRole} aria-label="Role for new member" onChange={(e) => setAddRole(e.target.value)}>{ROLES.map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}</select><button className="pj-btn pri" disabled={!!busy} onClick={() => { if (!EMAIL.test(addEmail.trim())) return fail("Enter the email address of a registered user."); rpc("add", "add_member", { p_org: org, p_email: addEmail.trim(), p_role: addRole }, "Member added.", () => { setAddEmail(""); loadOrg(); }); }}>Add member</button></div>}
            </div>

            <div className="pj-card"><div className="pj-ch"><b>Security and data</b></div><div className="pj-out" style={{ flexDirection: "column", alignItems: "flex-start", gap: 8 }}>
              <label><input type="checkbox" disabled={!isAdmin || !!busy} checked={!!orgs.find((o) => o.id === org)?.mfa} onChange={(e) => rpc("sec", "set_org_security", { p_org: org, p_require_mfa: e.target.checked }, "Security setting updated.", loadOrgs)} /> Require two-factor authentication to check or approve revisions</label>
              {canAudit && <button className="pj-btn" disabled={!!busy} onClick={() => run("export", async () => { const r = await supabase.rpc("export_organisation", { p_org: org }); if (!r.error) downloadText(`strucal-${orgs.find((o) => o.id === org)?.name ?? "organisation"}-export.json`, JSON.stringify(r.data, null, 2), "application/json"); return r; }, "Organisation data exported.")}>{busy === "export" ? "Exporting…" : "Export all organisation data (.json)"}</button>}
            </div></div>
            <div className="pj-card"><div className="pj-ch"><b>Projects</b>{projects.length > 0 && <select value={proj} aria-label="Project" onChange={(e) => setProj(e.target.value)}>{projects.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}</select>}
              {proj && <Link className="pj-btn" href={`/project?p=${proj}`} style={{ textDecoration: "none", lineHeight: "24px" }}>Open in Project page</Link>}<span style={{ flex: 1 }} />
              <input className="fac" style={{ width: 200, textAlign: "left", height: 26 }} placeholder="New project name" aria-label="New project name" value={newProj} onChange={(e) => setNewProj(e.target.value)} />
              <button className="pj-btn pri" disabled={!!busy} onClick={() => required(newProj, "a project name") && rpc("proj", "create_project", { p_org: org, p_name: newProj.trim(), p_description: "" }, "Project created.", () => { setNewProj(""); loadOrg(); })}>Create</button></div>
              {projects.length === 0 && <div className="pj-empty">No projects yet.</div>}
              {proj && (<div style={{ padding: 8 }}>
                <b>Revisions and approval</b>
                <div style={{ color: "#52606d", fontSize: 12, margin: "2px 0 6px" }}>Designer submits, a different person with the Checker role reviews, and a third person with the Approver role signs off. Approved revisions are permanently locked.</div>
                {revs.length === 0 && <div style={{ color: "#52606d" }}>No revisions. Save one from the Project page.</div>}
                {revs.map((r) => {
                  const rvs = reviews.filter((x) => x.revision_id === r.id);
                  const mine = r.created_by === uid;
                  const canSubmit = (r.status === "draft" || r.status === "changes_requested") && DESIGN_ROLES.includes(myRole ?? "");
                  const canCheck = myRole === "checker" && !mine && (r.status === "submitted" || r.status === "checked");
                  const canApprove = myRole === "approver" && !mine && r.status === "checked";
                  const review = (outcome: string, label: string) => {
                    if ((outcome === "reject" || outcome === "request_changes") && !comment.trim()) return fail(`A comment is required to ${label.toLowerCase()}.`);
                    if (outcome === "reject" && !confirm(`Reject ${r.name}?`)) return;
                    rpc("rev", "check_revision", { p_rev: r.id, p_outcome: outcome, p_comment: comment.trim() }, "Review recorded.", () => { setComment(""); loadRevs(); });
                  };
                  return (<div key={r.id} style={{ border: "1px solid #c9ced6", marginBottom: 6 }}>
                    <div className="pj-ch"><b>{r.name}</b><span className={`pj-b ${r.status === "approved" ? "pass" : r.status === "rejected" ? "fail" : r.status === "checked" ? "res" : "blk"}`}>{STATUS_LABEL[r.status] ?? r.status}</span><span className="pj-tag" title={r.content_hash}>hash {r.content_hash.slice(0, 10)}</span><span style={{ color: "#52606d" }}>by {names[r.created_by] ?? "former member"} · {when(r.created_at)}</span><span style={{ flex: 1 }} />
                      {canSubmit && <button className="pj-btn pri" disabled={!!busy} onClick={() => rpc("rev", "submit_revision", { p_rev: r.id }, "Submitted for checking.", loadRevs)}>Submit for checking</button>}</div>
                    {rvs.length > 0 && <div style={{ padding: "4px 10px", fontSize: 12 }}>{rvs.map((x) => <div key={x.id}><b>{names[x.reviewer] ?? "former member"}</b> · {x.kind.replace(/_/g, " ")} · {when(x.created_at)}{x.comment ? ` — ${x.comment}` : ""}</div>)}</div>}
                    {(canCheck || canApprove) && (
                      <div className="pj-out"><input className="fac" style={{ width: 280, textAlign: "left", height: 26 }} placeholder="Comment (required to reject or request changes)" aria-label="Review comment" value={comment} onChange={(e) => setComment(e.target.value)} />
                        {canCheck && ([["comment", "Comment"], ["recommend_approval", "Recommend approval"], ["request_changes", "Request changes"], ["reject", "Reject"]] as const).map(([o, l]) => <button key={o} className="pj-btn" disabled={!!busy} onClick={() => review(o, l)}>{l}</button>)}
                        {canApprove && (<><button className="pj-btn pri" disabled={!!busy} onClick={() => { if (confirm(`Approve and permanently lock ${r.name}?`)) rpc("rev", "approve_revision", { p_rev: r.id, p_approve: true, p_comment: comment.trim() }, "Revision approved and locked.", () => { setComment(""); loadRevs(); }); }}>Approve</button><button className="pj-btn" disabled={!!busy} onClick={() => { if (!comment.trim()) return fail("A comment is required to reject."); if (confirm(`Reject ${r.name}?`)) rpc("rev", "approve_revision", { p_rev: r.id, p_approve: false, p_comment: comment.trim() }, "Revision rejected.", () => { setComment(""); loadRevs(); }); }}>Reject</button></>)}
                      </div>)}
                  </div>);
                })}
              </div>)}
            </div>

            {canAudit && (<div className="pj-card"><div className="pj-ch"><b>Audit trail</b><span className="pj-tag">latest {audit.length}</span><span style={{ flex: 1 }} />{chain && <span className={`pj-b ${chain.startsWith("Chain intact") ? "pass" : "fail"}`}>{chain}</span>}
              <button className="pj-btn" disabled={!!busy} onClick={() => run("chain", async () => { const r = await supabase.rpc("verify_audit_chain", { p_org: org }); const row = Array.isArray(r.data) ? r.data[0] : r.data; if (!r.error) setChain(row?.ok ? `Chain intact (${row.checked} events)` : `TAMPERING DETECTED at event ${row?.broken_at}`); return r; })}>{busy === "chain" ? "Verifying…" : "Verify integrity"}</button></div>
              {audit.length === 0 ? <div className="pj-empty">No events recorded yet.</div> : <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}><tbody>{audit.map((a) => <tr key={a.id} style={{ borderTop: "1px solid #e4e7eb" }}><td style={{ padding: "4px 10px", whiteSpace: "nowrap" }}>{when(a.created_at)}</td><td>{names[a.actor] ?? "former member"}</td><td><b>{a.action}</b></td><td style={{ color: "#52606d" }}>{JSON.stringify(a.detail)}</td></tr>)}</tbody></table>}</div>)}
          </>)}
        </>)}
        {session && <div className="pj-card"><div className="pj-ch"><b>Close my account</b></div><div className="pj-out"><span>Removes you from all organisations and disables sign-in. Engineering records you created are kept for audit, shown as a closed account. You must not be the only owner of an organisation.</span><button className="pj-btn" disabled={!!busy} onClick={async () => { if (!confirm("Close your account? This cannot be undone.")) return; const ok = await run("close", async () => await supabase.rpc("close_account")); if (ok) await supabase.auth.signOut(); }}>Close account</button></div></div>}
        <div className="pj-warn">Preview backend. Single sign-on (SAML) is not enabled. Data is held in a development database on a free plan with no uptime guarantee.</div>
      </div>
    </div>
  );
}
