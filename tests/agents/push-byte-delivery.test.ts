// Push byte-delivery slice — DEFERRED.
//
// The original intent: prove that pushing a message to a dispatched
// keepalive claude session causes claude to act (assert
// hook.tool_use_started fires within window).
//
// Why it's deferred: the assertion depends on three independent
// guarantees, each its own thread:
//
//   1. Keepalive claude actually stays alive long enough to receive the
//      push. Live observation: a fresh keepalive dispatch goes
//      `status=created, pid=N, live=false` within seconds — claude
//      exited or never settled into the TUI loop. Separate diagnosis
//      needed before any push-delivery proof can land.
//
//   2. Channel push (claude's MCP `notifications/claude/channel`)
//      actually delivers studio.chat_turn to claude's MCP subscriber.
//      The daemon emits the directed event; whether claude's MCP
//      client surfaces it as a turn is implementation-side, opaque to
//      our test surface.
//
//   3. Claude decides to call a tool in response. Even with explicit
//      "use Bash to echo X", claude's response is non-deterministic;
//      a textual answer is a valid completion.
//
// Documented findings from this attempt:
//
//   - Actor projection race: waitForActorRow returns the row before
//     actor.created has applied, so `harness` is "" on the early read.
//     A `actor_status` poll that waits for `harness != ""` would close
//     the race; the current row mapping is fine.
//
//   - Empty-string vs null in actor row JSON: the API emits `harness: ""`
//     instead of `null` when unset. Type contract on the client says
//     `string | null`; the daemon's serialization writes empty string.
//     Worth aligning post-launch.
//
// What we DO verify in this domain today (push.test.ts):
//   - HTTP 202 on /api/v1/sessions/{id}/message
//   - studio.chat_turn kernel event with rendered <context> + annotations
//   - Per-mechanism routing (Channel / Stdin / Respawn)
//
// What follow-up threads need to land before this slice becomes useful:
//   - Diagnose why dispatched keepalive claude exits early
//     (likely related to the SessionStart-hook investigation in
//     .artifacts/agents-arc/discussions/claude-session-start-hook-not-firing.md
//     — same dispatch path, same opacity)
//   - PTY output capture helper so Stdin push can be byte-verified
//     against the harness's output stream

import { describe, test } from "bun:test";

describe("push byte-delivery / claude (Channel mechanism)", () => {
  test("DEFERRED — see file header", () => {
    // Intentional no-op. The proof requires keepalive-stability +
    // MCP-delivery-observability + tool-use-determinism, none of which
    // are ready. Filing a stub keeps the file in tree as a TODO marker.
  });
});
