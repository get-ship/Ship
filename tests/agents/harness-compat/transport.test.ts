// Harness-compat transport slice — verifies routing.rs::spawn sets
// the actor projection's `transport` field correctly per dispatch mode.
//
// Source of truth: crates/ship-supervisor/src/executors/routing.rs
//   let transport = if ctx.headless { "headless" } else { "pty" };
//
// auto:true (headless) → ProcessExecutor → transport = "headless"
// auto:false (interactive) → TmuxWindow (if tmux) → transport = "pty"
//
// The transport field drives downstream UX (attach button enabled?
// chat shape choice?), so a regression here breaks the UI silently.

import { describe, test, expect } from "bun:test";
import {
  binaryOnPath,
  cancelSession,
  dispatch,
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
  describe(`transport / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    for (const mode of [
      { label: "headless", auto: true, expected: "headless" as const },
      { label: "interactive", auto: false, expected: "pty" as const },
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
            auto: mode.auto,
            workspace: COMPAT_WORKSPACE,
          });

          try {
            // Poll until transport is populated (not just row present).
            // There's a projection race: process.spawned can land before
            // actor.created, creating a stub row with empty transport;
            // actor.created fills it in. Poll until we see the real value.
            const deadline = Date.now() + 15_000;
            let actor = null;
            while (Date.now() < deadline) {
              actor = await waitForActorRow(dispatched.slug);
              if (actor && actor.transport && actor.transport.length > 0) break;
              await new Promise((r) => setTimeout(r, 250));
            }
            if (!actor) {
              tierFailure(h.tier, h.id, "actor row never materialised", {
                slug: dispatched.slug,
              });
              return;
            }
            expect(actor.transport).toBe(mode.expected);
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
