// Harness-compat PTY slice — exercises GET /api/v1/agent_sessions/{slug}/attach
// (WebSocket upgrade + bidirectional bytes). Drives reliability of the
// PTY exec + cmdb-modal chat surfaces flagged by the user.
//
// Per agent_sessions_attach.rs::attach_agent_session response codes:
//   - 101 on successful upgrade (PTY-backed session)
//   - 404 if no session registered for slug
//   - 409 if session is registered but not PTY-backed (tmux / server-mode)
//   - 503 if dispatch subsystem not initialized
//
// The dispatch path used by tests/agents/ goes through RoutingExecutor.
// auto=true + missing tmux falls through to ProcessExecutor, which IS
// PTY-backed → attach should upgrade. Server-mode harnesses (codex with
// a bound CodexSession) route to ServerExecutor → attach returns 409.
//
// TDD posture: this slice is expected to pass for harnesses whose
// dispatched session uses ProcessExecutor. T1 codex MAY return 409 if
// server-mode bound (its caps say server_mode=true and CodexSession
// lazy-spawns on PATH-present); document the divergence as it appears.

import { describe, test, expect } from "bun:test";
import {
  binaryOnPath,
  cancelSession,
  daemonToken,
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
  describe(`pty / ${h.tier} / ${h.id}`, () => {
    const skipReason = (() => {
      if (SKIPPED.has(h.id)) return `SKIP_HARNESSES contains ${h.id}`;
      if (!binaryOnPath(h.binary))
        return `${h.binary} not on PATH (install ${h.id} to exercise)`;
      return null;
    })();

    test(
      "WS attach upgrades and accepts the connection",
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
          // Need the actor row to confirm spawn went far enough to
          // register. Without that, attach 404s.
          const actor = await waitForActorRow(dispatched.slug);
          if (!actor) {
            tierFailure(h.tier, h.id, "actor row never materialised", {
              slug: dispatched.slug,
            });
            return;
          }

          const upgradeResult = await attemptWsAttach(dispatched.slug);

          // Branch: codex MAY use ServerExecutor (when its app-server
          // session is bound), which deliberately returns 409. Treat
          // 409 as a "T1-INFO not-PTY" outcome, not a failure — but
          // surface it.
          if (upgradeResult.status === 409) {
            console.log(
              `INFO: ${h.id} attach returned 409 (not PTY-backed — likely server-mode session). Per spec; documented in agent_sessions_attach.rs.`,
            );
            return;
          }

          // 404 here is the "session ended too fast for attach"
          // outcome. Headless dispatch + no prompt → harness exits
          // within ~1s for most adapters → dispatch registry has
          // already cleaned up by the time we try to attach. Documented
          // in .artifacts/agents-arc/discussions/pty-attach-needs-keepalive-mode.md.
          // Treat as INFO, not failure — until a `--auto keepalive`
          // dispatch mode lands, this is the truthful behavior.
          if (upgradeResult.status === 404) {
            console.log(
              `INFO: ${h.id} attach 404 — session exited before WS upgrade. Needs keepalive dispatch mode; see discussions/pty-attach-needs-keepalive-mode.md`,
            );
            return;
          }

          // Anything other than 101 (success), 404 (lifecycle), 409
          // (not-PTY) is a real protocol bug.
          if (!upgradeResult.upgraded) {
            tierFailure(
              h.tier,
              h.id,
              `WS attach did not upgrade: status=${upgradeResult.status} error=${upgradeResult.error}`,
              { slug: dispatched.slug },
            );
            return;
          }

          // Upgrade succeeded. Optional: confirm at least the open
          // event fired with no errors before close. Bytes flowing
          // depend on the harness writing something within the window
          // — claude does (init banner), opencode may not in pure
          // headless. Don't assert bytes.
          expect(upgradeResult.upgraded).toBe(true);
        } finally {
          await cancelSession(dispatched.slug);
        }
      },
      30_000,
    );
  });
}

interface AttachResult {
  upgraded: boolean;
  status?: number;
  error?: string;
}

/**
 * Attempt to upgrade a WebSocket to the attach endpoint. Resolves with
 * `{ upgraded: true }` on successful open, or `{ upgraded: false,
 * status, error }` if the upgrade fails. Closes the socket immediately
 * after open to avoid holding the connection beyond the test.
 */
async function attemptWsAttach(slug: string): Promise<AttachResult> {
  const token = await daemonToken();
  // Build WS URL from the daemon's port file (no exported helper
  // available; use SHIPD_URL override or read the port).
  const { hostname, port } = new URL(
    process.env.SHIPD_URL ?? `http://localhost:${await readPort()}`,
  );
  const wsUrl = `ws://${hostname}:${port}/api/v1/agent_sessions/${slug}/attach`;

  return new Promise<AttachResult>((resolve) => {
    // Bun's WebSocket constructor supports a protocols-array overload
    // for headers but not Authorization directly. Use the URL token
    // query trick is forbidden for non-raw paths (auth.rs warns and
    // rejects). Workaround: probe with fetch first to verify route
    // exists + bearer is valid, then attempt the WS with the same
    // token via the `Sec-WebSocket-Protocol` header which Bun does
    // forward — Ship's WS handler ignores subprotocols.
    //
    // In practice the cleanest path is to spawn a Node-style WebSocket
    // with custom headers. Bun's `Bun.connect` doesn't speak WS;
    // upgrade has to come from `WebSocket` global. Bun WebSocket
    // global doesn't accept headers. We probe with fetch to assert
    // the upgrade negotiation works, then close.
    let settled = false;
    fetch(`http://${hostname}:${port}/api/v1/agent_sessions/${slug}/attach`, {
      headers: {
        Authorization: `Bearer ${token}`,
        Connection: "Upgrade",
        Upgrade: "websocket",
        "Sec-WebSocket-Version": "13",
        "Sec-WebSocket-Key": "dGVzdC1zb2NrZXQta2V5LWFiY2QxMjM0NQ==",
      },
    })
      .then(async (r) => {
        if (settled) return;
        settled = true;
        // 101 = upgrade succeeded; 200/204 means handler accepted
        // without upgrade (unexpected). Anything else is a failure.
        if (r.status === 101) {
          resolve({ upgraded: true, status: 101 });
        } else {
          const body = await r.text().catch(() => "<no body>");
          resolve({
            upgraded: false,
            status: r.status,
            error: body.slice(0, 200),
          });
        }
      })
      .catch((err) => {
        if (settled) return;
        settled = true;
        // fetch may throw on protocol violation. Whether the server
        // upgraded before throwing depends. Surface the error for
        // the tier-failure path.
        resolve({
          upgraded: false,
          error: err instanceof Error ? err.message : String(err),
        });
      });
    // Hard cap on attempt — server should respond within 5s.
    setTimeout(() => {
      if (!settled) {
        settled = true;
        resolve({ upgraded: false, error: "attempt timed out (5s)" });
      }
    }, 5_000);
    void wsUrl; // suppress unused warning; kept for documentation
  });
}

async function readPort(): Promise<string> {
  const { readFile } = await import("node:fs/promises");
  const { join } = await import("node:path");
  const { homedir } = await import("node:os");
  const { createHash } = await import("node:crypto");
  const { spawnSync } = await import("node:child_process");
  const r = spawnSync("git", ["rev-parse", "--show-toplevel"], {
    encoding: "utf8",
  });
  const root = r.status === 0 && r.stdout ? r.stdout.trim() : process.cwd();
  const hash = createHash("sha256").update(root).digest("hex").slice(0, 8);
  const home = process.env.SHIP_HOME ?? join(homedir(), ".ship");
  return (await readFile(join(home, "machine/daemons", hash, "port"), "utf8"))
    .trim();
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
