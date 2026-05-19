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
        // filters in-process. Server-side slug filter narrows the
        // response so we don't have to walk thousands of unrelated
        // events on a busy daemon. The poll endpoint matches slug
        // against entity_id OR payload.slug — our typed payload
        // doesn't carry a `slug` field today, so we rely on entity_id.
        const events = await pollForEvents(
          (e) =>
            e.event_type === c.eventType &&
            (e.entity_id === probeActor ||
              e.target_actor_id === probeActor ||
              e.actor_id === probeActor),
          {
            sinceId,
            slug: probeActor,
            timeoutMs: 10_000,
            stopAfter: 1,
          },
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

        const sinceId = await latestEventId();
        // Opencode hook synth fires from the structured output stream
        // of `opencode run --format json`, which only flows when auto
        // (headless) is true. For other harnesses we use auto:false to
        // keep them interactive (then cancel) — they fire hooks from
        // their own .hooks config regardless.
        const auto = h.id === "opencode";
        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          auto,
          workspace: COMPAT_WORKSPACE,
          prompt: auto ? "say hi" : undefined,
        });

        try {
          await waitForActorRow(dispatched.slug);
          await new Promise((r) => setTimeout(r, 2_000));
          if (!auto) await cancelSession(dispatched.slug);

          // Longer window for auto:true — the harness has to boot,
          // process the prompt, and emit events. Hard-assert for
          // opencode (T1 reliability gate); INFO-log for others where
          // keepalive Channel delivery is still the unresolved gap.
          const events = await pollForEvents(
            (e) =>
              (e.actor_id === dispatched.slug ||
                e.target_actor_id === dispatched.slug ||
                e.entity_id === dispatched.slug) &&
              e.event_type.startsWith("hook."),
            { sinceId, slug: dispatched.slug, timeoutMs: auto ? 30_000 : 5_000 },
          );

          if (h.id === "opencode") {
            // Hard assertion — the supervisor synthesizes hook.* from
            // opencode's --format json stream. session_started + at
            // least one tool_use_started must fire.
            const types = events.map((e) => e.event_type);
            expect(types).toContain("hook.session_started");
            expect(types).toContain("hook.tool_use_started");
          } else {
            if (events.length === 0) {
              console.log(
                `INFO: ${h.id} — no hook.* events observed for dispatched session ${dispatched.slug}. ` +
                  `Expected today: dispatch routes to build_headless (no harness session → no SessionStart). ` +
                  `Unblocks when keepalive dispatch mode lands. ` +
                  `See .artifacts/agents-arc/discussions/dispatch-modes-vs-session-hooks.md`,
              );
              return;
            }
            const types = [...new Set(events.map((e) => e.event_type))];
            console.log(`INFO: ${h.id} observed hook events: ${types.join(", ")}`);
          }
        } finally {
          await cancelSession(dispatched.slug);
        }
      },
      60_000,
    );
  });
}
