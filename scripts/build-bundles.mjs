#!/usr/bin/env node
/**
 * Build JS asset bundles consumed by the daemon (apps/shipd/assets/*.js).
 * Source of truth is packages/ship-set/src/web-components/. Output is NOT
 * tracked in git; run this before `cargo build -p shipd`, `cargo test`, or CI.
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
  { name: 'ship-elements',        entry: 'packages/ship-set/src/web-components/index.ts' },
  { name: 'ship-code-editor',     entry: 'packages/ship-set/src/web-components/islands/ship-code-editor.ts' },
  { name: 'ship-markdown-editor', entry: 'packages/ship-set/src/web-components/islands/ship-markdown-editor.ts' },
  { name: 'ship-terminal',        entry: 'packages/ship-set/src/web-components/islands/ship-terminal.ts' },
  // Phase 3 ship-set runtime + dom + sdk + per-view setups (committed
  // assets the daemon `include_str!`s from apps/shipd/assets/).
  { name: 'ship-set-runtime',     entry: 'packages/ship-set-runtime/src/index.ts' },
  { name: 'ship-set-dom',         entry: 'packages/ship-set-dom/src/index.ts',         external: ['@ship-set/runtime'] },
  { name: 'ship-set-sdk-client',  entry: 'packages/ship-set-sdk-client/src/index.ts',  external: ['@ship-set/runtime'] },
  { name: 'ship-set-components',  entry: 'packages/ship-set-components/src/index.ts',  external: ['@ship-set/runtime', '@ship-set/dom'], loaders: { '.set': 'text' } },
  // Each view: compile .html.set via the Rust compiler first (it
  // emits view.js next to view.ts, re-imports view.ts as setup, and
  // exports the `bindings` manifest attach() needs), then esbuild
  // bundles the compiled output, inlining the controller.
  { name: 'views/agents/view',    entry: 'views/agents/view.js',  htmlSet: 'views/agents/view.html.set',  external: ['@ship-set/runtime', '@ship-set/sdk-client'] },
  { name: 'views/counter/view',   entry: 'views/counter/view.js', htmlSet: 'views/counter/view.html.set', external: ['@ship-set/runtime', '@ship-set/sdk-client'] },
  { name: 'views/git/view',       entry: 'views/git/view.js',     htmlSet: 'views/git/view.html.set',     external: ['@ship-set/runtime', '@ship-set/sdk-client'] },
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
  loader: { '.svg': 'text' },
};

function hashTree(entryAbs) {
  // Ask esbuild for the metafile to enumerate inputs, then hash each input's
  // bytes. Cheaper than a full rebuild: we call esbuild once per target with
  // write: false, but only when the cached hash is missing or stale.
  return null; // placeholder — computed after build via metafile
}

/**
 * Run the Rust ship-set-compiler against a single `.html.set` source.
 * Output goes to a sibling `.compiled/` directory (gitignored) which
 * is then esbuild's entry. The compiler emits `view.js` (with the
 * `bindings` manifest + setup function) plus a `view.schema.json`.
 */
async function compileHtmlSet(htmlSetPath) {
  const { spawnSync } = await import('node:child_process');
  const inputAbs = join(repoRoot, htmlSetPath);
  // Emit alongside the source — view.js sits next to view.html.set
  // and view.ts. The compiled view.js does `import setup from "./view.ts"`,
  // so esbuild can resolve the controller from the same directory.
  const outDir = dirname(inputAbs);
  const result = spawnSync(
    'cargo',
    [
      'run',
      '--quiet',
      '-p',
      'ship-set-compiler',
      '--',
      'build',
      inputAbs,
      '--out',
      outDir,
    ],
    { cwd: repoRoot, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
  );
  if (result.status !== 0) {
    throw new Error(
      `ship-set-compiler failed for ${htmlSetPath}:\n${result.stderr || result.stdout}`,
    );
  }
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
  // For view targets: compile the .html.set first via the Rust
  // compiler. The emitted .compiled/view.js carries the `bindings`
  // manifest attach() reads + a default export that re-imports the
  // controller view.ts. esbuild then bundles the compiled file,
  // inlining the controller.
  if (t.htmlSet) {
    await compileHtmlSet(t.htmlSet);
  }
  const entryAbs = join(repoRoot, t.entry);
  const outfile = join(assetsDir, `${t.name}.js`);
  const cacheFile = join(cacheDir, `${t.name.replace(/[\\/]/g, '__')}.inputs.json`);
  mkdirSync(dirname(outfile), { recursive: true });

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
    ...(t.external ? { external: t.external } : {}),
    ...(t.loaders ? { loader: { ...baseOpts.loader, ...t.loaders } } : {}),
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
