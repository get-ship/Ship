// Regression test for directed-event delivery (L0 kernel
// "see_directed_events" opt-in).
//
// Directed events (target_actor_id set) bypass broadcast subscribers by
// kernel design; the event-stream actor opts in via
// ActorConfig.see_directed_events (see the StreamService.Events impl in
// apps/shipd/src/ops_rpc.rs). This test verifies a directed event
// (studio.chat_turn — fired by the session message endpoint with
// with_target) is persisted and observable by a subscriber.

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

        // Anchor the collection window BEFORE sending so we don't race
        // the emit (collectEvents only sees events persisted after it
        // starts).
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
