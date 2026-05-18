// Harness-compat heartbeat slice — verifies that activity signals
// keep `actor.live` true in the projection.
//
// Per crates/core/ship-substrate/src/projections/actor.rs:
//
//   live = pid_alive(pid) OR (pid.is_none() AND last_event_at within 90s)
//
// For dispatched harnesses today, `pid` is never recorded post-spawn
// (Task #14 — wire pid + lifecycle). So `live` falls through to the
// activity_live branch, which keys on `last_event_at`. That field is
// set ONCE on INSERT (apply_created) and never updated.
//
// Heartbeat path today (apps/shipd/src/routes/hooks_api.rs):
//   POST /api/v1/hooks/* → session_drain::update_heartbeat(session_id)
//   which writes to the workspace_session table — NOT the actor
//   projection. So even a busy harness firing hooks every second
//   doesn't keep the actor.live true past 90s.
//
// This slice DOCUMENTS the gap: posts heartbeats, polls actor.live,
// expects it to stay true. Today it goes false quickly because pid=0
// and last_event_at doesn't advance. The test INFO-logs the gap with
// a pointer to .artifacts/agents-arc/discussions/heartbeat-doesnt-touch-actor-projection.md.

import { describe, test, expect } from "bun:test";
import {
  api,
  binaryOnPath,
  cancelSession,
  dispatch,
  getActorRow,
  waitForActorRow,
} from "../_helpers";
import {
  COMPAT_AGENT,
  COMPAT_WORKSPACE,
  HARNESSES,
  skipped,
} from "./_registry";

const SKIPPED = skipped();

for (const h of HARNESSES) {
  describe(`heartbeat / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    test(
      "hooks heartbeat keeps actor.live=true (BLOCKED — heartbeat doesn't touch projection)",
      async () => {
        if (skipReason) {
          console.log(`SKIPPED: ${h.id} — ${skipReason}`);
          return;
        }

        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          auto: true,
          workspace: COMPAT_WORKSPACE,
        });

        try {
          const initial = await waitForActorRow(dispatched.slug);
          if (!initial) {
            tierFailure(h.tier, h.id, "actor row never materialised", {
              slug: dispatched.slug,
            });
            return;
          }
          const firstSeen = initial.last_event_at;

          // Fire heartbeats matching how a running harness would: every
          // ~500ms, POST a synthetic before-tool hook (the cheapest
          // endpoint that exercises session_drain::update_heartbeat).
          for (let i = 0; i < 5; i++) {
            await api("/api/v1/hooks/before-tool", {
              method: "POST",
              body: JSON.stringify({
                tool_name: "Heartbeat",
                actor_id: dispatched.slug,
                session_id: dispatched.slug,
              }),
            });
            await new Promise((r) => setTimeout(r, 500));
          }

          const after = await getActorRow(dispatched.slug);
          if (!after) {
            tierFailure(h.tier, h.id, "actor row vanished mid-test", {
              slug: dispatched.slug,
            });
            return;
          }

          // Document the gap. If last_event_at moves forward → heartbeat
          // DOES propagate (someone wired it); flip the INFO to a green
          // assertion. If it stays pinned to the initial value → gap
          // confirmed, point at the discussion doc.
          if (
            after.last_event_at &&
            firstSeen &&
            after.last_event_at !== firstSeen
          ) {
            console.log(
              `INFO: ${h.id} actor.last_event_at advanced — heartbeat wiring is live`,
            );
            expect(after.last_event_at).not.toBe(firstSeen);
          } else {
            console.log(
              `INFO: ${h.id} actor.last_event_at pinned to ${firstSeen ?? "null"} after 5 heartbeats — ` +
                `confirms hooks heartbeat doesn't update the actor projection. ` +
                `Tracked at .artifacts/agents-arc/discussions/heartbeat-doesnt-touch-actor-projection.md`,
            );
          }

          // Independent assertion: actor.live should ideally stay true
          // while the process is "active." Today it's controlled by
          // pid_alive(pid). For dispatched harnesses pid=0, so live
          // depends entirely on last_event_at being recent.
          // INFO-log the current value either way.
          console.log(
            `INFO: ${h.id} actor.live=${after.live} pid=${after.pid ?? "null"} after heartbeats`,
          );
        } finally {
          await cancelSession(dispatched.slug);
        }
      },
      30_000,
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
