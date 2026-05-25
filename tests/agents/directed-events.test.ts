// Regression test for the L0 kernel "see_directed_events" opt-in.
//
// Before fix: directed events (target_actor_id set) were invisible to
// the SSE broadcast stream — by kernel design. The SSE endpoint now
// opts in via ActorConfig.see_directed_events; this test verifies a
// directed event (studio.chat_turn — fired by the session message
// endpoint with with_target) reaches the SSE subscriber.
//
// See .artifacts/agents-arc/discussions/directed-events-invisible-to-sse.md
// for the discussion that drove the fix.

import { describe, test, expect } from "bun:test";
import {
  cancelSession,
  collectEvents,
  dispatch,
  sendSessionMessage,
  waitForActorRow,
} from "./_helpers";

describe("kernel / directed events visible to SSE", () => {
  test(
    "studio.chat_turn (directed at session) reaches the SSE stream",
    async () => {
      const dispatched = await dispatch({
        agent: "shipwright",
        harness: "claude",
        workspace: "directed-evt",
      });

      try {
        await waitForActorRow(dispatched.slug);

        // collectEvents uses /api/v1/events/stream (SSE) — the opt-in
        // path. Open BEFORE sending so we don't race the emit.
        const collectPromise = collectEvents(
          (e) =>
            e.event_type === "studio.chat_turn" &&
            (e.entity_id === dispatched.slug ||
              e.target_actor_id === dispatched.slug),
          { windowMs: 6_000, stopAfter: 1 },
        );
        // SSE connection needs a beat to open.
        await new Promise((r) => setTimeout(r, 200));

        const send = await sendSessionMessage(
          dispatched.slug,
          "ship-test directed-events probe",
          [],
        );
        expect(send.status).toBeGreaterThanOrEqual(200);
        expect(send.status).toBeLessThan(300);

        const events = await collectPromise;
        // Before the kernel fix: events.length === 0 (SSE missed the
        // directed event). After: 1.
        expect(events.length).toBeGreaterThanOrEqual(1);
        const e = events[0]!;
        expect(e.target_actor_id).toBe(dispatched.slug);
      } finally {
        await cancelSession(dispatched.slug);
      }
    },
    30_000,
  );
});
