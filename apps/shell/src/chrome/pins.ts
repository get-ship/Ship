//! Pin state persistence. Lives in daemon-side user settings under the
//! `shell.rail.pinned` key (tier-2 storage per the framework handoff
//! storage-tier model: user prefs, HTTP-API accessed, cross-device
//! syncable, surfaced in the settings view).
//!
//! The API is deliberately async + optimistic: the rail shows
//! `DEFAULT_PINS` while the daemon fetch resolves, then updates in place.
//! Writes fire-and-forget; a failed PUT logs but doesn't roll back the
//! local state (the next fetchPins() will re-sync).

import { DEFAULT_PINS } from "./view-catalog";

const SETTING_KEY = "shell.rail.pinned";

export async function fetchPins(): Promise<string[]> {
  try {
    const resp = await fetch("/api/v1/settings", { credentials: "same-origin" });
    if (!resp.ok) return DEFAULT_PINS;
    const body = (await resp.json()) as { data?: Record<string, unknown> };
    const raw = body.data?.[SETTING_KEY];
    if (typeof raw === "string") {
      try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed) && parsed.every((v) => typeof v === "string")) {
          return parsed as string[];
        }
      } catch {
        /* fall through to default */
      }
    } else if (Array.isArray(raw) && raw.every((v) => typeof v === "string")) {
      return raw as string[];
    }
    return DEFAULT_PINS;
  } catch {
    return DEFAULT_PINS;
  }
}

export async function savePins(pins: string[]): Promise<void> {
  try {
    await fetch("/api/v1/settings", {
      method: "PUT",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key: SETTING_KEY, value: JSON.stringify(pins) }),
    });
  } catch (err) {
    // Log + swallow. Next fetchPins() reconciles.
    // eslint-disable-next-line no-console
    console.warn("[ship-shell] savePins failed:", err);
  }
}

export async function pin(name: string, current: string[]): Promise<string[]> {
  if (current.includes(name)) return current;
  const next = [...current, name];
  await savePins(next);
  return next;
}

export async function unpin(
  name: string,
  current: string[],
): Promise<string[]> {
  const next = current.filter((n) => n !== name);
  await savePins(next);
  return next;
}
