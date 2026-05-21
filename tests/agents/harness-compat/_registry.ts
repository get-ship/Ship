// Harness tier registry — single source of truth for which harnesses
// the compat suite exercises and at what bar.
//
// Add a sixth harness here (hermes, pi, amp, …) and every slice
// (spawn, events, hooks, pty) picks it up automatically. Removing
// a harness or moving tiers happens here too.

export type Tier = "T1" | "T2" | "T3";

export interface HarnessSpec {
  /** Harness id used by the daemon's adapter registry (matches
   *  `crates/core/ship-platform/src/services/harnesses/<id>.rs`). */
  id: string;
  /** Binary name to look up on PATH. */
  binary: string;
  /** Tier (drives skip-vs-fail policy). */
  tier: Tier;
  /** Whether the adapter parses a SessionStarted event from output.
   *  Today: only claude + codex. Cursor / gemini / opencode return
   *  None from parse_output → the events slice will skip the
   *  SessionStarted assertion for them and point at the hook-analytics
   *  follow-up. Source of truth: each adapter's `parse_output` impl. */
  emitsSessionStarted: boolean;
}

export const HARNESSES: HarnessSpec[] = [
  // ── T1: must pass on every commit ───────────────────────────────────────
  { id: "claude", binary: "claude", tier: "T1", emitsSessionStarted: true },
  { id: "codex", binary: "codex", tier: "T1", emitsSessionStarted: true },
  { id: "opencode", binary: "opencode", tier: "T1", emitsSessionStarted: false },

  // ── T3: evaluate-as-we-go; human owns drop decisions ────────────────────
  // `binary` mirrors the Rust adapter's binary() — CursorAdapter returns
  // "cursor-agent" (the `agent` alias symlinks to it). Keep in sync with
  // crates/core/ship-platform/src/services/harnesses/cursor.rs.
  { id: "cursor", binary: "cursor-agent", tier: "T3", emitsSessionStarted: false },
];

/** Harnesses skipped via SKIP_HARNESSES env (comma-separated ids). */
export function skipped(): Set<string> {
  return new Set(
    (process.env.SKIP_HARNESSES ?? "")
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean),
  );
}

/**
 * The agent profile we dispatch to in every compat test. Lightweight,
 * exists in .ship/agents/, doesn't require unusual perms.
 *
 * `shipwright` is the smallest agent that's not Commander; using
 * Commander would risk recursive dispatch under test.
 */
export const COMPAT_AGENT = "shipwright";

/** Workspace label to use for compat dispatches. Keeps test slugs
 *  recognisable in the mesh and away from real workspace names. */
export const COMPAT_WORKSPACE = "harness-compat";
