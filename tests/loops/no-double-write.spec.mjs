#!/usr/bin/env node
/**
 * Loop 3 — no UNIQUE constraint failures during spawn / mesh-register /
 * chat-turn / shutdown.
 *
 * The daemon log was full of:
 *   WARN runtime::db::actor_events: db::actor_events: kernel route
 *        failed: SQLite operation failed: error returned from database:
 *        (code: 1555) UNIQUE constraint failed: events.id
 *
 * Two writers (run_tx and kernel.emit) racing for the same event id.
 * Even though delivery still happened, the noise drowned real warnings
 * and made debugging painful. Acceptance: a full spawn → push → kill
 * cycle produces ZERO new UNIQUE-constraint warnings in the daemon log.
 *
 * Usage:
 *   TOKEN=$(cat ~/.ship/daemon.token) node apps/shell/scripts/loop3-no-double-write.mjs
 */
import { readFile } from "node:fs/promises";

const BASE = process.env.SHIPD_URL ?? "http://localhost:9315";
const TOKEN = process.env.TOKEN;
const LOG_PATH = process.env.SHIP_LOG ?? `${process.env.HOME}/.ship/logs/ship.log`;

if (!TOKEN) {
  console.error("TOKEN env var required");
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

async function readLogTail() {
  try {
    return await readFile(LOG_PATH, "utf8");
  } catch {
    return "";
  }
}

function fail(reason, details = {}) {
  console.error(`FAIL: ${reason}`);
  for (const [k, v] of Object.entries(details)) {
    console.error(`  ${k}:`, typeof v === "string" ? v : JSON.stringify(v));
  }
  process.exit(1);
}

async function main() {
  const startSize = (await readLogTail()).length;

  // Full lifecycle: spawn → wait for mesh → input → kill.
  const { status, data: spawn } = await api("/api/v1/sessions/spawn", {
    method: "POST",
    body: JSON.stringify({
      workspace_id: "feat-view-dev-studio",
      kind: "agent",
      agent_id: "shipwright",
      provider: "claude",
    }),
  });
  if (status !== 200 || !spawn.session_id) fail("spawn failed", { status, spawn });
  const actorId = spawn.session_id;
  console.log(`spawned: ${actorId}`);

  // Wait for mesh-register.
  const meshDeadline = Date.now() + 25_000;
  while (Date.now() < meshDeadline) {
    const { data } = await api("/api/mesh");
    if ((data?.data?.actors ?? []).some((a) => a.actor_id === actorId)) break;
    await new Promise((r) => setTimeout(r, 1000));
  }

  // Hammer: 3 hooks + a chat turn. Maximizes write paths exercised.
  for (let i = 0; i < 3; i++) {
    await api("/api/hooks/before-tool", {
      method: "POST",
      body: JSON.stringify({
        actor_id: actorId,
        tool_name: `Test${i}`,
        tool_input: { i },
      }),
    });
    await api("/api/hooks/after-tool", {
      method: "POST",
      body: JSON.stringify({
        actor_id: actorId,
        tool_name: `Test${i}`,
        tool_response: { ok: true },
      }),
    });
  }
  await api(`/api/actors/${actorId}/input`, {
    method: "POST",
    body: JSON.stringify({ text: "loop3 probe" }),
  });

  // Cleanup.
  await new Promise((r) => setTimeout(r, 1000));
  await api(`/api/v1/sessions/${actorId}`, { method: "DELETE" });

  // Sweep new log lines for UNIQUE constraint failures.
  const fullLog = await readLogTail();
  const newSlice = fullLog.slice(startSize);
  const lines = newSlice.split("\n");
  const uniqueWarnings = lines.filter((l) =>
    l.includes("UNIQUE constraint failed: events.id")
  );

  if (uniqueWarnings.length > 0) {
    fail("UNIQUE constraint warnings during lifecycle", {
      actorId,
      count: uniqueWarnings.length,
      examples: uniqueWarnings.slice(0, 3),
    });
  }

  console.log(`PASS — 0 UNIQUE-constraint warnings across spawn/mesh/hooks/turn/kill`);
  console.log(`  log delta: ${newSlice.length} bytes, ${lines.length} lines`);
}

main().catch((e) => {
  console.error("ERROR:", e.message);
  process.exit(2);
});
