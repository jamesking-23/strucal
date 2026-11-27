"use client";
import { useCallback, useEffect, useState } from "react";
import { loadEngine, type Engine } from "./engine";

/** Loads the WebAssembly engine. `retry` re-attempts after a failed load. */
export function useEngine() {
  const [engine, setEngine] = useState<Engine | null>(null);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let live = true;
    setError("");
    loadEngine().then((e) => { if (live) setEngine(e); }).catch((e: Error) => { if (live) setError(e.message); });
    return () => { live = false; };
  }, [attempt]);
  const retry = useCallback(() => setAttempt((a) => a + 1), []);
  return { engine, error, retry };
}
