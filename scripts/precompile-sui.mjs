#!/usr/bin/env node
// Precompile every `.sui` source under a package's `src/` to its sibling
// `<stem>.compiled.js` artifact. Runs as a `pretest` hook before vitest
// boots so a buggy compile fails the test command before any test starts —
// rather than surfacing as a confusing "module not found" deep inside a
// vitest worker.
//
// The vitest plugin's lazy fallback (resolveId hook in vitest.config.ts)
// remains in place for HMR/watch loops where individual files change, but
// cold runs are covered here.
//
// Usage:
//   node scripts/precompile-sui.mjs <package-dir>
//
// `<package-dir>` may be absolute, or relative to either the current
// working directory or the repo root.

import { existsSync, mkdirSync, readdirSync, renameSync, statSync } from 'node:fs';
import { basename, dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnBoundedOrThrow } from './spawn-bounded.mjs';

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..');
const compilerBin = join(repoRoot, 'target/debug/sui-compiler');

function resolvePackageDir(arg) {
  if (!arg) {
    console.error('precompile-sui: missing <package-dir> argument');
    process.exit(2);
  }
  for (const candidate of [arg, join(process.cwd(), arg), join(repoRoot, arg)]) {
    const abs = resolve(candidate);
    if (existsSync(abs) && statSync(abs).isDirectory()) return abs;
  }
  console.error(`precompile-sui: package dir not found: ${arg}`);
  process.exit(2);
}

function* walkSui(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkSui(p);
    } else if (entry.isFile() && entry.name.endsWith('.sui')) {
      yield p;
    }
  }
}

function compileOne(suiPath) {
  const outDir = dirname(suiPath);
  const stem = basename(suiPath, '.sui');
  const hasBin = existsSync(compilerBin);
  const cmd = hasBin ? compilerBin : 'cargo';
  const args = hasBin
    ? ['build', suiPath, '--out', outDir, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--out', outDir, '--component-name', stem, '--no-dts'];

  const rel = relative(repoRoot, suiPath);
  spawnBoundedOrThrow(cmd, args, `sui-compiler ${rel}`, { cwd: repoRoot });

  const emitted = join(outDir, `${stem}.js`);
  const compiled = join(outDir, `${stem}.compiled.js`);
  if (existsSync(emitted)) renameSync(emitted, compiled);
  if (!existsSync(compiled)) {
    throw new Error(`sui-compiler ${rel}: compiled artifact missing at ${compiled}`);
  }
}

function main() {
  const pkgDir = resolvePackageDir(process.argv[2]);
  const srcDir = join(pkgDir, 'src');
  if (!existsSync(srcDir)) {
    console.log(`[precompile-sui] no src/ in ${relative(repoRoot, pkgDir)}, nothing to do`);
    return;
  }

  const sources = [...walkSui(srcDir)];
  if (sources.length === 0) {
    console.log(`[precompile-sui] no .sui files under ${relative(repoRoot, srcDir)}`);
    return;
  }

  const t0 = Date.now();
  let count = 0;
  for (const sui of sources) {
    try {
      compileOne(sui);
    } catch (err) {
      console.error(`[precompile-sui] FAIL ${relative(repoRoot, sui)}`);
      console.error(err.message);
      process.exit(1);
    }
    count += 1;
  }
  const elapsed = ((Date.now() - t0) / 1000).toFixed(1);
  console.log(`[precompile-sui] compiled ${count} .sui file(s) in ${elapsed}s`);
}

main();
