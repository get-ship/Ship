// Agent push-to-agent slice — load-bearing for annotations + cmdb-modal
// chat + canvas → agent flows.
//
// Verifies the end-to-end "message from outside the chat reaches the
// running agent's context" path:
//
//   1. Dispatch a harness via the daemon
//   2. POST /api/v1/sessions/{slug}/message with text + annotations
//   3. Observe `studio.chat_turn` kernel event with the rendered
//      <context> XML payload (harness-agnostic prompt wrapping)
//   4. Per-harness check on the delivery branch:
//      - PushMechanism::Channel (Claude) — event emit IS the delivery
//      - PushMechanism::Stdin (codex/cursor/gemini/opencode) — daemon
//        writes prompt to PTY master; we assert the studio.chat_turn
//        event fires (PTY write itself isn't observable without
//        attaching to the WS)
//      - PushMechanism::Respawn — legacy path; documented gap
//
// Lives at tests/agents/push.test.ts (NOT under harness-compat/) because
// it cross-cuts the harness matrix and exercises a non-harness surface
// (the session message endpoint + annotation render pipeline). The
// per-harness assertions are tier-graded.
//
// TDD posture: known gaps from arc intent that this slice surfaces:
//   - Stdin push mechanism has no real executor wiring for non-Claude
//     harnesses (apps/mcp/src/push/mod.rs explicitly admits TBD).
//     studio.chat_turn fires correctly, but downstream delivery to a
//     non-Claude PTY is best-effort today.
//   - Respawn mechanism is unimplemented at this layer.

import { describe, test, expect } from "bun:test";
import {
  binaryOnPath,
  cancelSession,
  dispatch,
  latestEventId,
  pollForEvents,
  sendSessionMessage,
  waitForActorRow,
} from "./_helpers";
import {
  COMPAT_AGENT,
  COMPAT_WORKSPACE,
  HARNESSES,
  skipped,
} from "./harness-compat/_registry";

const SKIPPED = skipped();
const STUDIO_CHAT_TURN = "studio.chat_turn";

const SAMPLE_TEXT = "ship-test: please ignore this turn";
const SAMPLE_ANNOTATIONS = [
  {
    kind: "comment",
    content: "test annotation body",
    source: { kind: "test", ref: "tests/agents/push.test.ts" },
  },
  {
    kind: "file",
    content: "fn main() {\n    println!(\"hello\");\n}",
    source: { kind: "file", ref: "src/main.rs:1-3" },
  },
];

for (const h of HARNESSES) {
  describe(`push / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    test(
      "session message with annotations emits studio.chat_turn with rendered <context>",
      async () => {
        if (skipReason) {
          console.log(`SKIPPED: ${h.id} — ${skipReason}`);
          return;
        }

        // Default axes (PTY, no attach, no keepalive). The session is
        // short-lived but actor_sessions registers it, which is all the
        // message endpoint needs to look it up.
        const dispatched = await dispatch({
          agent: COMPAT_AGENT,
          harness: h.id,
          workspace: COMPAT_WORKSPACE,
        });

        try {
          // Wait for the actor row before sending — the message endpoint
          // looks the session up in actor_sessions or pty_sessions and
          // 404s when neither has it.
          const actor = await waitForActorRow(dispatched.slug);
          if (!actor) {
            tierFailure(h.tier, h.id, "actor row never materialised", {
              slug: dispatched.slug,
            });
            return;
          }

          // Cursor BEFORE send so the poll only sees events caused by
          // this turn. studio.chat_turn is a DIRECTED event (target =
          // session id) → invisible to the SSE broadcast stream → poll
          // the persisted log instead.
          const sinceId = await latestEventId();

          const send = await sendSessionMessage(
            dispatched.slug,
            SAMPLE_TEXT,
            SAMPLE_ANNOTATIONS,
          );

          // studio.chat_turn is emitted BEFORE the per-registry lookup
          // that can 500. The HTTP status is a separate concern
          // (registry wiring); the kernel event is the load-bearing
          // piece for downstream features (annotations, etc.).
          const events = await pollForEvents(
            (e) =>
              e.event_type === STUDIO_CHAT_TURN &&
              (e.entity_id === dispatched.slug ||
                e.target_actor_id === dispatched.slug),
            {
              sinceId,
              slug: dispatched.slug,
              timeoutMs: 8_000,
              stopAfter: 1,
            },
          );

          // ── Assertion 1: studio.chat_turn fired with rendered context ──
          if (events.length === 0) {
            tierFailure(
              h.tier,
              h.id,
              `no ${STUDIO_CHAT_TURN} observed within window`,
              { slug: dispatched.slug, httpStatus: send.status },
            );
            return;
          }

          // Validate payload shape: text was rendered with <context>
          // wrapper, raw_text preserved, annotations array round-trips.
          const e = events[0];
          if (!e) {
            tierFailure(h.tier, h.id, "no event captured", {});
            return;
          }
          const payload = JSON.parse(e.payload_json) as {
            text?: string;
            raw_text?: string;
            annotations?: unknown[];
          };

          expect(payload.raw_text).toBe(SAMPLE_TEXT);
          expect(payload.annotations).toBeDefined();
          expect(payload.annotations!.length).toBe(SAMPLE_ANNOTATIONS.length);

          // Rendered text should contain the <context> wrapper and each
          // annotation's content. The render_message_with_annotations
          // function in apps/shipd/src/sessions/api/messages.rs is the
          // source of truth for this format.
          expect(payload.text).toContain("<context>");
          expect(payload.text).toContain("</context>");
          for (const a of SAMPLE_ANNOTATIONS) {
            expect(payload.text).toContain(a.content.split("\n")[0]!);
            expect(payload.text).toContain(`type="${a.kind}"`);
          }
          // Raw text appears AFTER the context block.
          expect(payload.text!.indexOf(SAMPLE_TEXT)).toBeGreaterThan(
            payload.text!.indexOf("</context>"),
          );

          // ── Assertion 2: HTTP 202 means the daemon successfully routed
          // the message to the dispatched session's registry entry.
          // Today this 500s for sessions dispatched via the orchestrator
          // because they don't register in actor_sessions OR pty_sessions
          // — see .artifacts/agents-arc/specs/job-spec-push-registry-wiring.md
          // for the drive. Asserted separately so the kernel-event signal
          // above is preserved even when this fails.
          if (send.status !== 202 && send.status !== 200) {
            tierFailure(
              h.tier,
              h.id,
              `session message HTTP rejected: status=${send.status}`,
              { body: JSON.stringify(send.body).slice(0, 200) },
            );
          }
        } finally {
          await cancelSession(dispatched.slug);
        }
      },
      30_000,
    );
  });
}

/** Tier-graded failure (T1 throws, T2/T3 warn). */
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
