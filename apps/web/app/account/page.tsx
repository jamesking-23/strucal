"use client";
import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import TopBar from "../components/TopBar";
import { supabase, ROLES, ROLE_LABEL, STATUS_LABEL } from "../../lib/supabase";
import { downloadText } from "../../lib/download";
import { Button, Notice, StatusBadge } from "../ui/primitives";
import "./account.css";

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

  if (!ready) return <div className="app"><TopBar section="account" /><div className="loading" role="status"><span className="spinner" />Checking your session…</div></div>;

  const banner = (<>{err && <Notice tone="error" onDismiss={() => setErr("")}>{err}</Notice>}{note && <Notice tone="ok">{note}</Notice>}</>);

  if (!session) return (
    <div className="app">
      <TopBar section="account" context="Cloud projects, approvals and the assistant require an account" />
      <main className="auth">
        <div className="auth-grid">
          <section className="auth-copy">
            <span className="eyebrow">StruCal account</span>
            <h1>Sign in to share projects and sign off revisions.</h1>
            <ul className="auth-points">
              <li><b>Cloud projects</b><span>Linked calculations saved with optimistic locking.</span></li>
              <li><b>Check and approval</b><span>Designer, checker and approver are three different people.</span></li>
              <li><b>Audit trail</b><span>Every action recorded in a tamper-evident chain.</span></li>
            </ul>
            <p className="faint small">The workbench works without an account; inputs stay in this browser.</p>
          </section>
          <form className="auth-form" onSubmit={(e) => { e.preventDefault(); signIn(); }} noValidate aria-labelledby="auth-h">
            <div className="seg full" role="tablist" aria-label="Sign in or create account">
              <button type="button" role="tab" aria-selected={mode === "in"} onClick={() => { setMode("in"); setErr(""); setNote(""); }}>Sign in</button>
              <button type="button" role="tab" aria-selected={mode === "up"} onClick={() => { setMode("up"); setErr(""); setNote(""); }}>Create account</button>
            </div>
            <h2 id="auth-h" className="sr-only">{mode === "in" ? "Sign in" : "Create account"}</h2>
            {banner}
            {mode === "up" && <div className="field"><label htmlFor="ac-dn">Display name · shown on reviews and approvals</label><input id="ac-dn" className="input" autoComplete="name" value={dn} onChange={(e) => setDn(e.target.value)} /></div>}
            <div className="field"><label htmlFor="ac-em">Email</label><input id="ac-em" className="input" type="email" autoComplete="email" value={email} onChange={(e) => setEmail(e.target.value)} /></div>
            <div className="field"><label htmlFor="ac-pw">Password · at least 8 characters</label><input id="ac-pw" className="input" type="password" autoComplete={mode === "in" ? "current-password" : "new-password"} value={pw} onChange={(e) => setPw(e.target.value)} /></div>
            <button type="submit" className="btn btn-primary btn-lg" disabled={!!busy}>{busy === "auth" ? "Please wait…" : mode === "in" ? "Sign in" : "Create account"}</button>
            <p className="faint small">Preview backend: single sign-on (SAML) is not enabled; data is held in a development database with no uptime guarantee.</p>
          </form>
        </div>
      </main>
    </div>
  );

  const orgName = orgs.find((o) => o.id === org)?.name ?? "";
  const STEPS = ["draft", "submitted", "checked", "approved"];
  const flow = (st: string) => {
    const at = st === "changes_requested" ? 0 : st === "rejected" ? -1 : STEPS.indexOf(st);
    return <div className="flow" aria-label={`Status: ${STATUS_LABEL[st] ?? st}`}>{STEPS.map((s, i) => <span key={s} className={st === "rejected" && i === 0 ? "bad" : i < at ? "done" : i === at ? (st === "changes_requested" ? "bad" : "now") : ""}>{st === "rejected" && i === 0 ? "Rejected" : st === "changes_requested" && i === 0 ? "Changes requested" : s}</span>)}</div>;
  };
  const SECTIONS: [string, string, boolean][] = [["ac-profile", "Profile and security", true], ["ac-org", "Organisation", true], ["ac-members", "Members and roles", !!org], ["ac-projects", "Projects and approval", !!org], ["ac-data", "Security and data", !!org], ["ac-audit", "Audit trail", !!org && canAudit], ["ac-close", "Close account", true]];
  const mfaOn = factors.some((f) => f.status === "verified");

  return (
    <div className="app">
      <TopBar section="account" context={orgName ? `${orgName} · ${ROLE_LABEL[myRole ?? ""] ?? ""}` : session.user.email} />
      <div className="ac-grid">
        <nav className="ac-nav" aria-label="Account sections">
          <div className="ac-who"><b>{session.user.user_metadata?.display_name || session.user.email}</b><span className="mono">{session.user.email}</span><Button size="sm" onClick={() => supabase.auth.signOut()}>Sign out</Button></div>
          {SECTIONS.filter((s) => s[2]).map(([id, label]) => <a key={id} href={`#${id}`}>{label}</a>)}
        </nav>

        <main className="ac-main">
          {banner}

          {needStepUp && (
            <div className="alert warn"><div className="grow"><b>Two-factor verification required</b> to check or approve revisions in this session.</div>
              <div className="ac-inline"><input className="input num" style={{ width: 120 }} inputMode="numeric" autoComplete="one-time-code" maxLength={6} aria-label="6-digit code" placeholder="000000" value={stepCode} onChange={(e) => setStepCode(e.target.value)} onKeyDown={(e) => e.key === "Enter" && stepUp()} /><Button variant="primary" disabled={!!busy} onClick={stepUp}>Verify</Button></div>
            </div>
          )}

          <section className="ac-sec" id="ac-profile" aria-labelledby="h-profile">
            <header><h2 id="h-profile">Profile and security</h2><StatusBadge status={mfaOn ? "ok" : "none"}>{mfaOn ? "2FA on" : "2FA off"}</StatusBadge></header>
            <div className="ac-body">
              <dl className="kv"><div><dt>Display name</dt><dd>{session.user.user_metadata?.display_name || "—"}</dd></div><div><dt>Email</dt><dd>{session.user.email}</dd></div></dl>
              {mfaOn ? (
                <div className="row-actions"><span className="muted small">An authenticator app protects checks and approvals.</span><span className="spacer" /><Button size="sm" variant="danger" disabled={!!busy} onClick={() => { if (confirm("Remove two-factor authentication? You will not be able to check or approve revisions in organisations that require it.")) run("unenroll", async () => { for (const f of factors) { const r = await supabase.auth.mfa.unenroll({ factorId: f.id }); if (r.error) return r; } await loadFactors(); await loadAal(); return {}; }, "Two-factor authentication removed."); }}>Remove 2FA</Button></div>
              ) : enroll ? (
                <div className="ac-enroll">
                  <div className="qr" dangerouslySetInnerHTML={{ __html: enroll.qr }} role="img" aria-label="QR code for your authenticator app" />
                  <div className="stack">
                    <span className="muted small">Scan the code, or enter this secret in your authenticator app:</span>
                    <code className="secret">{enroll.secret}</code>
                    <div className="ac-inline"><input className="input num" style={{ width: 120 }} inputMode="numeric" autoComplete="one-time-code" maxLength={6} aria-label="6-digit code" placeholder="000000" value={code} onChange={(e) => setCode(e.target.value)} onKeyDown={(e) => e.key === "Enter" && enable2fa()} /><Button variant="primary" disabled={!!busy} onClick={enable2fa}>Verify and enable</Button><Button variant="ghost" onClick={() => { setEnroll(null); setCode(""); }}>Cancel</Button></div>
                  </div>
                </div>
              ) : (
                <div className="row-actions"><span className="muted small">Organisations can require two-factor authentication before anyone checks or approves a revision.</span><span className="spacer" /><Button disabled={!!busy} onClick={() => run("startenroll", async () => { const r = await supabase.auth.mfa.enroll({ factorType: "totp" }); if (!r.error) setEnroll({ id: r.data.id, qr: r.data.totp.qr_code, secret: r.data.totp.secret }); return r; })}>Set up authenticator app</Button></div>
              )}
            </div>
          </section>

          <section className="ac-sec" id="ac-org" aria-labelledby="h-org">
            <header><h2 id="h-org">Organisation</h2>{orgs.length > 0 && <select className="select ac-pick" value={org} aria-label="Organisation" onChange={(e) => setOrg(e.target.value)}>{orgs.map((o) => <option key={o.id} value={o.id}>{o.name} · {ROLE_LABEL[o.role]}</option>)}</select>}</header>
            <div className="ac-body">
              {orgs.length === 0 && <p className="muted small">Create an organisation to start. You become its owner.</p>}
              <div className="ac-inline"><label className="sr-only" htmlFor="new-org">New organisation</label><input id="new-org" className="input" placeholder="New organisation name" value={newOrg} onChange={(e) => setNewOrg(e.target.value)} /><Button disabled={!!busy} onClick={() => required(newOrg, "an organisation name") && rpc("org", "create_organisation", { p_name: newOrg.trim() }, "Organisation created.", () => { setNewOrg(""); loadOrgs(); })}>Create</Button></div>
            </div>
          </section>

          {org && (<>
            <section className="ac-sec" id="ac-members" aria-labelledby="h-mem">
              <header><h2 id="h-mem">Members and roles</h2><span className="badge">Your role: {ROLE_LABEL[myRole ?? ""]}</span></header>
              <div className="tbl-wrap">
                <table className="tbl"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th className="r"><span className="sr-only">Actions</span></th></tr></thead><tbody>
                  {members.map((m) => (<tr key={m.user_id}>
                    <td>{m.display_name || "—"}{m.user_id === uid && <span className="faint"> (you)</span>}</td><td className="m muted">{m.email}</td>
                    <td><select className="select ac-role" disabled={!isAdmin || !!busy} value={m.role} aria-label={`Role of ${m.display_name || m.email}`} onChange={(e) => rpc("role", "set_member_role", { p_org: org, p_user: m.user_id, p_role: e.target.value }, "Role updated.", loadOrg)}>{ROLES.map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}</select></td>
                    <td className="r">{(isAdmin || m.user_id === uid) && <Button size="sm" variant="danger" disabled={!!busy} onClick={() => { const self = m.user_id === uid; if (confirm(self ? "Leave this organisation? You lose access to its projects." : `Remove ${m.display_name || m.email} from this organisation?`)) rpc("rm", "remove_member", { p_org: org, p_user: m.user_id }, self ? "You left the organisation." : "Member removed.", () => { loadOrg(); if (self) loadOrgs(); }); }}>{m.user_id === uid ? "Leave" : "Remove"}</Button>}</td>
                  </tr>))}
                </tbody></table>
              </div>
              {isAdmin && (
                <div className="ac-foot ac-inline">
                  <label className="sr-only" htmlFor="add-em">Email of a registered user</label><input id="add-em" className="input" type="email" placeholder="Email of a registered user" value={addEmail} onChange={(e) => setAddEmail(e.target.value)} />
                  <label className="sr-only" htmlFor="add-role">Role</label><select id="add-role" className="select ac-role" value={addRole} onChange={(e) => setAddRole(e.target.value)}>{ROLES.map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}</select>
                  <Button variant="primary" disabled={!!busy} onClick={() => { if (!EMAIL.test(addEmail.trim())) return fail("Enter the email address of a registered user."); rpc("add", "add_member", { p_org: org, p_email: addEmail.trim(), p_role: addRole }, "Member added.", () => { setAddEmail(""); loadOrg(); }); }}>Add member</Button>
                </div>
              )}
            </section>

            <section className="ac-sec" id="ac-projects" aria-labelledby="h-prj">
              <header><h2 id="h-prj">Projects and approval</h2>{projects.length > 0 && <><select className="select ac-pick" value={proj} aria-label="Project" onChange={(e) => setProj(e.target.value)}>{projects.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}</select>{proj && <Link className="btn btn-primary btn-sm" href={`/project?p=${proj}`}>Open project</Link>}</>}</header>
              <div className="ac-body">
                <div className="ac-inline"><label className="sr-only" htmlFor="new-proj">New project</label><input id="new-proj" className="input" placeholder="New project name" value={newProj} onChange={(e) => setNewProj(e.target.value)} /><Button disabled={!!busy} onClick={() => required(newProj, "a project name") && rpc("proj", "create_project", { p_org: org, p_name: newProj.trim(), p_description: "" }, "Project created.", () => { setNewProj(""); loadOrg(); })}>Create</Button></div>
                {projects.length === 0 && <p className="muted small">No projects yet.</p>}
                {proj && (<>
                  <p className="muted small">The designer submits; a different person with the Checker role reviews; a third with the Approver role signs off. Approved revisions are permanently locked.</p>
                  {revs.length === 0 && <p className="muted small">No revisions. Save one from the Project page.</p>}
                  <div className="revs">{revs.map((r) => {
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
                    return (
                      <article className="rev" key={r.id}>
                        <div className="rev-h">
                          <h4>{r.name}</h4>{flow(r.status)}<span className="spacer" />
                          <span className="code-id faint" title={r.content_hash}>{r.content_hash.slice(0, 10)}</span>
                          {canSubmit && <Button size="sm" variant="primary" disabled={!!busy} onClick={() => rpc("rev", "submit_revision", { p_rev: r.id }, "Submitted for checking.", loadRevs)}>Submit for checking</Button>}
                        </div>
                        <div className="rev-meta">By {names[r.created_by] ?? "former member"} · {when(r.created_at)} · engine v{r.engine_version}</div>
                        {rvs.length > 0 && <div className="reviews">{rvs.map((x) => <div key={x.id}><b>{names[x.reviewer] ?? "former member"}</b> · {x.kind.replace(/_/g, " ")} · <span className="faint">{when(x.created_at)}</span>{x.comment ? ` — ${x.comment}` : ""}</div>)}</div>}
                        {(canCheck || canApprove) && (
                          <div className="rev-act">
                            <div className="field"><label htmlFor={`cm-${r.id}`}>Comment · required to reject or request changes</label><input id={`cm-${r.id}`} className="input" value={comment} onChange={(e) => setComment(e.target.value)} /></div>
                            <div className="row-actions">
                              {canCheck && (<>
                                <Button size="sm" variant="primary" disabled={!!busy} onClick={() => review("recommend_approval", "Recommend approval")}>Recommend approval</Button>
                                <Button size="sm" disabled={!!busy} onClick={() => review("comment", "Comment")}>Comment</Button>
                                <Button size="sm" disabled={!!busy} onClick={() => review("request_changes", "Request changes")}>Request changes</Button>
                                <Button size="sm" variant="danger" disabled={!!busy} onClick={() => review("reject", "Reject")}>Reject</Button>
                              </>)}
                              {canApprove && (<>
                                <Button size="sm" variant="primary" disabled={!!busy} onClick={() => { if (confirm(`Approve and permanently lock ${r.name}?`)) rpc("rev", "approve_revision", { p_rev: r.id, p_approve: true, p_comment: comment.trim() }, "Revision approved and locked.", () => { setComment(""); loadRevs(); }); }}>Approve and lock</Button>
                                <Button size="sm" variant="danger" disabled={!!busy} onClick={() => { if (!comment.trim()) return fail("A comment is required to reject."); if (confirm(`Reject ${r.name}?`)) rpc("rev", "approve_revision", { p_rev: r.id, p_approve: false, p_comment: comment.trim() }, "Revision rejected.", () => { setComment(""); loadRevs(); }); }}>Reject</Button>
                              </>)}
                            </div>
                          </div>
                        )}
                      </article>
                    );
                  })}</div>
                </>)}
              </div>
            </section>

            <section className="ac-sec" id="ac-data" aria-labelledby="h-sec">
              <header><h2 id="h-sec">Security and data</h2></header>
              <div className="ac-body">
                <label className="check"><input type="checkbox" disabled={!isAdmin || !!busy} checked={!!orgs.find((o) => o.id === org)?.mfa} onChange={(e) => rpc("sec", "set_org_security", { p_org: org, p_require_mfa: e.target.checked }, "Security setting updated.", loadOrgs)} />Require two-factor authentication to check or approve revisions</label>
                {canAudit && <div><Button disabled={!!busy} onClick={() => run("export", async () => { const r = await supabase.rpc("export_organisation", { p_org: org }); if (!r.error) downloadText(`strucal-${orgName || "organisation"}-export.json`, JSON.stringify(r.data, null, 2), "application/json"); return r; }, "Organisation data exported.")}>{busy === "export" ? "Exporting…" : "Export all organisation data (.json)"}</Button></div>}
              </div>
            </section>

            {canAudit && (
              <section className="ac-sec" id="ac-audit" aria-labelledby="h-aud">
                <header><h2 id="h-aud">Audit trail</h2><span className="faint small">latest {audit.length}</span><span className="spacer" />{chain && <StatusBadge status={chain.startsWith("Chain intact") ? "ok" : "fail"}>{chain}</StatusBadge>}
                  <Button size="sm" disabled={!!busy} onClick={() => run("chain", async () => { const r = await supabase.rpc("verify_audit_chain", { p_org: org }); const row = Array.isArray(r.data) ? r.data[0] : r.data; if (!r.error) setChain(row?.ok ? `Chain intact (${row.checked} events)` : `Tampering detected at event ${row?.broken_at}`); return r; })}>{busy === "chain" ? "Verifying…" : "Verify integrity"}</Button></header>
                {audit.length === 0 ? <div className="ac-body"><p className="muted small">No events recorded yet.</p></div> : (
                  <div className="tbl-wrap"><table className="tbl audit"><thead><tr><th>When</th><th>Who</th><th>Action</th><th>Detail</th></tr></thead><tbody>
                    {audit.map((a) => <tr key={a.id}><td className="m" style={{ whiteSpace: "nowrap" }}>{when(a.created_at)}</td><td>{names[a.actor] ?? "former member"}</td><td><b>{a.action}</b></td><td className="detail">{JSON.stringify(a.detail)}</td></tr>)}
                  </tbody></table></div>)}
              </section>
            )}
          </>)}

          <section className="ac-sec danger-zone" id="ac-close" aria-labelledby="h-close">
            <header><h2 id="h-close">Close account</h2></header>
            <div className="ac-body"><div className="row-actions"><p className="muted small" style={{ flex: "1 1 320px" }}>Removes you from all organisations and disables sign-in. Engineering records you created are kept for audit, shown as a closed account. You must not be the only owner of an organisation.</p><Button variant="danger" disabled={!!busy} onClick={async () => { if (!confirm("Close your account? This cannot be undone.")) return; const ok = await run("close", async () => await supabase.rpc("close_account")); if (ok) await supabase.auth.signOut(); }}>Close account</Button></div></div>
          </section>
          <p className="faint small">Preview backend. Single sign-on (SAML) is not enabled. Data is held in a development database on a free plan with no uptime guarantee.</p>
        </main>
      </div>
    </div>
  );
}
