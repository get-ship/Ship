#!/usr/bin/env -S bunx tsx
/**
 * OpenAPI generate pipeline.
 *
 * Sequential `spawnSync` wrapper around:
 *
 *   cargo run -q --bin ship -- openapi export --pretty
 *      │
 *      ▼
 *   augment-openapi.ts  (x-codeSamples injection)
 *      │
 *      ▼
 *   apps/docs/openapi.json  (COMMITTED)
 *
 * Replaces the previous shell-pipe form. On a cold build `cargo` must compile
 * the `ship` binary before emitting any bytes; in a pipe, tsx would read EOF
 * from stdin and crash with `SyntaxError: Unexpected end of JSON input`.
 * Sequential spawns side-step that race and match the structure of
 * `check-openapi-drift.ts`.
 */
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const REPO_ROOT = resolve(new URL(".", import.meta.url).pathname, "..", "..", "..");
const OUT = resolve(REPO_ROOT, "apps/docs/openapi.json");
const AUGMENT = resolve(REPO_ROOT, "apps/docs/scripts/augment-openapi.ts");

function fatal(msg: string): never {
  console.error(`generate-openapi: ${msg}`);
  process.exit(1);
}

function main() {
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

  const augment = spawnSync(
    "bunx",
    ["tsx", AUGMENT, "--out", OUT],
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

  console.log(`Wrote ${OUT}`);
}

main();
