#!/usr/bin/env node
/**
 * Loop 4 — daemon mints actor IDs; clients never synthesize.
 *
 * Today: MCP child resolves actor_id via env or synthesizes
 * `agent.{name}.{branch}` when env is unset. Synthesis produces ids
 * with slashes (broken URL routing), inconsistent shape (ULID vs
 * dotted vs slugged), and ambiguity about who's authoritative.
 *
 * Goal: daemon is the ONLY id minter. POST /api/actor/spawn with no
 * actor_id mints a fresh ULID, returns it. The caller adopts that id
 * for everything else (env, subsequent calls, identity in logs).
 *
 * Acceptance: a register-me request without actor_id returns a ULID-
 * shaped id, AND the resulting actor appears on the mesh under that
 * exact id. No `agent.X.Y` synthesis.
 *
 * Usage:
 *   TOKEN=$(cat ~/.ship/daemon.token) node apps/shell/scripts/loop4-daemon-minted-ids.mjs
 */

const BASE = process.env.SHIPD_URL ?? "http://localhost:9315";
const TOKEN = process.env.TOKEN;
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

const ULID_RE = /^[0-9A-HJKMNP-TV-Z]{26}$/;

function fail(reason, details = {}) {
  console.error(`FAIL: ${reason}`);
  for (const [k, v] of Object.entries(details)) {
    console.error(`  ${k}:`, typeof v === "string" ? v : JSON.stringify(v));
  }
  process.exit(1);
}

async function main() {
  // Send actor_spawn with NO actor_id. Daemon should mint one.
  const config = {
    namespace: "agent.test-loop4",
    write_namespaces: [""],
    read_namespaces: ["agent."],
    subscribe_namespaces: ["studio.chat_turn"],
  };
  const { status, data } = await api("/api/v1/actor/spawn", {
    method: "POST",
    body: JSON.stringify({
      config,
      capabilities: ["loop4-test"],
      // Deliberately omit actor_id so the daemon mints one.
    }),
  });

  if (status !== 200) {
    fail("actor_spawn returned non-200", { status, data });
  }

  // The daemon should return the minted id. Look for it in the response.
  const mintedId = data?.data?.actor_id ?? data?.data?.id ??
    (typeof data?.data === "string" ? data.data.replace(/^spawned: /, "") : null);

  if (!mintedId) {
    fail("daemon didn't return minted actor_id", { response: data });
  }

  if (!ULID_RE.test(mintedId)) {
    fail("daemon-minted id is not a clean ULID", {
      mintedId,
      hint: "still going through the synthesis fallback (`agent.X.Y`) instead of minting fresh",
    });
  }
  console.log(`minted: ${mintedId}`);

  // Verify the actor appears on the mesh under that id.
  await new Promise((r) => setTimeout(r, 500));
  const mesh = await api("/api/mesh");
  const actors = mesh?.data?.data?.actors ?? [];
  const found = actors.find((a) => a.actor_id === mintedId);
  if (!found) {
    fail("minted actor not on mesh", {
      mintedId,
      mesh_actors: actors.map((a) => a.actor_id),
    });
  }

  // Cleanup — try to deregister.
  // (No explicit endpoint; the kernel cleans up when the connection
  //  drops, but we registered via HTTP so the actor lives until the
  //  daemon restarts. Acceptable for the test.)
  console.log(`PASS — daemon minted clean ULID ${mintedId} and actor is on mesh`);
}

main().catch((e) => {
  console.error("ERROR:", e.message);
  process.exit(2);
});
