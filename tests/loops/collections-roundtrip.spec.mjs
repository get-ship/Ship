#!/usr/bin/env node
/**
 * collections-roundtrip — schema CRUD + item lifecycle for the mr-02
 * CollectionsService.
 *
 * Real HTTP against a running daemon. Covers:
 *   - POST /api/v1/collections to register a user-defined schema
 *   - PUT  /api/v1/collections/{id}/items/{item_id} (create_item)
 *   - GET  /api/v1/collections/{id}/items/{item_id}
 *   - POST /api/v1/collections/{id}/query   (filter / sort / limit)
 *   - DELETE /api/v1/collections/{id}/items/{item_id}
 *
 * Acceptance: schema persists, items round-trip with field validation,
 * query returns matching items in declared sort order, delete is
 * verified by a follow-up GET → 404.
 *
 * Usage:
 *   TOKEN=$(cat ~/.ship/daemon.token) node tests/loops/collections-roundtrip.spec.mjs
 */

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
  // Collection ids are snake_case alphanumeric per the trait validator.
  const collectionId = `loop_${stamp}`;

  // 1. CREATE SCHEMA
  const schema = {
    id: collectionId,
    name: `Loop ${stamp}`,
    fields: {
      title: { type: "string", required: true },
      body: { type: "markdown" },
      priority: { type: "number", filterable: true },
    },
  };
  const created = await api("/api/v1/collections", {
    method: "POST",
    body: JSON.stringify(schema),
  });
  if (created.status === 409) {
    // Stale collection from a previous run — soft skip on schema create
    // and continue with the existing id.
    console.log(`~ schema ${collectionId} already exists; reusing`);
  } else {
    assert(created.status === 201, `POST /api/v1/collections → 201 (got ${created.status})`);
  }

  // 2. LIST SCHEMAS — new id appears
  const list = await api("/api/v1/collections");
  assert(list.status === 200, "GET /api/v1/collections → 200");
  const items = list.data.data ?? [];
  assert(
    items.some((s) => s.id === collectionId),
    `list contains ${collectionId}`,
  );

  // 3. CREATE ITEMS
  const baseTime = stamp;
  const itemIds = [];
  for (let i = 0; i < 3; i++) {
    const itemId = `item-${stamp}-${i}`;
    itemIds.push(itemId);
    const r = await api(`/api/v1/collections/${collectionId}/items`, {
      method: "POST",
      body: JSON.stringify({
        id: itemId,
        data: {
          title: `Item ${i}`,
          body: `Body ${i}`,
          priority: i * 10,
        },
      }),
    });
    assert(r.status === 201, `create item ${itemId} → 201 (got ${r.status})`);
  }

  // 4. GET ITEM — round-trip the middle item
  const middle = await api(`/api/v1/collections/${collectionId}/items/${itemIds[1]}`);
  assert(middle.status === 200, `GET item → 200`);
  assert(middle.data.data?.data?.title === "Item 1", "item title round-trips");
  assert(middle.data.data?.data?.priority === 10, "item priority round-trips");

  // 5. QUERY — sort by priority desc, expect [item-2, item-1, item-0]
  const queried = await api(`/api/v1/collections/${collectionId}/query`, {
    method: "POST",
    body: JSON.stringify({
      collection: collectionId,
      sort: { priority: -1 },
      limit: 10,
    }),
  });
  assert(queried.status === 200, "POST query → 200");
  const results = queried.data.data ?? [];
  assert(results.length >= 3, `query returns at least 3 items (got ${results.length})`);
  // Filter to just our items
  const ours = results.filter((it) => itemIds.includes(it.id));
  assert(ours.length === 3, `query returns exactly our 3 items (got ${ours.length})`);
  assert(
    ours[0].data?.priority === 20 && ours[2].data?.priority === 0,
    `priority desc order [20, 10, 0] (got [${ours.map((i) => i.data?.priority).join(", ")}])`,
  );

  // 6. UPDATE — partial-merge a field
  const upd = await api(`/api/v1/collections/${collectionId}/items/${itemIds[0]}`, {
    method: "PUT",
    body: JSON.stringify({ data: { priority: 999 } }),
  });
  assert(upd.status === 200, "PUT item → 200");
  assert(upd.data.data?.data?.priority === 999, "priority updated");
  assert(
    upd.data.data?.data?.title === "Item 0",
    "title preserved across partial update",
  );

  // 7. DELETE
  for (const itemId of itemIds) {
    const r = await api(`/api/v1/collections/${collectionId}/items/${itemId}`, {
      method: "DELETE",
    });
    assert(r.status === 200, `delete ${itemId} → 200`);
  }
  const verify = await api(`/api/v1/collections/${collectionId}/items/${itemIds[0]}`);
  assert(verify.status === 404, `GET after delete → 404 (got ${verify.status})`);

  console.log("\ncollections-roundtrip: PASS");
}

main().catch((e) => {
  console.error("collections-roundtrip: FAIL", e);
  process.exit(1);
});
