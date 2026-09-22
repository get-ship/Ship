// Shared prod-home guard for bun/node e2e suites (tests/agents + tests/e2e).
//
// THE LEAK this closes: every e2e spec here talks to a LIVE daemon over HTTP,
// and a daemon's home is whatever it was launched with. The dev daemon
// (`just dev`) is launched against the operator's prod `~/.ship`. These specs
// MUTATE daemon kernel state — they POST hook events, start sessions, register
// collections, drive pipelines. Pointed at prod they pollute day-to-day state.
//
// run.mjs already guarded its own path (#185), but running a spec directly
//   bun test tests/loops/collections-roundtrip.spec.mjs
//   node tests/loops/collections-roundtrip.spec.mjs
// bypasses run.mjs entirely, and tests/agents/_helpers.ts had no guard at all.
// This module is the single chokepoint both paths import so the refusal fires
// on DIRECT spec runs too, mirroring the Rust fail-loud guard in
// crates/core/ship-substrate/src/project/global.rs (which PANICS if a test
// resolves the production path).
//
// Opt out only with SHIP_LOOPS_ALLOW_PROD=1 (you really mean prod).

import { homedir } from "node:os";
import { join } from "node:path";

const ALLOW_PROD_ENV = "SHIP_LOOPS_ALLOW_PROD";

/** The operator's production Ship home — the one we refuse to touch. */
export function prodHome() {
  return join(homedir(), ".ship");
}

/**
 * Resolve the Ship home a daemon would be using, mirroring run.mjs's #185
 * resolution and _helpers.ts::daemonDir: `SHIP_HOME` if set, else `~/.ship`.
 * One source of truth so every caller agrees on what "prod" means.
 */
export function resolveHome() {
  // Treat unset OR empty/whitespace SHIP_HOME as prod (fail loud). `??` alone
  // would let SHIP_HOME="" resolve to "" — a relative/cwd path that silently
  // bypasses the guard. An empty home is a misconfig, not an opt-out.
  const h = process.env.SHIP_HOME;
  return h && h.trim() ? h : prodHome();
}

/** True when the explicit opt-out is set. */
export function prodAllowed() {
  return process.env[ALLOW_PROD_ENV] === "1";
}

/** True when the resolved home is the prod home and prod is not opted into. */
export function wouldHitProd(home = resolveHome()) {
  return home === prodHome() && !prodAllowed();
}

/** The shared refusal message, parameterised by the resolved home. */
export function prodRefusalMessage(home = resolveHome()) {
  return (
    `\n  ✖ e2e tests would mutate your prod daemon db at ${home}.\n` +
    `    Either:\n` +
    `      SHIP_HOME=<tmpdir> <run the test>\n` +
    `    or, if you really mean to run against prod:\n` +
    `      ${ALLOW_PROD_ENV}=1 <run the test>\n`
  );
}
// shipd writes a JSON file appender at `<home>/machine/logs/shipd.log`
// (apps/shipd/src/main.rs:32-49). ship-cli (the consumer of SHIP_LOG)
// writes plain-text INFO+ to `<home>/machine/logs/ship.log`
// (apps/ship-cli/src/logging.rs:5-7). Resolve against the active home,
// NOT `$HOME/.ship` — when SHIP_HOME points at a tempdir, the daemon's
// log file lives there too. `name` selects which file; default
// "shipd.log" is the one the runtime actually writes.
export function logPath(name = "shipd.log", home = resolveHome()) {
  return join(home, "machine", "logs", name);
}

/**
 * Throwing guard — for in-process callers (module load in _helpers.ts, the
 * guard's own test). Throws if the resolved home is prod and not opted in.
 * Returns the resolved home otherwise.
 */
export function assertNotProdHome(home = resolveHome()) {
  if (wouldHitProd(home)) {
    throw new Error(prodRefusalMessage(home).trim());
  }
  return home;
}

/**
 * Exiting guard — for standalone spec/runner processes. Prints the refusal to
 * stderr and exits 2 (matching run.mjs's #185 behaviour) when the resolved
 * home is prod and not opted in. Otherwise returns the resolved home.
 */
export function guardProdHome(home = resolveHome()) {
  if (wouldHitProd(home)) {
    console.error(prodRefusalMessage(home));
    process.exit(2);
  }
  return home;
}
