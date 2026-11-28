// Theme: "drafting" (light) or "obsidian" (dark). The initial value is applied before paint by the script in
// app/layout.tsx; this module reads and switches it and remembers the choice.
"use client";
import { useEffect, useState } from "react";

export type Theme = "drafting" | "obsidian";
const KEY = "strucal.theme";

export function currentTheme(): Theme {
  if (typeof document === "undefined") return "drafting";
  return document.documentElement.dataset.theme === "obsidian" ? "obsidian" : "drafting";
}

export function setTheme(t: Theme) {
  document.documentElement.dataset.theme = t;
  try { localStorage.setItem(KEY, t); } catch { /* storage unavailable: theme still applies for this page */ }
  window.dispatchEvent(new CustomEvent("strucal-theme", { detail: t }));
}

export function useTheme(): [Theme, () => void] {
  const [t, setT] = useState<Theme>("drafting");
  useEffect(() => {
    setT(currentTheme());
    const on = (e: Event) => setT((e as CustomEvent<Theme>).detail);
    window.addEventListener("strucal-theme", on);
    return () => window.removeEventListener("strucal-theme", on);
  }, []);
  return [t, () => setTheme(t === "obsidian" ? "drafting" : "obsidian")];
}
