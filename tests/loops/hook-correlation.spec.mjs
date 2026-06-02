#!/usr/bin/env node
/**
 * Loop 2 — hook events correlate to actor by actor_id.
 *
 * Today: hooks key by harness_session_id which is unknown for the
 * long-lived PTY claude path. Tool calls fire and silently drop —
 * dashboard never sees them, can't render tool cards.
 *
 * Acceptance: POST /api/hooks/before-tool with `actor_id` in the
 * payload → a `hook.tool_use_started` event lands in
 * /api/actors/{id}/events with target_actor_id matching. No Claude
 * required; this is a substrate-wiring loop.
 *
 * Usage:
 *   TOKEN=$(cat ~/.ship/daemon.token) node apps/shell/scripts/loop2-hook-correlation.mjs
 */
import { writeFileSync } from "node:fs";
import { guardProdHome } from "./_prod-guard.mjs";

// Refuse to mutate the operator's prod daemon (~/.ship) on direct runs.
guardProdHome();

const BASE = process.env.SHIPD_URL ?? "http://localhost:9315";
const TOKEN = process.env.TOKEN;

if (!TOKEN) {
  console.error("TOKEN env var required (cat ~/.ship/daemon.token)");
  process.exit(2);
}

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
  // Spawn a real agent so we have a valid actor_id with mesh registration.
  const spawnRes = await api("/api/v1/sessions/spawn", {
    method: "POST",
    body: JSON.stringify({
      workspace_id: "feat-view-dev-studio",
      kind: "agent",
      agent_id: "shipwright",
      harness: "claude",
    }),
  });
  if (spawnRes.status !== 200 || !spawnRes.data.session_id) {
    fail("spawn failed", spawnRes);
  }
  const actorId = spawnRes.data.session_id;
  console.log(`spawned actor: ${actorId}`);

  // Wait for the actor to be on the mesh — confirms full registration.
  const meshDeadline = Date.now() + 20_000;
  let onMesh = false;
  while (Date.now() < meshDeadline) {
    const { data } = await api("/api/v1/mesh");
    if ((data?.data?.actors ?? []).some((a) => a.actor_id === actorId)) {
      onMesh = true;
      break;
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  if (!onMesh) {
    // Cleanup before failing.
    await api(`/api/v1/sessions/${actorId}`, { method: "DELETE" });
    fail("actor never reached the mesh", { actorId });
  }
  console.log("actor on mesh");

  // Snapshot existing events so we don't false-positive on history.
  const before = await api(`/api/v1/actors/${actorId}/events?limit=100`);
  const baselineIds = new Set(
    (before.data?.data?.events ?? []).map((e) => e.id),
  );

  // Fire a synthetic hook with the actor_id we know.
  const tool = "TestTool";
  const hookRes = await api("/api/v1/hooks/before-tool", {
    method: "POST",
    body: JSON.stringify({
      actor_id: actorId,
      tool_name: tool,
      tool_input: { probe: "loop2" },
    }),
  });
  if (hookRes.status !== 200) {
    await api(`/api/v1/sessions/${actorId}`, { method: "DELETE" });
    fail("hook before-tool failed", hookRes);
  }
  console.log("hook posted");

  // Tail for the corresponding kernel event keyed on the actor.
  const deadline = Date.now() + 10_000;
  let landed = null;
  while (Date.now() < deadline) {
    const { data } = await api(`/api/v1/actors/${actorId}/events?limit=50`);
    const fresh = (data?.data?.events ?? []).filter((e) => !baselineIds.has(e.id));
    landed = fresh.find(
      (e) =>
        (e.event_type === "hook.tool_use_started"
          || e.event_type === "hook.tool_use") &&
        (e.target_actor_id === actorId || e.actor_id === actorId),
    );
    if (landed) break;
    await new Promise((r) => setTimeout(r, 500));
  }

  // Cleanup.
  await api(`/api/v1/sessions/${actorId}`, { method: "DELETE" });

  if (!landed) {
    fail("hook event never appeared in actor's event stream", {
      actorId,
      hint: "broadcast_tool_use looks up pty_sessions by harness_session_id; dispatched sessions don't register there — see job-spec-push-registry-wiring.md",
    });
  }
  let payload = {};
  try { payload = JSON.parse(landed.payload_json); } catch {}
  if (payload.tool !== tool) {
    fail("hook event landed but payload missing tool name", {
      actorId,
      event: landed,
    });
  }

  const summary = {
    pass: true,
    actorId,
    eventType: landed.event_type,
    eventId: landed.id,
    tool: payload.tool,
  };
  writeFileSync("/tmp/loop2-result.json", JSON.stringify(summary, null, 2));
  console.log(`PASS — ${landed.event_type} correlated to ${actorId}`);
  console.log(`  payload: ${JSON.stringify(payload).slice(0, 200)}`);
}

main().catch((e) => {
  console.error("ERROR:", e.message);
  process.exit(2);
});
