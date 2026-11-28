// localStorage access that tolerates private browsing, quota errors and corrupt data.
export function readJson<T>(key: string): T | null {
  try {
    const s = window.localStorage.getItem(key);
    return s ? (JSON.parse(s) as T) : null;
  } catch {
    return null;
  }
}

/** Returns false when the value could not be stored. */
export function writeJson(key: string, value: unknown): boolean {
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
}
