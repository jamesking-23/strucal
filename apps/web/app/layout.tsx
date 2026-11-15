import "./globals.css";
import type { Metadata } from "next";
export const metadata: Metadata = { title: "StruCal | Structural calculation & verification", description: "Deterministic, traceable Eurocode calculations for structural engineers." };
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (<html lang="en"><body>{children}</body></html>);
}
