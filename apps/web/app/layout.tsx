import "./globals.css";
import type { Metadata, Viewport } from "next";

export const metadata: Metadata = {
  title: "StruCal — Structural calculation & verification",
  description: "Deterministic, traceable Eurocode calculations for structural engineers. Rust engine, WebAssembly workbench, native desktop.",
};
export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: [{ media: "(prefers-color-scheme: light)", color: "#ffffff" }, { media: "(prefers-color-scheme: dark)", color: "#101215" }],
};

// Applies the saved theme (or the system preference) before first paint, so there is no flash of the wrong theme.
const THEME_BOOT = `try{var t=localStorage.getItem("strucal.theme");if(t!=="drafting"&&t!=="obsidian"){t=matchMedia("(prefers-color-scheme: dark)").matches?"obsidian":"drafting"}document.documentElement.dataset.theme=t}catch(e){document.documentElement.dataset.theme="drafting"}`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_BOOT }} />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="" />
        <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Geist:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" />
      </head>
      <body>{children}</body>
    </html>
  );
}
