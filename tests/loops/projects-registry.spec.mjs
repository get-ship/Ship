#!/usr/bin/env node
/**
 * projects-registry — list / add / switch round-trip against a running
 * daemon.
 *
 * Validates the launcher-facing surface that backs the project rail
 * and quick-switch palette. Asserts:
 *   - GET  /api/v1/projects        returns the registry
 *   - POST /api/v1/projects        adds a path; idempotent on re-add
 *   - POST /api/v1/projects/switch sets active; rejects unknown ids
 *   - active id round-trips on next list
 *
 * Usage:
 *   TOKEN=$(cat <daemon-token>) node tests/loops/projects-registry.spec.mjs
 */

import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { guardProdHome } from "./_prod-guard.mjs";

// Refuse to mutate the operator's prod daemon (~/.ship) on direct runs.
guardProdHome();

const BASE = process.env.SHIPD_URL ?? "http://localhost:9315";
const TOKEN = process.env.TOKEN;

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

function assert(cond, msg) {
  if (!cond) {
    console.error(`✗ ${msg}`);
    process.exit(1);
  }
  console.log(`✓ ${msg}`);
}

async function main() {
  // 1. baseline list — capture the previous active id so we can restore
  // it on exit. `SpaceService` exposes no Remove RPC (see
  // proto/ship/ops/v1/space.proto:83-117 — List/Active/Info/Add/Activate/Watch
  // only), so the added project row persists for the lifetime of the
  // daemon home. The tempdir IS under our control and must be removed.
  const before = await api("/api/v1/projects");
  assert(before.status === 200, "GET /api/v1/projects → 200");
  const baselineCount = before.data.projects?.length ?? 0;
  const previousActive = before.data.active?.id ?? before.data.active ?? null;
  let addedProjectId = null;
  let tmp = null;

  try {
    // 2. add a fresh tempdir as a project
    tmp = await mkdtemp(join(tmpdir(), "ship-loop-proj-"));
    const added = await api("/api/v1/projects", {
      method: "POST",
      body: JSON.stringify({ path: tmp, label: "loop-project" }),
    });
    assert(
      added.status === 200,
      `POST /api/v1/projects → 200 (got ${added.status})`,
    );
    const found = added.data.projects?.find((p) => p.path === tmp || p.path === tmp.toString());
    assert(found != null, `added project surfaces in response (path ${tmp})`);
    assert(typeof found?.id === "string" && found.id.length > 0, "id is a stable string");
    const projectId = found.id;
    addedProjectId = projectId;

    // 3. re-add same path is idempotent (id stable, count doesn't double)
    const readded = await api("/api/v1/projects", {
      method: "POST",
      body: JSON.stringify({ path: tmp }),
    });
    assert(readded.status === 200, "re-add same path → 200");
    const stillFound = readded.data.projects?.find((p) => p.id === projectId);
    assert(stillFound != null, "id stable across re-register");
    const count = readded.data.projects.length;
    assert(
      count === baselineCount + 1,
      `count grew by exactly 1 (${baselineCount} → ${count})`,
    );

    // 4. switch to the project
    const switched = await api("/api/v1/projects/switch", {
      method: "POST",
      body: JSON.stringify({ id: projectId }),
    });
    assert(switched.status === 200, "POST /api/v1/projects/switch → 200");
    // Switch returns `active: { id, name, path }` (richer than list's
    // `active: <id>` string). Extract the id explicitly.
    assert(
      switched.data.active?.id === projectId,
      "switch returns the new active id",
    );

    // 5. list reflects the active id
    const after = await api("/api/v1/projects");
    assert(after.data.active === projectId, "list.active === switched id");

    // 6. switching to an unknown id returns 404
    const bad = await api("/api/v1/projects/switch", {
      method: "POST",
      body: JSON.stringify({ id: "not-a-real-project" }),
    });
    assert(bad.status === 404, `unknown id → 404 (got ${bad.status})`);

    console.log("\nprojects-registry: PASS");
  } finally {
    // Restore the previous active project so the operator's workspace
    // pointer isn't left pointing at a tempdir that we're about to rm.
    if (previousActive && previousActive !== addedProjectId) {
      await api("/api/v1/projects/switch", {
        method: "POST",
        body: JSON.stringify({ id: previousActive }),
      }).catch((e) => {
        console.warn(
          `  ⚠ could not restore previous active project '${previousActive}': ${e.message}`,
        );
      });
    }
    if (tmp) {
      try {
        await rm(tmp, { recursive: true, force: true });
      } catch (e) {
        console.warn(`  ⚠ could not remove tempdir ${tmp}: ${e.message}`);
      }
    }
    if (addedProjectId) {
      console.warn(
        `  ⚠ project '${addedProjectId}' (path ${tmp ?? "<rm'd>"}) persists in ` +
          `the registry (no Remove RPC). To clean up: re-init the home, or ` +
          `delete the row directly from the projects table.`,
      );
    }
  }
}
