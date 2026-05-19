// Analytics integration slice — proves the actor.created fallback fires
// end-to-end for real dispatched sessions, not just unit-tested.
//
// Unit tests in crates/ship-analytics/src/tests_projection.rs cover the
// projection apply logic. This slice closes the integration gap: dispatch
// → supervisor emits actor.created → analytics_sessions row materialises
// → HTTP endpoint returns it with harness populated.
//
// Why this matters: dispatched keepalive harnesses may not fire
// hook.session_started reliably (see
// claude-session-start-hook-not-firing.md). actor.created is the
// load-bearing fallback. If the unit→integration wire is broken, the
// fallback is theoretical only.

import { describe, test, expect } from "bun:test";

import {
  api,
  binaryOnPath,
  cancelSession,
  dispatch,
  waitForActorRow,
} from "./_helpers";
import { COMPAT_AGENT, COMPAT_WORKSPACE } from "./harness-compat/_registry";

type Envelope<T> = { ok: boolean; data: T };

interface SessionRow {
  session_key: string;
  actor_id: string;
  harness: string | null;
  model: string | null;
  started_at: string;
  ended_at: string | null;
}

async function getAnalyticsSession(key: string): Promise<SessionRow | null> {
  const res = await api<Envelope<{ session?: SessionRow }>>(
    `/api/v1/analytics/sessions/${key}`,
  );
  if (res.status !== 200) return null;
  return res.body.data.session ?? null;
}

async function waitFor<T>(
  poll: () => Promise<T | null>,
  timeoutMs: number,
  intervalMs = 250,
): Promise<T | null> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const v = await poll();
    if (v) return v;
    await new Promise((r) => setTimeout(r, intervalMs));
  }
  return null;
}

describe("analytics integration / actor.created fallback", () => {
  test(
    "dispatched claude session lands in analytics_sessions with harness=claude",
    async () => {
      if (!binaryOnPath("claude")) {
        console.log("SKIPPED: claude binary not on PATH");
        return;
      }

      const dispatched = await dispatch({
        agent: COMPAT_AGENT,
        harness: "claude",
        auto: true,
        workspace: COMPAT_WORKSPACE,
      });

      try {
        // The actor projection must show the row before we expect
        // analytics — actor.created flows through both projections.
        const actor = await waitForActorRow(dispatched.slug);
        expect(actor).not.toBeNull();
        expect(actor!.harness).toBe("claude");

        // analytics_sessions is populated by the same event. Allow a
        // generous window for projection apply.
        const row = await waitFor(
          () => getAnalyticsSession(dispatched.slug),
          15_000,
        );
        expect(row).not.toBeNull();
        expect(row!.session_key).toBe(dispatched.slug);
        expect(row!.actor_id).toBe(dispatched.slug);
        expect(row!.harness).toBe("claude");
        expect(row!.started_at.length).toBeGreaterThan(0);
      } finally {
        await cancelSession(dispatched.slug);
      }
    },
    45_000,
  );

  test(
    "session lands in /api/v1/analytics/sessions list",
    async () => {
      if (!binaryOnPath("claude")) {
        console.log("SKIPPED: claude binary not on PATH");
        return;
      }

      const dispatched = await dispatch({
        agent: COMPAT_AGENT,
        harness: "claude",
        auto: true,
        workspace: COMPAT_WORKSPACE,
      });

      try {
        await waitForActorRow(dispatched.slug);
        const found = await waitFor(async () => {
          const res = await api<
            Envelope<{ sessions: { session_key: string }[] }>
          >("/api/v1/analytics/sessions?limit=50");
          return res.body.data.sessions.find(
            (s) => s.session_key === dispatched.slug,
          )
            ? true
            : null;
        }, 15_000);
        expect(found).toBe(true);
      } finally {
        await cancelSession(dispatched.slug);
      }
    },
    45_000,
  );
});
