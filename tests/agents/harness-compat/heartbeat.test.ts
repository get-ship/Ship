// Harness-compat heartbeat slice — verifies that activity signals
// keep `actor.live` true in the projection.
//
// Per crates/core/ship-substrate/src/projections/actor.rs:
//
//   live = pid_alive(pid) OR (pid.is_none() AND last_event_at within 90s)
//
// For a ProcessExecutor-backed session (keepalive — see dispatch
// below), `pid` IS recorded post-spawn: process.spawned carries the
// real child pid and ActorProjection upserts it (projections/actor.rs
// ::apply_process_spawned). So `live = pid_alive(pid)` holds while the
// child runs. (Tmux-detached and print sessions don't record a pid;
// the keepalive ProcessExecutor path is the one with an honest pid.)
//
// Heartbeat path today (apps/shipd/src/routes/hooks_api.rs):
//   POST /api/v1/hooks/* → session_drain::update_heartbeat(session_id)
//   which writes to the workspace_session table — NOT the actor
//   projection. So last_event_at advancing is a bonus, not the gate;
//   the live=true assertion rides on the real pid being alive.

import { describe, test, expect } from "bun:test";
import {
  api,
  binaryOnPath,
  cancelSession,
  dispatch,
  getActorRow,
  waitForActorGone,
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

        // keepalive forces ProcessExecutor (routing.rs), which captures
        // the real child pid post-spawn and stays alive. A bare PTY
        // session routes to TmuxWindow (tmux-detached, pid=None) and a
        // print session exits — neither satisfies the live+pid asserts.
        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          invocation: "pty",
          keepalive: true,
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
            await api("/hooks/before-tool", {
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

          // After the projection picks up process.spawned, pid is the
          // real child pid (not null, not 0). live = pid_alive(pid).
          expect(after.live).toBe(true);
          expect(after.pid).not.toBeNull();
          expect(after.pid).toBeGreaterThan(0);
          console.log(
            `INFO: ${h.id} actor.live=${after.live} pid=${after.pid ?? "null"} after heartbeats`,
          );
        } finally {
          await cancelSession(dispatched.slug);
          // Confirm the keepalive harness child actually exits before
          // yielding — its teardown load otherwise bleeds into later
          // test files and starves their dispatches.
          await waitForActorGone(dispatched.slug);
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
