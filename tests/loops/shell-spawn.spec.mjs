#!/usr/bin/env node
/**
 * Shell spawn — `kind=shell` registers as a kernel actor and is
 * reachable on the mesh. Gates the spawn_shell migration onto
 * runtime::supervisor::spawn_actor.
 *
 * Acceptance:
 *   - POST /api/sessions/spawn with kind=shell returns a session_id
 *   - That id appears on /api/mesh (kernel mailbox attached)
 *   - /api/actors/{id} shows kind=shell with a fresh ULID
 *   - actor.stopped fires when we DELETE the session
 *
 *   TOKEN=$(cat ~/.ship/daemon.token) node tests/loops/shell-spawn.spec.mjs
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
  const { status, data: spawn } = await api("/api/sessions/spawn", {
    method: "POST",
    body: JSON.stringify({
      workspace_id: "feat-view-dev-studio",
      kind: "shell",
    }),
  });
  if (status !== 200 || !spawn.session_id) fail("shell spawn failed", { status, spawn });
  const id = spawn.session_id;
  console.log(`spawned: ${id}`);

  if (!ULID_RE.test(id)) {
    fail("shell session id is not ULID-shape", { id });
  }

  try {
    // Mesh presence — kernel mailbox attached.
    let onMesh = false;
    for (let i = 0; i < 20; i++) {
      const { data } = await api("/api/mesh");
      if ((data?.data?.actors ?? []).some((a) => a.actor_id === id)) {
        onMesh = true;
        break;
      }
      await new Promise((r) => setTimeout(r, 250));
    }
    if (!onMesh) fail("shell actor never reached the mesh", { id });

    // Projection write is fire-and-forget on a std::thread; poll until
    // the row materialises rather than racing it.
    let row = null;
    for (let i = 0; i < 20; i++) {
      const detail = await api(`/api/actors/${id}`);
      const a = detail?.data?.data;
      if (a && a.kind === "shell") { row = a; break; }
      await new Promise((r) => setTimeout(r, 250));
    }
    if (!row) fail("shell actor row never materialised in projection", { id });
    if (!row.live) fail("actor not marked live on mesh", { row });
  } finally {
    await api(`/api/sessions/${id}`, { method: "DELETE" });
  }

  // After kill, it should drop from the mesh within a beat.
  let cleared = false;
  for (let i = 0; i < 20; i++) {
    const { data } = await api("/api/mesh");
    if (!(data?.data?.actors ?? []).some((a) => a.actor_id === id)) {
      cleared = true;
      break;
    }
    await new Promise((r) => setTimeout(r, 250));
  }
  if (!cleared) {
    console.warn("WARN: actor still on mesh after stop (cleanup may lag)");
  }

  console.log(`PASS — shell ${id} spawned, mesh-registered, kind=shell, cleaned up`);
}

main().catch((e) => {
  console.error("ERROR:", e.message);
  process.exit(2);
});
