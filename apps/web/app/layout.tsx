import "./globals.css";
import type { Metadata } from "next";
export const metadata: Metadata = { title: "StruCal | Structural calculation & verification", description: "Deterministic, traceable Eurocode calculations for structural engineers." };
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (<html lang="en"><head>
    <link rel="preconnect" href="https://fonts.googleapis.com" /><link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="" />
    <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600;700&display=swap" />
  </head><body>{children}</body></html>);
}
