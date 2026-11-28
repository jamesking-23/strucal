import Link from "next/link";

export type Section = "workspace" | "project" | "standards" | "account";

const NAV: { id: Section; href: string; label: string }[] = [
  { id: "workspace", href: "/workspace", label: "Workspace" },
  { id: "project", href: "/project", label: "Project" },
  { id: "standards", href: "/standards", label: "Standards" },
  { id: "account", href: "/account", label: "Account" },
];

/** Application header shared by every app page: brand, section navigation and an optional context line. */
export default function TopBar({ section, context, children }: { section: Section; context?: React.ReactNode; children?: React.ReactNode }) {
  return (
    <header className="wk-title">
      <Link href="/" className="wk-logo" aria-label="StruCal home">S</Link>
      <nav className="wk-nav" aria-label="Sections">
        {NAV.map((n) => <Link key={n.id} href={n.href} className={n.id === section ? "on" : undefined} aria-current={n.id === section ? "page" : undefined}>{n.label}</Link>)}
      </nav>
      <div className="wk-wt">{context}</div>
      {children}
    </header>
  );
}
