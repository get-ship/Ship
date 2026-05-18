// Harness-compat hooks slice — load-bearing for analytics.
//
// For each supported harness: dispatch and observe the hook-driven
// lifecycle events Ship needs for "every agent visible, tool usage
// traceable" (agents-arc intent.md). The kernel events these tests
// look for are emitted by apps/shipd/src/routes/hooks_api.rs when
// the harness's hook config fires `ship hook <event>` (or for Claude:
// when its HTTP hook config POSTs directly).
//
// TDD posture: this slice is INTENTIONALLY RED for the known gaps
// documented in .artifacts/agents-arc/intent.md:
//
//   - Claude: missing SessionStart / SessionEnd kernel events
//   - Codex: missing SessionStart / SessionEnd kernel events
//   - Cursor: parse_output is a no-op → no lifecycle events
//   - Gemini: parse_output is a no-op → no lifecycle events
//   - OpenCode: hooks=false in caps (uses plugins, different shape)
//
// Tier policy applies: T1 throws (drives the work), T2 warns, T3 warns.

import { describe, test, expect } from "bun:test";
import {
  binaryOnPath,
  cancelSession,
  dispatch,
  latestEventId,
  pollForEvents,
  waitForActorRow,
} from "../_helpers";
import {
  COMPAT_AGENT,
  COMPAT_WORKSPACE,
  HARNESSES,
  skipped,
} from "./_registry";

const SKIPPED = skipped();

// Hook kernel events the slice asserts on. Source of truth:
// apps/shipd/src/routes/hooks_api.rs (hook_*) and any equivalent emitter
// for session-start / session-end (today: not emitted as kernel events —
// the endpoint just updates session_drain heartbeat; that's a gap this
// slice is meant to surface and drive).
const HOOK_EVENT_TYPES = {
  sessionStarted: "hook.session_started",
  sessionEnded: "hook.session_ended",
  toolUseStarted: "hook.tool_use_started",
  toolUseCompleted: "hook.tool_use_completed",
} as const;

for (const h of HARNESSES) {
  describe(`hooks / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    test(
      "session-start / session-end hooks emit kernel events",
      async () => {
        if (skipReason) {
          console.log(`SKIPPED: ${h.id} — ${skipReason}`);
          return;
        }

        // OpenCode uses a plugin model rather than the harness-native
        // hook file shape. Plugins should still be able to call back
        // into Ship via `ship hook` or an equivalent — but until that
        // wiring lands, opencode can't produce session-start/end kernel
        // events. Document and skip.
        if (h.id === "opencode") {
          console.log(
            `INFO: ${h.id} uses plugins (hooks=false); plugin → ship hook bridge is a separate follow-up`,
          );
          return;
        }

        // Cursor BEFORE dispatch — hook.* events are directed (target =
        // actor id) so SSE wouldn't see them. Poll the persisted log.
        const sinceId = await latestEventId();
        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          auto: true,
          workspace: COMPAT_WORKSPACE,
        });

        try {
          // Allow time for the harness to spawn + fire its session-start
          // hook. The actor row materialising is a coarse signal that
          // the binary is up.
          await waitForActorRow(dispatched.slug);

          // Give session-start a beat to fire, then cancel so session-end
          // can fire within the collection window.
          await new Promise((r) => setTimeout(r, 2_000));
          await cancelSession(dispatched.slug);

          const events = await pollForEvents(
            (e) =>
              (e.actor_id === dispatched.slug ||
                e.target_actor_id === dispatched.slug ||
                e.entity_id === dispatched.slug) &&
              e.event_type.startsWith("hook."),
            { sinceId, timeoutMs: 8_000 },
          );
          const types = new Set(events.map((e) => e.event_type));

          // ── SessionStart kernel event ───────────────────────────────
          if (!types.has(HOOK_EVENT_TYPES.sessionStarted)) {
            tierFailure(
              h.tier,
              h.id,
              `no ${HOOK_EVENT_TYPES.sessionStarted} observed within window`,
              { slug: dispatched.slug, observed: [...types] },
            );
          }

          // ── SessionEnd kernel event ─────────────────────────────────
          if (!types.has(HOOK_EVENT_TYPES.sessionEnded)) {
            tierFailure(
              h.tier,
              h.id,
              `no ${HOOK_EVENT_TYPES.sessionEnded} observed within window`,
              { slug: dispatched.slug, observed: [...types] },
            );
          }
        } finally {
          // Cancel is idempotent; safe to call twice if the inner path
          // didn't reach it.
          await cancelSession(dispatched.slug);
        }
      },
      30_000,
    );
  });
}

/** Tier-graded failure shared with spawn.test.ts. T1 throws, T2/T3 warn. */
function tierFailure(
  tier: "T1" | "T2" | "T3",
  harness: string,
  reason: string,
  details: Record<string, unknown>,
): void {
  const detailStr = Object.entries(details)
    .map(([k, v]) => `${k}=${typeof v === "string" ? v : JSON.stringify(v)}`)
    .join(" ");
  const msg = `${harness}: ${reason} (${detailStr})`;
  if (tier === "T1") {
    // expect() throws synchronously; using a thrown Error lets the
    // caller see the full message instead of "Expected true received false".
    expect.unreachable(`T1-FAIL: ${msg}`);
  }
  if (tier === "T2") {
    console.warn(`T2-FLAKY: ${msg}`);
    return;
  }
  console.warn(`T3-INFO: ${msg}`);
}
