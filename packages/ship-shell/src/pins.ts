//! Pin state persistence. Phase 2a: localStorage. Phase 2b+: daemon-side
//! shell preferences (workspace- or user-scoped).

import { defaultPinned } from "./view-catalog";

const PIN_KEY = "ship-rail-pins";

export function loadPins(): string[] {
  try {
    const raw = localStorage.getItem(PIN_KEY);
    if (!raw) return defaultPinned();
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed) || parsed.some((v) => typeof v !== "string")) {
      return defaultPinned();
    }
    return parsed as string[];
  } catch {
    return defaultPinned();
  }
}

export function savePins(pins: string[]): void {
  try {
    localStorage.setItem(PIN_KEY, JSON.stringify(pins));
  } catch {
    // Quota exceeded or disabled — persistence best-effort.
  }
}

export function pin(name: string): string[] {
  const current = loadPins();
  if (current.includes(name)) return current;
  const next = [...current, name];
  savePins(next);
  return next;
}

export function unpin(name: string): string[] {
  const current = loadPins();
  const next = current.filter((n) => n !== name);
  savePins(next);
  return next;
}
