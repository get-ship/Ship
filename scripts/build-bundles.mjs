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
import { mkdirSync, readFileSync, writeFileSync, existsSync, statSync, renameSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { basename, dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..');
const assetsDir = join(repoRoot, 'apps/shipd/assets');
const cacheDir = join(repoRoot, 'apps/shipd/assets/.cache');

const targets = [
  { name: 'ship-elements',        entry: 'packages/ship-set/src/web-components/index.ts' },
  { name: 'ship-code-editor',     entry: 'packages/ship-set/src/web-components/islands/ship-code-editor.ts' },
  { name: 'ship-markdown-editor', entry: 'packages/ship-set/src/web-components/islands/ship-markdown-editor.ts' },
  { name: 'ship-terminal',        entry: 'packages/ship-set/src/web-components/islands/ship-terminal.ts' },
  // SUI runtime + dom + sdk + per-view setups (committed
  // assets the daemon `include_str!`s from apps/shipd/assets/).
  { name: 'sui-runtime',          entry: 'packages/ui-runtime/src/index.ts' },
  { name: 'sui-dom',              entry: 'packages/ui-dom/src/index.ts',          external: ['@ship/ui-runtime'] },
  { name: 'sui-sdk-client',       entry: 'packages/ui-sdk-client/src/index.ts',   external: ['@ship/ui-runtime'] },
  // ship-set-components: each ported primitive's .sui is
  // AOT-compiled by sui-compiler before esbuild runs. The
  // compiler emits <tag>.compiled.js next to the source; the .ts
  // file is a side-effect import of that compiled module
  // (`import './<tag>.compiled.js'`), so esbuild inlines the
  // generated `customElements.define()` call. After Arc B-1 the
  // bundle has no @ship/ui-dom dep — `external` reflects that.
  { name: 'sui-components',       entry: 'packages/ship-set-components/src/index.ts',  external: ['@ship/ui-runtime'],
    componentSources: [
      { htmlSet: 'packages/ship-set-components/src/set-status-dot/set-status-dot.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-spinner/set-spinner.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-badge/set-badge.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-chip/set-chip.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-card/set-card.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-link/set-link.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-divider/set-divider.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-skeleton/set-skeleton.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-stat/set-stat.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-mark/set-mark.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-kbd/set-kbd.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-avatar/set-avatar.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-progress/set-progress.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-button/set-button.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-tabs/set-tabs.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-tab/set-tab.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-empty-state/set-empty-state.html.set' },
      { htmlSet: 'packages/ship-set-components/src/set-input/set-input.html.set' },
    ] },
  // Each view: compile .sui via the Rust compiler first (it
  // emits view.js next to view.ts, re-imports view.ts as setup, and
  // exports the `bindings` manifest attach() needs), then esbuild
  // bundles the compiled output, inlining the controller.
  { name: 'views/agents/view',    entry: 'views/agents/view.js',  htmlSet: 'views/agents/view.sui',  external: ['@ship/ui-runtime', '@ship/ui-sdk-client'] },
  { name: 'views/counter/view',   entry: 'views/counter/view.js', htmlSet: 'views/counter/view.sui', external: ['@ship/ui-runtime', '@ship/ui-sdk-client'] },
  { name: 'views/git/view',       entry: 'views/git/view.js',     htmlSet: 'views/git/view.sui',     external: ['@ship/ui-runtime', '@ship/ui-sdk-client'] },
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
 * Read the component metadata from a `<stem>.sui.json` file next to the
 * `.html.set` source. Falls back to the directory name if the file is absent.
 */
function readComponentMeta(suiPath) {
  const inputAbs = join(repoRoot, suiPath);
  const dir = dirname(inputAbs);
  const stem = basename(inputAbs).replace(/\.html\.set$/, '').replace(/\.sui$/, '');
  const metaPath = join(dir, `${stem}.sui.json`);
  try {
    const raw = readFileSync(metaPath, 'utf8');
    const meta = JSON.parse(raw);
    return { name: meta.component ?? stem, shadow: meta.shadow ?? false };
  } catch {
    return { name: stem, shadow: false };
  }
}

/**
 * Run the Rust sui-compiler against a single `.sui` source.
 *
 * Views: emitted `<stem>.js` carries the `bindings` manifest + the
 * default-export setup that re-imports the paired controller `.ts`.
 * esbuild then bundles the compiled file, inlining the controller.
 *
 * Components: reads `<stem>.sui.json` for the tag name and shadow flag,
 * then passes `--component-name` / `--component-shadow` to the compiler.
 * The build script renames the output to `<stem>.compiled.js` so the .ts
 * side-effect import (`import './<stem>.compiled.js'`) is unambiguous.
 * When `cssPath` is supplied, the CSS is prepended as a `<style>` block.
 */
async function compileSui(suiPath, opts = {}) {
  const { mode = 'view', cssPath = null } = opts;
  const { spawnSync } = await import('node:child_process');
  const inputAbs = join(repoRoot, suiPath);
  const outDir = dirname(inputAbs);
  const stem = basename(inputAbs).replace(/\.html\.set$/, '').replace(/\.sui$/, '');
  const args = [
    'run', '--quiet', '-p', 'sui-compiler', '--',
    'build', inputAbs, '--out', outDir,
  ];
  if (mode === 'component') {
    const meta = readComponentMeta(suiPath);
    args.push('--component-name', meta.name);
    if (meta.shadow) args.push('--component-shadow');
    args.push('--no-setup');
  }
  if (cssPath) {
    args.push('--styles', join(repoRoot, cssPath));
  }
  const result = spawnSync('cargo', args, {
    cwd: repoRoot, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (result.status !== 0) {
    throw new Error(
      `sui-compiler failed for ${suiPath}:\n${result.stderr || result.stdout}`,
    );
  }
  if (mode === 'component') {
    // Rename <stem>.js → <stem>.compiled.js so the build artifact is
    // visibly distinct from the .ts source next to it.
    const emitted = join(outDir, `${stem}.js`);
    const renamed = join(outDir, `${stem}.compiled.js`);
    renameSync(emitted, renamed);
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
  // Views: compile the .sui first; the emitted view.js carries
  // the `bindings` manifest attach() reads + a default export that
  // re-imports the controller view.ts. esbuild bundles the compiled
  // file, inlining the controller.
  if (t.htmlSet) {
    await compileSui(t.htmlSet);
  }
  // Components (Arc B-1): compile each ported primitive's .html.set
  // ahead-of-time. The compiler's `customElements.define()` call
  // happens as a top-level side-effect of the emitted module; the
  // `<tag>.ts` source-side files just `import './<tag>.compiled.js'`
  // so the entire bundle is registration-by-import.
  if (t.componentSources) {
    for (const cs of t.componentSources) {
      await compileSui(cs.htmlSet, { mode: 'component', cssPath: cs.cssPath });
    }
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

  // Regression guard: views go through the Rust compiler, so the
  // bundle MUST export both `bindings` (manifest attach() reads) and
  // a `default` (the view setup). Catches the foundation-level bug
  // where view.ts was bundled directly without sui-compiler.
  if (t.htmlSet) {
    const out = readFileSync(outfile, 'utf8');
    const hasBindings = /export\s*\{[^}]*\bbindings\b/.test(out)
      || /\bbindings\s*,/.test(out);
    const hasDefault = /\bdefault\b/.test(out);
    if (!hasBindings || !hasDefault) {
      throw new Error(
        `[bundles] ${t.name}.js missing required exports — bindings:${hasBindings} default:${hasDefault}. View bundles must be compiled via sui-compiler before esbuild.`,
      );
    }
  }

  // Regression guard (Arc B-1): the components bundle is registration
  // by side-effect — `customElements.define()` calls only, no template
  // parsing. If any future change drags `hydrate` / `parse` /
  // @ship/ui-dom back in, this asserts and we don't ship the size
  // regression. Each ported tag must also appear in the bundle so a
  // missing source file can't quietly fall off the registration list.
  if (t.componentSources) {
    const out = readFileSync(outfile, 'utf8');
    if (/from\s*["']@ship\/ui-dom["']/.test(out)) {
      throw new Error(
        `[bundles] ${t.name}.js still imports from @ship/ui-dom — Arc B-1 broken`,
      );
    }
    if (/\bhydrate\s*\(/.test(out) || /\bimport\b[^;]*\bhydrate\b/.test(out)) {
      throw new Error(
        `[bundles] ${t.name}.js still references hydrate — Arc B-1 broken`,
      );
    }
    if (/\bimport\b[^;]*\bparse\b[^;]*from\s*["']@ship\/ui-dom["']/.test(out)) {
      throw new Error(
        `[bundles] ${t.name}.js still imports parse from @ship/ui-dom — Arc B-1 broken`,
      );
    }
    for (const cs of t.componentSources) {
      const tag = cs.htmlSet.split('/').pop().replace(/\.html\.set$/, '').replace(/\.sui$/, '');
      // The compiler emits `component("set-button", { … })` —
      // matching the literal tag string is the most direct way to
      // confirm every ported primitive made it into the bundle.
      if (!out.includes(`"${tag}"`)) {
        throw new Error(
          `[bundles] ${t.name}.js missing ${tag} registration — check componentSources`,
        );
      }
    }
  }

  // Bundle-size CI gate for sui-dom.js (budget: ≤ 9 KB gzip).
  // The parser was moved to @ship/ui-dom/legacy in job/dom-legacy-subpath;
  // the default surface exports attach() only. Assert zero parse-path
  // symbols to catch any regression that drags the parser back in.
  if (t.name === 'sui-dom') {
    const out = readFileSync(outfile, 'utf8');
    if (/['"@]ship\/ui-dom\/legacy/.test(out) || /from\s*["'][^"']*\/legacy/.test(out)) {
      throw new Error(
        `[bundles] sui-dom.js contains a reference to the legacy subpath — parser leaked into the default surface`,
      );
    }
    if (/\bparseSetSource\b/.test(out)) {
      throw new Error(
        `[bundles] sui-dom.js contains parseSetSource — parser leaked into the default surface`,
      );
    }
    const gzBytes = gzipSync(Buffer.from(out)).length;
    const GZ_BUDGET_KB = 9;
    if (gzBytes > GZ_BUDGET_KB * 1024) {
      throw new Error(
        `[bundles] sui-dom.js gzip size ${(gzBytes / 1024).toFixed(1)} KiB exceeds ${GZ_BUDGET_KB} KiB budget`,
      );
    }
    console.log(`[bundles]        sui-dom.js gzip ${(gzBytes / 1024).toFixed(1)} KiB (budget: ${GZ_BUDGET_KB} KiB)`);
  }

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
