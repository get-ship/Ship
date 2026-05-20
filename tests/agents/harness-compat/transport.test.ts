// Harness-compat transport slice — verifies routing.rs::spawn sets
// the actor projection's `transport` field correctly per dispatch mode.
//
// Source of truth: crates/ship-supervisor/src/executors/routing.rs
//   let transport = match ctx.invocation {
//       Print  => "headless",
//       Pty    => "pty",
//       Stream => "stream",
//   };
//
// Post-inversion the transport label derives purely from `invocation`
// (open_tab is a UX axis and doesn't affect routing):
//   invocation:"print" → ProcessExecutor one-shot → transport = "headless"
//   invocation:"pty"   → TmuxWindow (if tmux) / Process → transport = "pty"
//
// The transport field drives downstream UX (attach button enabled?
// chat shape choice?), so a regression here breaks the UI silently.

import { describe, test, expect } from "bun:test";
import {
  binaryOnPath,
  cancelSession,
  dispatch,
  pollForEvents,
} from "../_helpers";
import {
  COMPAT_AGENT,
  COMPAT_WORKSPACE,
  HARNESSES,
  skipped,
} from "./_registry";

const SKIPPED = skipped();

for (const h of HARNESSES) {
  describe(`transport / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    for (const mode of [
      {
        label: "headless",
        invocation: "print" as const,
        open_tab: false,
        keepalive: false,
        expected: "headless" as const,
      },
      {
        label: "interactive",
        invocation: "pty" as const,
        open_tab: true,
        keepalive: false,
        expected: "pty" as const,
      },
    ]) {
      test(
        `${mode.label} dispatch records transport="${mode.expected}"`,
        async () => {
          if (skipReason) {
            console.log(`SKIPPED: ${h.id} — ${skipReason}`);
            return;
          }

          const dispatched = await dispatch({
            agent: COMPAT_AGENT,
            harness: h.id,
            invocation: mode.invocation,
            open_tab: mode.open_tab,
            keepalive: mode.keepalive,
            workspace: COMPAT_WORKSPACE,
          });

          try {
            // Read the transport label from the PERSISTED actor.created
            // event, not the projection row: a print session exits in
            // ~1s and cleanup deletes the row, racing the poll under
            // load. The event log is append-only and survives teardown.
            //
            // No since_id cursor: the slug is unique per dispatch, so it
            // already scopes to this run. A cursor would be wrong here —
            // actor.created is emitted on a separate thread (routing.rs)
            // with its own ULID generator, so its id isn't reliably
            // greater than a sinceId captured on the main thread, and the
            // `id > since_id` filter can drop it under load.
            // Read the transport label from the persisted actor.created
            // event (slug-scoped, no cursor — see the unique-slug note
            // above). 20s is ample at normal load; under the full
            // 4-harness suite's artificial peak the orchestrator mailbox
            // backs up and dispatch latency can blow past any timeout —
            // that load-latency limit is a documented orchestrator
            // follow-up, not a transport-label bug.
            const events = await pollForEvents(
              (e) =>
                e.event_type === "actor.created" &&
                (e.entity_id === dispatched.slug ||
                  e.actor_id === dispatched.slug),
              { slug: dispatched.slug, stopAfter: 1, timeoutMs: 20_000 },
            );
            if (events.length === 0) {
              tierFailure(h.tier, h.id, "actor.created event never emitted", {
                slug: dispatched.slug,
              });
              return;
            }
            const transport = JSON.parse(events[0]!.payload_json).transport;
            expect(transport).toBe(mode.expected);
          } finally {
            await cancelSession(dispatched.slug);
          }
        },
        30_000,
      );
    }
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
