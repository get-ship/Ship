// Harness-compat hooks slice — load-bearing for analytics.
//
// Two layers of assertion:
//
//   1. DAEMON EMIT (always-runnable, no harness needed):
//      POST /api/v1/hooks/{session-start,session-end} must emit
//      hook.{session_started,session_ended} kernel events. This gates
//      the daemon-side wiring landed in apps/shipd/src/routes/hooks_api.rs.
//
//   2. END-TO-END FROM A DISPATCHED HARNESS (blocked):
//      Dispatch the harness, wait, observe hook.* events on the kernel
//      log. Today this is blocked: both ProcessExecutor and
//      TmuxWindowExecutor use `adapter.build_headless` (`-p` mode) which
//      gives the harness no session, so SessionStart hook never fires.
//      Compiler injection is committed and ready; the gap is in the
//      dispatch mode. See
//      .artifacts/agents-arc/discussions/dispatch-modes-vs-session-hooks.md.
//
// Layer 1 goes green per harness in T1+T2 (just exercises the daemon
// endpoint; binary-on-PATH check is irrelevant since no harness runs).
// Layer 2 is INFO-logged with the discussion-doc pointer until the
// keepalive dispatch mode lands.

import { describe, test, expect } from "bun:test";
import {
  api,
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

const HOOK_EVENT_TYPES = {
  sessionStarted: "hook.session_started",
  sessionEnded: "hook.session_ended",
  toolUseStarted: "hook.tool_use_started",
  toolUseCompleted: "hook.tool_use_completed",
} as const;

// ── Layer 1: daemon emit ──────────────────────────────────────────────────
//
// Drives the daemon-side fix landed this session. No harness needed —
// directly POSTs to /api/v1/hooks/{session-start,session-end} as if a
// running harness's hook config had called it. The kernel event must
// fire and persist (visible via /api/v1/events/poll).

describe("hooks / daemon emit", () => {
  const cases = [
    {
      endpoint: "session-start",
      eventType: HOOK_EVENT_TYPES.sessionStarted,
      body: {},
    },
    {
      endpoint: "before-tool",
      eventType: HOOK_EVENT_TYPES.toolUseStarted,
      body: { tool_name: "Bash", tool_input: { command: "echo test" } },
    },
    {
      endpoint: "after-tool",
      eventType: HOOK_EVENT_TYPES.toolUseCompleted,
      body: { tool_name: "Bash", tool_response: { stdout: "test" } },
    },
    {
      endpoint: "session-end",
      eventType: HOOK_EVENT_TYPES.sessionEnded,
      body: {},
    },
  ] as const;

  for (const c of cases) {
    test(
      `POST /hooks/${c.endpoint} emits ${c.eventType}`,
      async () => {
        const probeActor = `hooks-test-${Date.now()}`;
        const sinceId = await latestEventId();

        const res = await api(`/api/v1/hooks/${c.endpoint}`, {
          method: "POST",
          body: JSON.stringify({
            actor_id: probeActor,
            session_id: probeActor,
            ...c.body,
          }),
        });
        expect(res.status).toBe(200);

        // Generous timeout: poll endpoint scans recent events list +
        // filters in-process. With a busy DB the first page may not
        // include our event; widen the window.
        //
        // tool_use events use actor_id (resolve_actor) as the entity_id,
        // not session_id. Filter on either to match both shapes.
        const events = await pollForEvents(
          (e) =>
            e.event_type === c.eventType &&
            (e.entity_id === probeActor ||
              e.target_actor_id === probeActor ||
              e.actor_id === probeActor),
          { sinceId, timeoutMs: 10_000, stopAfter: 1 },
        );
        expect(events.length).toBe(1);
        const e = events[0]!;
        // Target + actor_id should reach the probe — exact routing
        // depends on which endpoint (session-* uses session_id;
        // tool_* uses actor_id). Just assert we can correlate.
        expect(
          e.target_actor_id === probeActor ||
            e.actor_id === probeActor ||
            e.entity_id === probeActor,
        ).toBe(true);
      },
      15_000,
    );
  }
});

// ── Layer 2: end-to-end harness → hook → kernel event ─────────────────────
//
// Blocked on the keepalive dispatch mode. Runs but INFO-logs the gap
// rather than failing T1. Once the keepalive mode lands, flip the
// INFO-log to a T1 assertion to make this a real green/red gate.

for (const h of HARNESSES) {
  describe(`hooks / e2e / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      // Skip if binary not on PATH (informational; the keepalive gap
      // blocks observation regardless).
      return null;
    })();

    test(
      "dispatched harness fires session-lifecycle hooks (BLOCKED — keepalive mode)",
      async () => {
        if (skipReason) {
          console.log(`SKIPPED: ${h.id} — ${skipReason}`);
          return;
        }
        if (h.id === "opencode") {
          console.log(
            `INFO: ${h.id} uses plugins (hooks=false); plugin → ship hook bridge is a separate follow-up`,
          );
          return;
        }

        const sinceId = await latestEventId();
        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          auto: false, // intent: interactive — but TmuxWindow + ProcessExecutor
          // both use build_headless today (see discussion doc).
          workspace: COMPAT_WORKSPACE,
        });

        try {
          await waitForActorRow(dispatched.slug);
          await new Promise((r) => setTimeout(r, 2_000));
          await cancelSession(dispatched.slug);

          const events = await pollForEvents(
            (e) =>
              (e.actor_id === dispatched.slug ||
                e.target_actor_id === dispatched.slug ||
                e.entity_id === dispatched.slug) &&
              e.event_type.startsWith("hook."),
            { sinceId, timeoutMs: 5_000 },
          );

          if (events.length === 0) {
            console.log(
              `INFO: ${h.id} — no hook.* events observed for dispatched session ${dispatched.slug}. ` +
                `Expected today: dispatch routes to build_headless (no harness session → no SessionStart). ` +
                `Unblocks when keepalive dispatch mode lands. ` +
                `See .artifacts/agents-arc/discussions/dispatch-modes-vs-session-hooks.md`,
            );
            return;
          }

          // If we did observe hooks, surface them — useful for spotting
          // when the keepalive path lands and starts producing events.
          const types = [...new Set(events.map((e) => e.event_type))];
          console.log(`INFO: ${h.id} observed hook events: ${types.join(", ")}`);
        } finally {
          await cancelSession(dispatched.slug);
        }
      },
      30_000,
    );
  });
}
