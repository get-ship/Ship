#!/usr/bin/env node
/**
 * rules-crud — CRUD round-trip for the mr-02 RulesService against a
 * running daemon.
 *
 * Covers:
 *   - POST   /api/v1/rules       create with explicit scope
 *   - GET    /api/v1/rules       list (scope-annotated)
 *   - GET    /api/v1/rules/{id}  body content
 *   - PUT    /api/v1/rules/{id}  full replace
 *   - DELETE /api/v1/rules/{id}  trash
 *
 * Also asserts the trash semantics: a deleted rule is gone from the
 * canonical listing but the `delete` operation is reversible by a
 * follow-up create with the same id.
 *
 * Usage:
 *   TOKEN=$(cat ~/.ship/daemon.token) node tests/loops/rules-crud.spec.mjs
 */
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

function assert(cond, msg) {
  if (!cond) {
    console.error(`✗ ${msg}`);
    process.exit(1);
  }
  console.log(`✓ ${msg}`);
}

async function main() {
  const stamp = Date.now();
  const id = `loop-rule-${stamp}`;
  const body = `# Loop test rule\n\nWritten by tests/loops/rules-crud.spec.mjs at ${stamp}.\n`;

  // 1. CREATE (project scope)
  const created = await api("/api/v1/rules", {
    method: "POST",
    body: JSON.stringify({ id, content: body, scope: "project" }),
  });
  assert(
    created.status === 201,
    `POST /api/v1/rules → 201 (got ${created.status}): ${JSON.stringify(created.data)}`,
  );
  assert(created.data.id === id, "create returns matching id");
  assert(created.data.scope === "project", "create returns project scope");

  // 2. LIST — scope-annotated entries; new rule must appear
  const list = await api("/api/v1/rules");
  assert(list.status === 200, "GET /api/v1/rules → 200");
  const items = list.data.items ?? [];
  const found = items.find((r) => r.id === id);
  assert(found != null, `list contains ${id}`);
  assert(found?.scope === "project", "listed scope matches");

  // 3. GET body
  const got = await api(`/api/v1/rules/${id}`);
  assert(got.status === 200, "GET /api/v1/rules/{id} → 200");
  assert(got.data.content === body, "body round-trips byte-exact");

  // 4. PUT — replace body
  const newBody = `${body}\n\n## Edit ${stamp}\n`;
  const replaced = await api(`/api/v1/rules/${id}`, {
    method: "PUT",
    body: JSON.stringify({ content: newBody }),
  });
  assert(replaced.status === 200, "PUT /api/v1/rules/{id} → 200");
  const reread = await api(`/api/v1/rules/${id}`);
  assert(reread.data.content === newBody, "body reflects PUT replacement");

  // 5. DELETE — trash
  const deleted = await api(`/api/v1/rules/${id}`, { method: "DELETE" });
  assert(deleted.status === 200, "DELETE /api/v1/rules/{id} → 200");
  const after = await api(`/api/v1/rules/${id}`);
  assert(after.status === 404, `GET after delete → 404 (got ${after.status})`);

  // 6. RECREATE — id should be reusable after trash
  const recreated = await api("/api/v1/rules", {
    method: "POST",
    body: JSON.stringify({ id, content: "recreated", scope: "project" }),
  });
  assert(
    recreated.status === 201,
    `recreate after trash → 201 (got ${recreated.status})`,
  );
  // Clean up
  await api(`/api/v1/rules/${id}`, { method: "DELETE" });

  console.log("\nrules-crud: PASS");
}

main().catch((e) => {
  console.error("rules-crud: FAIL", e);
  process.exit(1);
});
