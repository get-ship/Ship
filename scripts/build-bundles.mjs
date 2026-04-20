#!/usr/bin/env node
/**
 * Build JS asset bundles consumed by the daemon (apps/shipd/assets/*.js).
 * Source of truth is packages/ui/src/web-components/. Output is NOT tracked
 * in git; run this before `cargo build -p shipd`, `cargo test`, or CI.
 *
 * Usage:
 *   node scripts/build-bundles.mjs            build all bundles (skips if unchanged)
 *   node scripts/build-bundles.mjs --force    rebuild everything
 *   node scripts/build-bundles.mjs --watch    rebuild on source changes
 */

import { build, context } from 'esbuild';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, existsSync, statSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..');
const assetsDir = join(repoRoot, 'apps/shipd/assets');
const cacheDir = join(repoRoot, 'apps/shipd/assets/.cache');

const targets = [
  { name: 'ship-elements',        entry: 'packages/ui/src/web-components/index.ts' },
  { name: 'ship-code-editor',     entry: 'packages/ui/src/web-components/ship-code-editor.ts' },
  { name: 'ship-markdown-editor', entry: 'packages/ui/src/web-components/ship-markdown-editor.ts' },
  { name: 'ship-terminal',        entry: 'packages/ui/src/web-components/ship-terminal.ts' },
];

const args = new Set(process.argv.slice(2));
const force = args.has('--force');
const watch = args.has('--watch');

const baseOpts = {
  bundle: true,
  format: 'esm',
  target: 'es2020',
  logLevel: 'warning',
  absWorkingDir: repoRoot,
};

function hashTree(entryAbs) {
  // Ask esbuild for the metafile to enumerate inputs, then hash each input's
  // bytes. Cheaper than a full rebuild: we call esbuild once per target with
  // write: false, but only when the cached hash is missing or stale.
  return null; // placeholder — computed after build via metafile
}

function fileMtimeHash(paths) {
  const h = createHash('sha256');
  for (const p of paths) {
    try {
      const st = statSync(p);
      h.update(p);
      h.update(String(st.mtimeMs));
      h.update(String(st.size));
    } catch {
      h.update(p);
      h.update('missing');
    }
  }
  return h.digest('hex');
}

async function buildOne(t) {
  const entryAbs = join(repoRoot, t.entry);
  const outfile = join(assetsDir, `${t.name}.js`);
  const cacheFile = join(cacheDir, `${t.name}.inputs.json`);

  if (!force && existsSync(outfile) && existsSync(cacheFile)) {
    try {
      const cached = JSON.parse(readFileSync(cacheFile, 'utf8'));
      const currentHash = fileMtimeHash(cached.inputs.map((i) => join(repoRoot, i)));
      if (currentHash === cached.hash) {
        console.log(`[bundles] skip   ${t.name}.js (unchanged)`);
        return;
      }
    } catch {
      // fall through to rebuild
    }
  }

  const result = await build({
    ...baseOpts,
    entryPoints: [entryAbs],
    outfile,
    metafile: true,
  });

  const inputs = Object.keys(result.metafile.inputs).map((p) =>
    relative(repoRoot, resolve(repoRoot, p)),
  );
  const hash = fileMtimeHash(inputs.map((i) => join(repoRoot, i)));
  mkdirSync(cacheDir, { recursive: true });
  writeFileSync(cacheFile, JSON.stringify({ hash, inputs }, null, 2));

  const bytes = statSync(outfile).size;
  console.log(`[bundles] build  ${t.name}.js  (${(bytes / 1024).toFixed(1)} KiB, ${inputs.length} inputs)`);
}

async function watchAll() {
  const ctxs = await Promise.all(
    targets.map((t) =>
      context({
        ...baseOpts,
        entryPoints: [join(repoRoot, t.entry)],
        outfile: join(assetsDir, `${t.name}.js`),
      }),
    ),
  );
  await Promise.all(ctxs.map((c) => c.watch()));
  console.log('[bundles] watching for changes…');
}

async function main() {
  mkdirSync(assetsDir, { recursive: true });
  if (watch) {
    await watchAll();
    return;
  }
  for (const t of targets) {
    await buildOne(t);
  }
}

main().catch((err) => {
  console.error('[bundles] failed:', err);
  process.exit(1);
});
