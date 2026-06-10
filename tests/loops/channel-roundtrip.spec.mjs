#!/usr/bin/env node
/**
 * Channel round-trip — headless.
 *
 * Spawns a Claude agent through the daemon, waits for mesh registration,
 * pushes a chat turn via /api/actors/{id}/input, and asserts that the
 * channel adapter actually delivered the notification (`claude channel
 * push: sent` line in ~/.ship/logs/ship.log).
 *
 * Reply derivation (parsing assistant turns out of stream-json or the
 * PTY) is not yet implemented — this test gates the WIRE: did the
 * notification reach Claude over MCP. UI rendering of replies will be
 * tested separately once <ship-agent-thread> lands.
 *
 *   TOKEN=$(cat ~/.ship/daemon.token) node tests/loops/channel-roundtrip.spec.mjs
 *   just test-loops
 */
import { readFile } from "node:fs/promises";
import { guardProdHome } from "./_prod-guard.mjs";

// Refuse to mutate the operator's prod daemon (~/.ship) on direct runs.
guardProdHome();

const BASE = process.env.SHIPD_URL ?? "http://localhost:9315";
const TOKEN = process.env.TOKEN;
const LOG_PATH = process.env.SHIP_LOG ?? `${process.env.HOME}/.ship/logs/ship.log`;
const TIMEOUT_MS = 90_000;

if (!TOKEN) { console.error("TOKEN env var required"); process.exit(2); }

const headers = {
  Authorization: `Bearer ${TOKEN}`,
  "Content-Type": "application/json",
};

async function api(path, init = {}) {
  const r = await fetch(BASE + path, {
    ...init,
    headers: { ...headers, ...(init.headers ?? {}) },
  });
  const text = await r.text();
  let data = text;
  try { data = JSON.parse(text); } catch {}
  return { status: r.status, data };
}

function fail(reason, details = {}) {
  console.error(`FAIL: ${reason}`);
  for (const [k, v] of Object.entries(details)) {
    console.error(`  ${k}:`, typeof v === "string" ? v : JSON.stringify(v));
  }
  process.exit(1);
}

async function main() {
  const startedAt = Date.now();
  const startLogSize = (await readFile(LOG_PATH, "utf8").catch(() => "")).length;

  // Spawn.
  const { status, data: spawn } = await api("/api/v1/sessions/spawn", {
    method: "POST",
    body: JSON.stringify({
      workspace_id: "feat-view-dev-studio",
      kind: "agent",
      agent_id: "shipwright",
      harness: "claude",
    }),
  });
  if (status !== 200 || !spawn.session_id) fail("spawn failed", { status, spawn });
  const actorId = spawn.session_id;
  console.log(`spawned: ${actorId}`);

  try {
    // Wait for mesh-register — confirms the kernel mailbox is live AND
    // claude has booted past the dev-channels confirmation menu.
    // The mesh roster (`GET /api/v1/mesh`) was culled with the mesh op
    // surface; `GET /api/v1/actors` is the surviving projection (rows
    // keyed by `id`, `live` overlaid from the kernel registry).
    const meshDeadline = Date.now() + 25_000;
    let onMesh = false;
    while (Date.now() < meshDeadline) {
      const { data } = await api("/api/v1/actors");
      if ((data?.data?.actors ?? []).some((a) => a.id === actorId)) {
        onMesh = true;
        break;
      }
      await new Promise((r) => setTimeout(r, 1000));
    }
    if (!onMesh) fail("actor never reached the roster", { actorId });
    console.log("on mesh");

    // Push a chat turn. The per-actor `/input` REST route was culled with
    // the actor.* op surface; `POST /api/v1/sessions/{id}/message` is the
    // surviving chat-turn push (emits studio.chat_turn).
    const inputRes = await api(`/api/v1/sessions/${actorId}/message`, {
      method: "POST",
      body: JSON.stringify({ text: "ping (channel-roundtrip test)" }),
    });
    if (inputRes.status !== 202) fail("input rejected", { actorId, inputRes });

    // The wire signal: ship.log records "claude channel push: sent" when
    // the relay adapter hands the notification to Claude over MCP.
    // That's the channel-protocol-side acceptance — Claude receiving and
    // rendering it is on the Claude binary.
    const deadline = startedAt + TIMEOUT_MS;
    let pushSent = false;
    while (Date.now() < deadline) {
      const log = await readFile(LOG_PATH, "utf8").catch(() => "");
      const slice = log.slice(startLogSize);
      if (slice.includes(`claude channel push: sent event_type=studio.chat_turn`)) {
        pushSent = true;
        break;
      }
      await new Promise((r) => setTimeout(r, 500));
    }
    if (!pushSent) {
      fail("no `claude channel push: sent` line for studio.chat_turn", {
        actorId,
        elapsed_s: ((Date.now() - startedAt) / 1000).toFixed(1),
      });
    }

    console.log(
      `PASS — channel push delivered to Claude in ${((Date.now() - startedAt) / 1000).toFixed(1)}s`,
    );
  } finally {
    await api(`/api/v1/sessions/${actorId}`, { method: "DELETE" });
  }
}

main().catch((e) => {
  console.error("ERROR:", e.message);
  process.exit(2);
});
