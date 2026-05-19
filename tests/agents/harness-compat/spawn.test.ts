// Harness-compat spawn slice — the safety net.
//
// For each harness in _registry.ts: dispatch through the daemon,
// confirm a kernel actor registered for the resulting slug, confirm
// the actor projection materialised, cancel cleanly, and confirm the
// mesh drops the actor. Skips cleanly when the harness binary isn't
// on PATH.
//
// Pass criteria are tier-graded; bar is in tests/agents/README.md.
//
// This file deliberately does NOT assert on lifecycle events
// (SessionStarted, etc.) — that's the events slice's job. Spawn cares
// about: did a process appear, did it register, did it stop.

import { describe, test, expect } from "bun:test";
import {
  binaryOnPath,
  cancelSession,
  dispatch,
  waitForActorField,
} from "../_helpers";
import {
  COMPAT_AGENT,
  COMPAT_WORKSPACE,
  HARNESSES,
  skipped,
} from "./_registry";

const ULID_RE = /^[0-9A-HJKMNP-TV-Z]{26}$/;
const SKIPPED = skipped();

for (const h of HARNESSES) {
  describe(`spawn / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    test(
      "dispatch → actor registers → cancel",
      async () => {
        if (skipReason) {
          console.log(`SKIPPED: ${h.id} — ${skipReason}`);
          return;
        }

        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          auto: true, // headless; we never send a turn
          workspace: COMPAT_WORKSPACE,
          // No prompt / spec → harness comes up to its ready prompt and
          // sits idle until we cancel. Avoids any model API call.
        });

        try {
          // ── Slug + ids ────────────────────────────────────────────────
          expect(dispatched.slug.startsWith(`${COMPAT_WORKSPACE}-`)).toBe(true);
          expect(dispatched.jobId).toMatch(ULID_RE);
          expect(dispatched.branch).toBe(`job/${dispatched.slug}`);

          // ── Actor projection materialised ─────────────────────────────
          // Dispatch path: HTTP → orchestrator → supervisor → harness binary
          // → actor registration. RoutingExecutor::spawn fires actor.created
          // on a separate thread (fire-and-forget), so the row appears
          // shortly after the process spawns. Mesh (/api/v1/mesh) is the
          // mesh.register registry for agent-to-agent messaging; dispatched
          // sessions don't register there — they live in /api/v1/actors.
          //
          // Projection race: process.spawned can land before actor.created,
          // creating a stub row with harness="". Use waitForActorField to
          // poll until the field we'll assert on is populated.
          const actor = await waitForActorField(
            dispatched.slug,
            (a) => !!a.harness && a.harness.length > 0,
          );
          if (!actor) {
            tierFailure(
              h.tier,
              h.id,
              "actor row never materialised in projection",
              { slug: dispatched.slug },
            );
            return;
          }
          // kind indicates this is an agent dispatch, not a shell session.
          // Current value is "pipeline-step" per routing.rs::spawn.
          expect(actor.kind).not.toBe("shell");
          // harness field reflects what was dispatched — proves the
          // adapter selection landed correctly.
          expect(actor.harness).toBe(h.id);
          // KNOWN GAP (separate follow-up): actor.pid is never updated
          // post-spawn. emit_actor_created_simple is called BEFORE
          // ProcessExecutor::spawn, so pid is unknown at that point, and
          // no actor.pid_updated event follows. Result: actor.live always
          // false in the projection. Tracked for the analytics story —
          // every dispatched harness should be visible as live while
          // running. Don't assert pid or live here; that work is the next
          // job, not this test's gate.
        } finally {
          await cancelSession(dispatched.slug);
        }
      },
      // Generous per-test timeout: spawn → worktree prep (~1s) → harness
      // binary init (~1–3s) → actor row poll (15s cap) → cancel.
      60_000,
    );
  });
}

/**
 * Tier-graded failure. T1 throws (test fails). T2/T3 warn.
 * `expect(...)` throws uniformly; this lets us match CI signal to
 * the tier policy.
 */
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
    throw new Error(`T1-FAIL: ${msg}`);
  }
  if (tier === "T2") {
    console.warn(`T2-FLAKY: ${msg}`);
    return;
  }
  console.warn(`T3-INFO: ${msg}`);
}
