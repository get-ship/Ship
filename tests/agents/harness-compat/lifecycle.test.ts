// Harness-compat lifecycle slice — full birth-to-death event sequence.
//
// For each T1 harness binary on PATH: dispatch, cancel, then poll the
// event log for the expected sequence:
//
//   spawn-phase events (any order; some are emitted on a separate
//   thread, so strict ordering would be flaky):
//     agent.session.started    ← daemon dispatch handler (the primitive)
//     agent.session.dispatched ← orchestrator after executor.spawn
//     process.spawned          ← orchestrator after spawn registration
//     actor.created            ← RoutingExecutor::spawn (fire-and-forget thread)
//
//   teardown-phase events (after cancel):
//     agent.session.cancelled  ← cancel route (agent_sessions projection)
//     process.killed           ← orchestrator handlers::handle_job_cancelled
//
// We don't assert strict order (actor.created is on its own thread;
// process.spawned races it). We assert all required types appear
// scoped to this dispatch.
//
// Subsumes the older "events.test.ts" task — broader signal, same
// observability goal.

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

const SPAWN_PHASE_EVENTS = [
  "agent.session.started",
  "agent.session.dispatched",
  "process.spawned",
  "actor.created",
] as const;

const TEARDOWN_PHASE_EVENTS = [
  "agent.session.cancelled",
  "process.killed",
] as const;

for (const h of HARNESSES) {
  describe(`lifecycle / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    test(
      "dispatch → spawn-phase events → cancel → teardown-phase events",
      async () => {
        if (skipReason) {
          console.log(`SKIPPED: ${h.id} — ${skipReason}`);
          return;
        }

        const sinceId = await latestEventId();
        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          workspace: COMPAT_WORKSPACE,
        });

        try {
          await waitForActorRow(dispatched.slug);
          // Give the fire-and-forget actor.created emit a beat to land.
          await new Promise((r) => setTimeout(r, 500));
        } finally {
          await cancelSession(dispatched.slug);
        }

        // Poll for all events scoped to this dispatch since the cursor.
        // Window covers both phases (spawn already happened by now;
        // teardown fires within ~2s of cancel).
        const events = await pollForEvents(
          (e) =>
            e.entity_id === dispatched.slug ||
            e.entity_id === dispatched.sessionId ||
            e.actor_id === dispatched.slug ||
            e.target_actor_id === dispatched.slug,
          {
            sinceId,
            slug: dispatched.slug,
            timeoutMs: 8_000,
          },
        );

        const types = new Set(events.map((e) => e.event_type));

        // ── Spawn-phase coverage ────────────────────────────────────────
        const missingSpawn = SPAWN_PHASE_EVENTS.filter((t) => !types.has(t));
        if (missingSpawn.length > 0) {
          tierFailure(
            h.tier,
            h.id,
            `missing spawn-phase events: ${missingSpawn.join(", ")}`,
            { observed: [...types].sort().join(",") },
          );
          return;
        }

        // ── Teardown-phase coverage ────────────────────────────────────
        // process.killed is the load-bearing one; process.exited may
        // also fire from the supervisor's exit detection. Accept either
        // as evidence the process actually went away.
        const missingTeardown = TEARDOWN_PHASE_EVENTS.filter(
          (t) => !types.has(t),
        );
        if (missingTeardown.length > 0) {
          tierFailure(
            h.tier,
            h.id,
            `missing teardown-phase events: ${missingTeardown.join(", ")}`,
            { observed: [...types].sort().join(",") },
          );
          return;
        }

        // All required events present — slice green.
        expect(missingSpawn.length).toBe(0);
        expect(missingTeardown.length).toBe(0);
      },
      45_000,
    );
  });
}

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
    expect.unreachable(`T1-FAIL: ${msg}`);
  }
  if (tier === "T2") {
    console.warn(`T2-FLAKY: ${msg}`);
    return;
  }
  console.warn(`T3-INFO: ${msg}`);
}
