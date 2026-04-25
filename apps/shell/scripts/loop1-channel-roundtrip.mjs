#!/usr/bin/env node
/**
 * Loop 1 — channel round-trip end-to-end.
 *
 * The acceptance gate for "chat actually works in Ship." Spawns a fresh
 * Claude agent through the daemon, opens the actors dashboard, types a
 * known prompt into the live card's inline send form, and asserts the
 * reply renders inline within 30 seconds.
 *
 *   pass: studio.reply event lands within timeout AND its text contains
 *         the marker we asked Claude to echo back. Reply visible in the
 *         dashboard's send-status footer.
 *   fail: timeout, or wrong text, or 4xx/5xx anywhere on the path.
 *
 * No layer is "done" until this test passes. Everything else — substrate,
 * projections, identity, supervisor unification — exists to make this
 * green and keep it green.
 *
 * Usage:
 *   TOKEN=$(cat ~/.ship/daemon.token) node apps/shell/scripts/loop1-channel-roundtrip.mjs
 */
import { chromium } from "playwright";
import { writeFileSync } from "node:fs";

const BASE = process.env.SHIPD_URL ?? "http://localhost:9315";
const TOKEN = process.env.TOKEN;
const TIMEOUT_MS = 30_000;
const MARKER = `LOOP1-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const PROMPT = `Reply with exactly this token, nothing else: ${MARKER}`;

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
  try {
    data = JSON.parse(text);
  } catch {}
  return { status: r.status, data };
}

function fail(reason, details = {}) {
  console.error(`FAIL: ${reason}`);
  for (const [k, v] of Object.entries(details)) {
    console.error(`  ${k}:`, typeof v === "string" ? v : JSON.stringify(v));
  }
  process.exit(1);
}

async function spawnAgent() {
  const { status, data } = await api("/api/sessions/spawn", {
    method: "POST",
    body: JSON.stringify({
      workspace_id: "feat-view-dev-studio",
      kind: "agent",
      agent_id: "shipwright",
      provider: "claude",
    }),
  });
  if (status !== 200 || !data.session_id) fail("spawn failed", { status, data });
  return data.session_id;
}

async function waitLive(actorId) {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    const { data } = await api(`/api/actors/${actorId}`);
    if (data?.data?.live) return;
    await new Promise((r) => setTimeout(r, 500));
  }
  fail("actor never reached live", { actorId });
}

async function waitMeshRegistered(actorId) {
  // The `live` flag fires from activity-based recency, but the kernel
  // doesn't have a mailbox until the MCP child actually connects and
  // calls actor_spawn — visible as `mesh.register` in the event log.
  // Sending a chat_turn before this lands routes nowhere.
  const deadline = Date.now() + 25_000;
  while (Date.now() < deadline) {
    const { data } = await api(`/api/actors/${actorId}/events?limit=50`);
    const events = data?.data?.events ?? [];
    if (events.some((e) => e.event_type === "mesh.register")) return;
    await new Promise((r) => setTimeout(r, 1000));
  }
  fail("actor never registered with the mesh", {
    actorId,
    hint: "MCP child probably failed to boot (claude stuck on dev-channels prompt?)",
  });
}

async function tailReply(actorId, deadline) {
  let lastId = "";
  // Skip the actor.created/mesh.register that exist before our turn.
  const { data: snap } = await api(`/api/actors/${actorId}/events?limit=50`);
  const seen = snap?.data?.events ?? [];
  if (seen.length) lastId = seen[seen.length - 1].id;

  while (Date.now() < deadline) {
    const url = `/api/actors/${actorId}/events?from_id=${encodeURIComponent(lastId)}&limit=20`;
    const { data } = await api(url);
    const events = data?.data?.events ?? [];
    for (const e of events) {
      lastId = e.id;
      if (e.event_type === "studio.reply") {
        let text = "";
        try {
          text = JSON.parse(e.payload_json).text ?? "";
        } catch {}
        return { ok: true, text, eventId: e.id };
      }
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  return { ok: false };
}

async function main() {
  const startedAt = Date.now();
  console.log(`marker: ${MARKER}`);

  const actorId = await spawnAgent();
  console.log(`spawned actor: ${actorId}`);
  await waitLive(actorId);
  console.log(`actor live`);

  await waitMeshRegistered(actorId);
  console.log(`actor mesh-registered (mailbox attached)`);

  const browser = await chromium.launch();
  const ctx = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    extraHTTPHeaders: { Authorization: `Bearer ${TOKEN}` },
  });
  const page = await ctx.newPage();

  await page.goto(`${BASE}/api/v/actors`, { waitUntil: "networkidle", timeout: 10_000 });

  // Find the live card matching our actorId.
  const cardSelector = `.actor-card[data-live="true"]:has(form.send[data-actor="${actorId}"])`;
  const card = page.locator(cardSelector);
  if ((await card.count()) === 0) {
    await page.screenshot({ path: "/tmp/loop1-no-card.png", fullPage: true });
    fail("live card for spawned actor not found in dashboard", { actorId });
  }

  // Type the prompt and submit.
  const input = card.locator("form.send input");
  await input.fill(PROMPT);
  await card.locator('form.send button[type="submit"]').click();

  // Wait for the inline reply OR the kernel event log to confirm.
  const deadline = startedAt + TIMEOUT_MS;
  const tail = tailReply(actorId, deadline);

  // Also watch the inline status element for the reply rendering path
  // the user actually sees in the UI.
  const status = card.locator(".send-status");
  const uiReplied = status
    .filter({ hasText: /^reply:/ })
    .waitFor({ timeout: TIMEOUT_MS })
    .then(() => true)
    .catch(() => false);

  const eventResult = await tail;
  const uiResult = await uiReplied;
  const elapsed = ((Date.now() - startedAt) / 1000).toFixed(1);

  await page.screenshot({ path: "/tmp/loop1-final.png", fullPage: true });

  // Cleanup
  try {
    await api(`/api/sessions/${actorId}`, { method: "DELETE" });
  } catch {}
  await browser.close();

  if (!eventResult.ok) {
    fail("no studio.reply event within 30s", {
      actorId,
      elapsed_s: elapsed,
      marker: MARKER,
      screenshot: "/tmp/loop1-final.png",
    });
  }
  if (!eventResult.text.includes(MARKER)) {
    fail("studio.reply text didn't contain marker", {
      actorId,
      elapsed_s: elapsed,
      marker: MARKER,
      reply_text: eventResult.text,
    });
  }
  if (!uiResult) {
    fail("event landed but UI didn't render the reply", {
      actorId,
      elapsed_s: elapsed,
      event_id: eventResult.eventId,
    });
  }

  const summary = {
    pass: true,
    actorId,
    marker: MARKER,
    elapsed_s: Number(elapsed),
    reply_text: eventResult.text,
    event_id: eventResult.eventId,
  };
  writeFileSync("/tmp/loop1-result.json", JSON.stringify(summary, null, 2));
  console.log(`PASS in ${elapsed}s`);
  console.log(`  reply: ${eventResult.text.slice(0, 200)}`);
}

main().catch((e) => {
  console.error("ERROR:", e.message);
  console.error(e.stack);
  process.exit(2);
});
