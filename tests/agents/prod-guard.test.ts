// Proof for the shared prod-home guard (tests/_shared/prod-guard.mjs).
//
// The guard is the single chokepoint that refuses to point an e2e suite at a
// daemon whose home is the operator's prod ~/.ship — mirroring the Rust
// fail-loud panic in crates/core/ship-substrate/src/project/global.rs. It must:
//   - throw / exit when the resolved home is ~/.ship (and prod isn't opted in)
//   - pass for a tmpdir
//   - pass for ~/.ship when SHIP_LOOPS_ALLOW_PROD=1 (explicit override)
//
// In-process functions (assertNotProdHome/wouldHitProd) are exercised directly.
// The exiting variant (guardProdHome → process.exit(2)) is exercised in a
// subprocess so it can't kill this runner.

import { afterEach, describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { homedir, tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import {
  assertNotProdHome,
  logPath,
  prodHome,
  resolveHome,
  wouldHitProd,
} from "../_shared/prod-guard.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const GUARD_PATH = join(__dirname, "../_shared/prod-guard.mjs");
const PROD = join(homedir(), ".ship");
const TMP = tmpdir();

// Snapshot + restore the two env vars the guard reads.
const savedHome = process.env.SHIP_HOME;
const savedAllow = process.env.SHIP_LOOPS_ALLOW_PROD;
afterEach(() => {
  if (savedHome === undefined) delete process.env.SHIP_HOME;
  else process.env.SHIP_HOME = savedHome;
  if (savedAllow === undefined) delete process.env.SHIP_LOOPS_ALLOW_PROD;
  else process.env.SHIP_LOOPS_ALLOW_PROD = savedAllow;
});

describe("prod-home guard (in-process)", () => {
  test("prodHome() is ~/.ship", () => {
    expect(prodHome()).toBe(PROD);
  });

  test("resolveHome() defaults to ~/.ship when SHIP_HOME unset", () => {
    delete process.env.SHIP_HOME;
    expect(resolveHome()).toBe(PROD);
  });

  test("resolveHome() honours SHIP_HOME", () => {
    process.env.SHIP_HOME = TMP;
    expect(resolveHome()).toBe(TMP);
  });

  test("assertNotProdHome THROWS when home resolves to ~/.ship", () => {
    delete process.env.SHIP_HOME;
    delete process.env.SHIP_LOOPS_ALLOW_PROD;
    expect(() => assertNotProdHome()).toThrow(/prod daemon db/);
    expect(wouldHitProd()).toBe(true);
  });

  test("assertNotProdHome THROWS for explicit SHIP_HOME=~/.ship", () => {
    process.env.SHIP_HOME = PROD;
    delete process.env.SHIP_LOOPS_ALLOW_PROD;
    expect(() => assertNotProdHome()).toThrow(/prod daemon db/);
  });

  test("assertNotProdHome PASSES for a tmpdir", () => {
    process.env.SHIP_HOME = TMP;
    delete process.env.SHIP_LOOPS_ALLOW_PROD;
    expect(assertNotProdHome()).toBe(TMP);
    expect(wouldHitProd()).toBe(false);
  });

  test("SHIP_LOOPS_ALLOW_PROD=1 overrides the refusal for ~/.ship", () => {
    delete process.env.SHIP_HOME;
    process.env.SHIP_LOOPS_ALLOW_PROD = "1";
    expect(assertNotProdHome()).toBe(PROD);
    expect(wouldHitProd()).toBe(false);
  });
});

describe("logPath (shipd log resolver)", () => {
  test("resolves to <home>/machine/logs/shipd.log by default", () => {
    delete process.env.SHIP_HOME;
    // Default home resolves to ~/.ship (prod), but logPath itself
    // never refuses — it's a path resolver, not a guard. Pin SHIP_HOME
    // to make the assertion stable.
    process.env.SHIP_HOME = TMP;
    expect(logPath()).toBe(join(TMP, "machine", "logs", "shipd.log"));
  });

  test("selects a named file (e.g. ship.log)", () => {
    process.env.SHIP_HOME = TMP;
    expect(logPath("ship.log")).toBe(join(TMP, "machine", "logs", "ship.log"));
  });

  test("honours an explicit home argument", () => {
    expect(logPath("shipd.log", TMP)).toBe(join(TMP, "machine", "logs", "shipd.log"));
  });

  test("home can be ~/.ship — logPath does not refuse (it's a path)", () => {
    expect(logPath("shipd.log", PROD)).toBe(join(PROD, "machine", "logs", "shipd.log"));
  });
});

// The exiting variant must abort the process (exit 2) on prod and return
// cleanly otherwise. Run it in a subprocess so the assertion is on exit code.
function runGuard(env: Record<string, string | undefined>): number {
  const r = spawnSync(
    process.execPath,
    ["-e", `import("${GUARD_PATH}").then(m => m.guardProdHome());`],
    { env: { ...process.env, ...env }, encoding: "utf8" },
  );
  return r.status ?? -1;
}

describe("prod-home guard (exiting variant, subprocess)", () => {
  test("guardProdHome EXITS 2 when home is ~/.ship", () => {
    expect(
      runGuard({ SHIP_HOME: undefined, SHIP_LOOPS_ALLOW_PROD: undefined }),
    ).toBe(2);
  });

  test("guardProdHome EXITS 0 (returns) for a tmpdir", () => {
    expect(
      runGuard({ SHIP_HOME: TMP, SHIP_LOOPS_ALLOW_PROD: undefined }),
    ).toBe(0);
  });

  test("guardProdHome EXITS 0 for ~/.ship with the opt-out set", () => {
    expect(
      runGuard({ SHIP_HOME: undefined, SHIP_LOOPS_ALLOW_PROD: "1" }),
    ).toBe(0);
  });
});
