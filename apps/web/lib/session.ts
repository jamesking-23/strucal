// Single source of truth for the Supabase auth session in the browser. Every component reads the same store,
// so sign-in/out in one place (or another tab) updates the shell, the assistant and every page at once.
"use client";
import { useSyncExternalStore } from "react";
import type { Session } from "@supabase/supabase-js";
import { supabase } from "./supabase";

export type SessionState =
  | { status: "loading"; session: null }
  | { status: "signed-out"; session: null }
  | { status: "signed-in"; session: Session };

let state: SessionState = { status: "loading", session: null };
const listeners = new Set<() => void>();
let started = false;

function set(next: SessionState) {
  state = next;
  listeners.forEach((l) => l());
}

function start() {
  if (started || typeof window === "undefined") return;
  started = true;
  supabase.auth.getSession()
    .then(({ data }) => set(data.session ? { status: "signed-in", session: data.session } : { status: "signed-out", session: null }))
    .catch(() => set({ status: "signed-out", session: null }));
  supabase.auth.onAuthStateChange((_event, session) => set(session ? { status: "signed-in", session } : { status: "signed-out", session: null }));
}

function subscribe(l: () => void) {
  start();
  listeners.add(l);
  return () => listeners.delete(l);
}

const server: SessionState = { status: "loading", session: null };

export function useSession(): SessionState {
  return useSyncExternalStore(subscribe, () => state, () => server);
}

/** Display name for the signed-in user. */
export function displayName(s: Session | null): string {
  if (!s) return "";
  return (s.user.user_metadata?.display_name as string | undefined) || s.user.email || "Signed in";
}

/** Two-letter initials for the account avatar. */
export function initials(s: Session | null): string {
  const n = displayName(s).replace(/@.*/, "").trim();
  const parts = n.split(/[\s._-]+/).filter(Boolean);
  return ((parts[0]?.[0] ?? "") + (parts[1]?.[0] ?? parts[0]?.[1] ?? "")).toUpperCase() || "?";
}
