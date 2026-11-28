// Application shell: left rail (sections, theme, account), command bar (breadcrumbs, palette, page actions),
// main area and status bar. Session-aware: the account avatar reflects the signed-in user everywhere.
"use client";
import { useEffect, useMemo, useState, type ReactNode } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useTheme } from "../../lib/theme";
import { useSession, initials, displayName } from "../../lib/session";
import { supabase } from "../../lib/supabase";
import { IconAccount, IconMoon, IconProject, IconSearch, IconStandards, IconSun, IconWorkbench } from "../ui/icons";
import CommandPalette, { type Command } from "./CommandPalette";

export type Section = "workbench" | "project" | "standards" | "account";

const NAV: { id: Section; href: string; label: string; Icon: (p: { className?: string }) => ReactNode }[] = [
  { id: "workbench", href: "/workspace", label: "Workbench", Icon: IconWorkbench },
  { id: "project", href: "/project", label: "Project", Icon: IconProject },
  { id: "standards", href: "/standards", label: "Standards", Icon: IconStandards },
  { id: "account", href: "/account", label: "Account", Icon: IconAccount },
];

export default function AppShell({ section, crumbs, actions, status, commands = [], scroll, children }: {
  section: Section; crumbs: string[]; actions?: ReactNode; status?: ReactNode; commands?: Command[]; scroll?: boolean; children: ReactNode;
}) {
  const router = useRouter();
  const [theme, toggleTheme] = useTheme();
  const auth = useSession();
  const [palette, setPalette] = useState(false);

  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") { e.preventDefault(); setPalette((p) => !p); }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  const all = useMemo<Command[]>(() => [
    ...commands,
    ...NAV.map((n) => ({ id: `nav-${n.id}`, group: "Go to", title: n.label, hint: n.href, run: () => router.push(n.href) })),
    { id: "nav-home", group: "Go to", title: "Product overview", hint: "/", run: () => router.push("/") },
    { id: "theme", group: "Preferences", title: theme === "obsidian" ? "Switch to drafting (light) theme" : "Switch to obsidian (dark) theme", keywords: "theme dark light mode", run: toggleTheme },
    ...(auth.status === "signed-in"
      ? [{ id: "signout", group: "Account", title: "Sign out", hint: auth.session.user.email ?? "", run: () => { supabase.auth.signOut(); } }]
      : [{ id: "signin", group: "Account", title: "Sign in", hint: "/account", run: () => router.push("/account") }]),
  ], [commands, router, theme, toggleTheme, auth]);

  return (
    <div className={`shell${scroll ? " scroll" : ""}`}>
      <aside className="s-rail" aria-label="Application">
        <Link href="/" className="s-brand" aria-label="StruCal overview"><i>S</i></Link>
        <nav className="s-nav" aria-label="Sections">
          {NAV.map(({ id, href, label, Icon }) => (
            <Link key={id} href={href} aria-current={id === section ? "page" : undefined} aria-label={label}><Icon /><span className="tip">{label}</span></Link>
          ))}
        </nav>
        <div className="s-rail-foot">
          <button type="button" className="s-rail-btn theme" onClick={toggleTheme} aria-label={theme === "obsidian" ? "Use drafting theme" : "Use obsidian theme"}>
            {theme === "obsidian" ? <IconSun /> : <IconMoon />}<span className="tip">{theme === "obsidian" ? "Drafting theme" : "Obsidian theme"}</span>
          </button>
          <Link href="/account" className="s-rail-btn" aria-label={auth.status === "signed-in" ? `Account: ${displayName(auth.session)}` : "Sign in"}>
            <span className={`s-avatar${auth.status === "signed-in" ? " on" : ""}`}>{auth.status === "signed-in" ? initials(auth.session) : "—"}</span>
            <span className="tip">{auth.status === "signed-in" ? displayName(auth.session) : auth.status === "loading" ? "Checking session…" : "Sign in"}</span>
          </Link>
        </div>
      </aside>

      <header className="s-bar">
        <nav className="s-crumbs" aria-label="Location">
          <span>StruCal</span>
          {crumbs.slice(0, -1).map((c) => <span key={c}>{c}</span>)}
          <b>{crumbs[crumbs.length - 1]}</b>
        </nav>
        <button type="button" className="s-cmd" onClick={() => setPalette(true)} aria-label="Open command palette"><IconSearch width={15} height={15} /><span className="lbl">Modules, pages, actions</span><kbd>⌘K</kbd></button>
        <div className="s-bar-actions">{actions}</div>
      </header>

      <main className="s-main">{children}</main>
      <footer className="s-status">{status ?? <span className="grow" />}</footer>
      <CommandPalette open={palette} onClose={() => setPalette(false)} commands={all} />
    </div>
  );
}
