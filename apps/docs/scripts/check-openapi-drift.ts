#!/usr/bin/env -S bunx tsx
/**
 * OpenAPI drift check.
 *
 * Exits non-zero when the committed `apps/docs/openapi.json` disagrees with
 * the live spec emitted by `ship openapi export --pretty`. Run as a CI gate
 * and via `just check-openapi-drift` locally.
 *
 * Failure mode: a contributor edited a `#[utoipa::path]` annotation or
 * added/removed a handler but forgot to regenerate the committed snapshot.
 * The diff tells them exactly what changed.
 */
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const REPO_ROOT = resolve(new URL(".", import.meta.url).pathname, "..", "..", "..");
const COMMITTED = resolve(REPO_ROOT, "apps/docs/openapi.json");
const AUGMENT = resolve(
  REPO_ROOT,
  "apps/docs/scripts/augment-openapi.ts",
);

function fatal(msg: string): never {
  console.error(`check-openapi-drift: ${msg}`);
  process.exit(2);
}

function runSpec(): string {
  // Step 1: emit the raw spec from shipd.
  const cargo = spawnSync(
    "cargo",
    ["run", "-q", "--bin", "ship", "--", "openapi", "export", "--pretty"],
    { cwd: REPO_ROOT, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  if (cargo.status !== 0) {
    fatal(
      `cargo run failed (exit ${cargo.status}): ${cargo.stderr.slice(0, 2000)}`,
    );
  }
  // Step 2: run the same augmentation the generate pipeline applies, so
  // drift-check compares augmented-vs-augmented. The augment script is
  // deterministic (fixed traversal order, no clock / rng) so this stays
  // exact.
  const augment = spawnSync(
    "bunx",
    ["tsx", AUGMENT],
    {
      cwd: REPO_ROOT,
      encoding: "utf8",
      input: cargo.stdout,
      maxBuffer: 32 * 1024 * 1024,
    },
  );
  if (augment.status !== 0) {
    fatal(
      `augment-openapi failed (exit ${augment.status}): ${augment.stderr.slice(0, 2000)}`,
    );
  }
  return augment.stdout;
}

function loadCommitted(): string {
  try {
    return readFileSync(COMMITTED, "utf8");
  } catch (e) {
    fatal(`could not read ${COMMITTED}: ${(e as Error).message}`);
  }
}

function canonicalize(raw: string): string {
  // Parse + re-stringify so whitespace / key-order drift doesn't fire false
  // positives. Final-newline variance is normalized too.
  const parsed = JSON.parse(raw);
  return `${JSON.stringify(parsed, null, 2)}\n`;
}

function main() {
  const live = canonicalize(runSpec());
  const committed = canonicalize(loadCommitted());

  if (live === committed) {
    console.log("OpenAPI spec matches committed snapshot.");
    process.exit(0);
  }

  console.error("OpenAPI spec drift detected.\n");
  console.error(
    `Committed: ${COMMITTED}\n` +
      "Regenerate with:\n" +
      "  cd apps/docs && bun run openapi:generate\n" +
      "then commit the updated openapi.json.\n",
  );

  // Emit a minimal per-path summary so the reviewer sees what changed without
  // a 4MB diff in the CI log.
  const liveSpec = JSON.parse(live);
  const committedSpec = JSON.parse(committed);
  const livePaths = new Set(Object.keys(liveSpec.paths ?? {}));
  const committedPaths = new Set(Object.keys(committedSpec.paths ?? {}));

  const added = [...livePaths].filter((p) => !committedPaths.has(p));
  const removed = [...committedPaths].filter((p) => !livePaths.has(p));

  if (added.length > 0) {
    console.error(`Added paths (${added.length}):`);
    for (const p of added.sort()) console.error(`  + ${p}`);
  }
  if (removed.length > 0) {
    console.error(`Removed paths (${removed.length}):`);
    for (const p of removed.sort()) console.error(`  - ${p}`);
  }
  if (added.length === 0 && removed.length === 0) {
    console.error(
      "Path set is identical — differences are inside operations, schemas, or security.",
    );
  }
  process.exit(1);
}

main();
