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
import { mkdirSync, readFileSync, writeFileSync, existsSync, statSync, readdirSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { basename, dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnBoundedOrThrow } from './spawn-bounded.mjs';

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..');
// Assets split between two destinations:
//   - apps/shipd/assets/      — non-SUI bundles (Lit ship-elements, datastar, etc.)
//   - crates/ship-ui-framework/assets/ — every SUI bundle + the view-host static
// The split mirrors the D1 module extraction (sui-framework owns SUI hosting;
// shipd owns Lit/REST/auth/static-Lit-assets).
const litAssetsDir = join(repoRoot, 'apps/shipd/assets');
const suiAssetsDir = join(repoRoot, 'crates/ship-ui-framework/assets');
const cacheDir = join(repoRoot, 'apps/shipd/assets/.cache');

function assetsDirFor(name) {
  if (
    name === 'sui-custom-elements' ||
    name.startsWith('sui-') ||
    name.startsWith('sui-islands/') ||
    name.startsWith('views/') // baked SUI views: counter/agents/git → SUI bundle
  ) {
    return suiAssetsDir;
  }
  return litAssetsDir;
}

// Discover non-island SUI components: each subdirectory of src/ that is not
// `islands/` and contains one or more `<name>.sui` files. Directories may
// host sibling components (e.g. sui-toast + sui-toast-container co-located
// in sui-toast/); all .sui files in the directory are compiled.
function discoverComponentSources() {
  const srcDir = join(repoRoot, 'packages/ui-components/src');
  const results = [];
  for (const entry of readdirSync(srcDir, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name === 'islands') continue;
    // Always include the primary file matching the directory name (if present).
    const primaryFile = join(srcDir, entry.name, `${entry.name}.sui`);
    if (existsSync(primaryFile)) {
      results.push({ htmlSet: `packages/ui-components/src/${entry.name}/${entry.name}.sui` });
    }
    // Also include any sibling .sui files in the same directory.
    const dirEntries = readdirSync(join(srcDir, entry.name), { withFileTypes: true });
    for (const f of dirEntries) {
      if (!f.isFile() || !f.name.endsWith('.sui') || f.name === `${entry.name}.sui`) continue;
      results.push({ htmlSet: `packages/ui-components/src/${entry.name}/${f.name}` });
    }
  }
  return results;
}

// Discover island bundles: each subdirectory of src/islands/ is one
// bundle. The bundle's name is the directory name; every `.sui` file
// inside the directory contributes a component to that bundle (so a
// family — e.g. command-bar / -palette / -chat — packs into one
// asset). The synthesised entry imports each compiled stem.
function discoverIslandBundles() {
  const islandsDir = join(repoRoot, 'packages/ui-components/src/islands');
  if (!existsSync(islandsDir)) return [];
  const out = [];
  for (const entry of readdirSync(islandsDir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const dir = join(islandsDir, entry.name);
    const stems = readdirSync(dir, { withFileTypes: true })
      .filter((f) => f.isFile() && f.name.endsWith('.sui'))
      .map((f) => f.name.replace(/\.sui$/, ''))
      .sort();
    if (stems.length === 0) continue;
    out.push({ bundle: entry.name, stems });
  }
  return out;
}

const componentEntries = discoverComponentSources();
const islandBundles = discoverIslandBundles();

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
  // ui-components: each non-island SUI primitive's .sui is AOT-compiled by
  // sui-compiler before esbuild runs. Island components are excluded here
  // and emitted as separate bundles below.
  { name: 'sui-components',       entry: 'packages/ui-components/src/index.ts',  external: ['@ship/ui-runtime'],
    componentSources: componentEntries },
  // Per-island bundles — one per packages/ui-components/src/islands/<bundle>/.
  // Each externalises @ship/ui-runtime and @ship/ui-dom so the shared runtime
  // loads once. Family bundles include every `.sui` stem in the bundle dir;
  // the synthesised entry imports each compiled stem so all related
  // `customElements.define()` side-effects land in one asset.
  ...islandBundles.map(({ bundle, stems }) => ({
    name: `sui-islands/${bundle}`,
    islandBundle: bundle,
    islandStems: stems,
    islandDir: `packages/ui-components/src/islands/${bundle}`,
    external: ['@ship/ui-runtime', '@ship/ui-dom'],
    componentSources: stems.map((stem) => ({
      htmlSet: `packages/ui-components/src/islands/${bundle}/${stem}.sui`,
    })),
  })),
  // Each view: compile .sui via the Rust compiler first (it
  // emits view.js next to view.ts, re-imports view.ts as setup, and
  // exports the `bindings` manifest attach() needs), then esbuild
  // bundles the compiled output, inlining the controller.
  // @ship/ui-dom is externalised because Phase 3b.3 compiled views import
  // hydrate helpers (`hydrateText`, `collectTextOpens`) from it. The
  // shared sui-dom.js bundle on the page already exposes them globally
  // via the runtime registry — bundling them again would duplicate code.
  { name: 'views/agents/view',    entry: 'views/agents/dist/page.compiled.js',  htmlSet: 'views/agents/page.sui',  external: ['@ship/ui-runtime', '@ship/ui-dom', '@ship/ui-sdk-client'] },
  { name: 'views/counter/view',   entry: 'views/counter/dist/page.compiled.js', htmlSet: 'views/counter/page.sui', external: ['@ship/ui-runtime', '@ship/ui-dom', '@ship/ui-sdk-client'] },
  { name: 'views/git/view',       entry: 'views/git/dist/page.compiled.js',     htmlSet: 'views/git/page.sui',     external: ['@ship/ui-runtime', '@ship/ui-dom', '@ship/ui-sdk-client'] },
];

const args = new Set(process.argv.slice(2));
const force = args.has('--force');
const watch = args.has('--watch');

// Aggregate per-component CEMs into a single file the daemon can serve at
// /api/assets/sui-custom-elements.json. Each subdir of ui-components/src
// (excluding `islands/`) contributes its `<tag>.custom-elements.json` if
// present. Writes when content changes; otherwise leaves the file alone
// so cargo build doesn't re-link unnecessarily.
function aggregateSuiCem() {
  const srcDir = join(repoRoot, 'packages/ui-components/src');
  const out = { schemaVersion: '1.0.0', readme: '', modules: [] };
  for (const entry of readdirSync(srcDir, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name === 'islands') continue;
    const cemPath = join(srcDir, entry.name, 'dist', `${entry.name}.custom-elements.json`);
    if (!existsSync(cemPath)) continue;
    try {
      const cem = JSON.parse(readFileSync(cemPath, 'utf8'));
      if (Array.isArray(cem.modules)) out.modules.push(...cem.modules);
    } catch {
      // ignore malformed CEMs — they show up as warnings during the next build
    }
  }
  const target = join(suiAssetsDir, 'sui-custom-elements.json');
  const next = JSON.stringify(out);
  let prev = '';
  try { prev = readFileSync(target, 'utf8'); } catch {}
  if (prev !== next) {
    mkdirSync(suiAssetsDir, { recursive: true });
    writeFileSync(target, next);
    console.log(`[bundles] write  sui-custom-elements.json  (${out.modules.length} modules)`);
  }
}
aggregateSuiCem();

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
 * Run the Rust sui-compiler against a single `.sui` source.
 *
 * The directory IS the component:
 *   - tag name = filename stem (e.g. `sui-button.sui` → `<sui-button>`)
 *   - shadow = always on
 *   - paired controller = `<stem>.ts` next to source, auto-detected by the compiler
 *
 * The compiler emits all artifacts into a sibling `dist/` directory next
 * to each source file:
 *   <source-dir>/dist/<stem>.compiled.js
 *   <source-dir>/dist/<stem>.custom-elements.json   (components)
 *   <source-dir>/dist/<stem>.schema.json            (views)
 *   <source-dir>/dist/<stem>.sui.meta.json
 * The emitted `<stem>.compiled.js` carries an `import setup from
 * "../<stem>.controller.ts"` line (paired controller, auto-detected).
 * esbuild then bundles the compiled file, inlining the controller.
 *
 * When `cssPath` is supplied, the CSS is prepended as a `<style>` block.
 */
async function compileSui(suiPath, opts = {}) {
  const { mode = 'view', cssPath = null } = opts;
  const inputAbs = join(repoRoot, suiPath);
  const stem = basename(inputAbs).replace(/\.sui$/, '');
  const args = [
    'run', '--quiet', '-p', 'sui-compiler', '--',
    'build', inputAbs,
  ];
  if (mode === 'component') {
    // Compiler infers tag from filename stem; shadow is always on for
    // components. No per-component config file. Paired `<stem>.ts`
    // controllers are auto-detected by the compiler.
    args.push('--component-name', stem);
  }
  if (cssPath) {
    args.push('--styles', join(repoRoot, cssPath));
  }
  // Wrap in `prlimit + timeout` so a runaway compile (pathological input,
  // unbounded allocation) dies cleanly instead of taking down the host.
  // 90s wall is generous: cold `cargo run` rebuilds the compiler before
  // first compile.
  spawnBoundedOrThrow('cargo', args, `sui-compiler ${suiPath}`, {
    cwd: repoRoot,
    timeoutSecs: 90,
  });
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
  // Components (Arc B-1): compile each ported primitive's .sui
  // ahead-of-time. The compiler's `customElements.define()` call
  // happens as a top-level side-effect of the emitted module; the
  // `<tag>.ts` source-side files just `import './<tag>.compiled.js'`
  // so the entire bundle is registration-by-import.
  if (t.componentSources) {
    for (const cs of t.componentSources) {
      await compileSui(cs.htmlSet, { mode: 'component', cssPath: cs.cssPath });
    }
  }
  // Island targets use a synthesised stdin entry instead of a real file.
  const isIsland = Boolean(t.islandBundle);
  const entryAbs = isIsland ? null : join(repoRoot, t.entry);
  const outfile = join(assetsDirFor(t.name), `${t.name}.js`);
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

  const buildInput = isIsland
    ? {
        stdin: {
          contents: t.islandStems
            .map((stem) => `import './dist/${stem}.compiled.js';`)
            .join('\n'),
          resolveDir: join(repoRoot, t.islandDir),
          loader: 'js',
        },
      }
    : { entryPoints: [entryAbs] };

  // `external: ['@ship/ui-runtime']` matches the bare specifier AND
  // its submodules, but the runtime split for built-in template hooks
  // means the SUI compiler emits `import "@ship/ui-runtime/template-hooks"`
  // into views that use `<use-*>` — and that subpath has no importmap
  // entry. Force those imports to resolve locally so esbuild inlines
  // the eight built-ins into the view bundle that needs them.
  const inlineRuntimeSubpathsPlugin = {
    name: 'inline-runtime-subpaths',
    setup(build) {
      build.onResolve({ filter: /^@ship\/ui-runtime\/.+/ }, async (args) => {
        const sub = args.path.slice('@ship/ui-runtime/'.length);
        // Subpath exports may be a directory with index.ts (template-hooks/)
        // or a flat file (patch.ts, route.ts → patch.ts is single-file in
        // the source tree even though `route` is a dir). Prefer the dir
        // form, fall back to the .ts file. Mirror package.json exports
        // logic without re-implementing the resolver wholesale.
        const dirEntry = join(repoRoot, 'packages/ui-runtime/src', `${sub}/index.ts`);
        const fileEntry = join(repoRoot, 'packages/ui-runtime/src', `${sub}.ts`);
        const path = existsSync(dirEntry) ? dirEntry : fileEntry;
        return { path };
      });
    },
  };
  const usesRuntime =
    Array.isArray(t.external) && t.external.includes('@ship/ui-runtime');

  const result = await build({
    ...baseOpts,
    ...buildInput,
    outfile,
    metafile: true,
    ...(t.external ? { external: t.external } : {}),
    ...(t.loaders ? { loader: { ...baseOpts.loader, ...t.loaders } } : {}),
    ...(usesRuntime ? { plugins: [inlineRuntimeSubpathsPlugin] } : {}),
  });

  const inputs = Object.keys(result.metafile.inputs).map((p) =>
    relative(repoRoot, resolve(repoRoot, p)),
  );
  const hash = fileMtimeHash(inputs.map((i) => join(repoRoot, i)));
  mkdirSync(cacheDir, { recursive: true });
  writeFileSync(cacheFile, JSON.stringify({ hash, inputs }, null, 2));

  // Regression guard: views go through the Rust compiler, so the
  // bundle MUST export a `default` (the view setup). Phase 3b.5
  // dropped the `bindings` manifest — every kind is now wired by the
  // compiled view's hydrate() directly. Catches the foundation-level
  // bug where view.ts was bundled directly without sui-compiler.
  if (t.htmlSet) {
    const out = readFileSync(outfile, 'utf8');
    const hasDefault = /\bdefault\b/.test(out);
    if (!hasDefault) {
      throw new Error(
        `[bundles] ${t.name}.js missing required default export. View bundles must be compiled via sui-compiler before esbuild.`,
      );
    }
  }

  // Regression guard (Arc B-1): the main components bundle is registration
  // by side-effect — `customElements.define()` calls only, no template
  // parsing. Island bundles are intentionally excluded from this check
  // since they externalise @ship/ui-dom.
  if (t.componentSources && !isIsland) {
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
      // The compiler emits `component("sui-button", { … })` —
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
        outfile: join(assetsDirFor(t.name), `${t.name}.js`),
      }),
    ),
  );
  await Promise.all(ctxs.map((c) => c.watch()));
  console.log('[bundles] watching for changes…');
}

// Phase 3b.5 (gate N): bundle-size gates. The strict targets in the
// arc spec are aspirational — they assume the next sweep of dead-code
// elimination + smaller hydrate helpers. Current ceilings (just above
// today's measurements) catch regressions; comments record the spec
// target so future shrinkage work can ratchet them down.
// F1 mount-unification ended at 14.9 KiB gz — mount() inlines per-view
// signal/effect plumbing that used to live across attach() in ui-dom.
// ui-dom shrunk from 5.1 KiB → 2.3 KiB in the same change, so the
// system's net SUI runtime is ~17.2 KiB (was ~19.9). Tighten the
// runtime ceiling back to 15 KiB; future tightening targets the inlined
// live-channel + live-state envelope detection.
// Bundle gates: framework only.
//
// `sui-runtime` and `sui-dom` ship on every page; their cost is borne by the
// framework team and warrants a hard ceiling. `sui-components.js` is an
// aggregate of N independently-authored components whose total size is
// determined by the registry shape and the framework's per-component emit
// boilerplate, NOT by any one port. We track its size + per-component
// average as observability, but a single component port can't meaningfully
// move that number, so the gate is non-fatal. When the components bundle
// genuinely needs splitting (rule of thumb: avg-per-component drifts upward
// or the bundle crosses ~80 KiB gz on cold load), the framework team splits
// it into core + extended. That decision lives here, not in port specs.
const BUNDLE_GATES = [
  { name: 'sui-runtime',     ceiling: 16, target: 13, scope: 'bundle' },
  { name: 'sui-dom',         ceiling:  6, target:  4, scope: 'bundle' },
  // Components bundle: observability only, not a fail-build gate.
  // Reports gz size + per-component average so framework owners see drift.
  { name: 'sui-components',  scope: 'observe', perComponentAvgTargetB: 800 },
];

async function enforceBundleGates() {
  for (const gate of BUNDLE_GATES) {
    const outfile = join(assetsDirFor(gate.name), `${gate.name}.js`);
    let body;
    try {
      body = readFileSync(outfile);
    } catch {
      continue; // bundle wasn't built (e.g. islands-only run)
    }
    const gz = gzipSync(body).length;
    const gzKb = gz / 1024;

    // Observability-only bundles: log size + per-component avg, never throw.
    if (gate.scope === 'observe') {
      let avgInfo = '';
      const target = gate.perComponentAvgTargetB;
      if (target) {
        const componentCount = countRegisteredComponents(gate.name);
        if (componentCount > 0) {
          const avgB = Math.round(gz / componentCount);
          const drift = avgB > target ? ' — drift over target, consider audit' : '';
          avgInfo = ` (n=${componentCount}, avg ${avgB} B/component, target ≤ ${target} B${drift})`;
        }
      }
      console.log(`[bundles][observe] ${gate.name}.js gz ${gzKb.toFixed(1)} KiB${avgInfo}`);
      continue;
    }

    // Hard-gated bundles (framework runtime + dom): fail build over ceiling.
    const status =
      gzKb <= gate.target ? 'AT TARGET'
      : gzKb <= gate.ceiling ? 'within ceiling'
      : 'OVER CEILING';
    console.log(
      `[bundles][gate] ${gate.name}.js gz ${gzKb.toFixed(1)} KiB ` +
      `(target ≤ ${gate.target} KiB, ceiling ≤ ${gate.ceiling} KiB) — ${status}`,
    );
    if (gzKb > gate.ceiling) {
      throw new Error(
        `[bundles] ${gate.name}.js gz ${gzKb.toFixed(1)} KiB exceeds ceiling ${gate.ceiling} KiB`,
      );
    }
  }
}

// Count distinct sui-* component imports in the registry index. Used to
// derive a per-component-average size from the aggregated bundle.
function countRegisteredComponents(bundleName) {
  if (bundleName !== 'sui-components') return 0;
  try {
    const indexPath = join(repoRoot, 'packages/ui-components/src/index.ts');
    const src = readFileSync(indexPath, 'utf8');
    const matches = src.match(/^\s*import\s+'\.\/sui-[^']+\/dist\/[^']+\.compiled\.js'/gm) || [];
    return matches.length;
  } catch {
    return 0;
  }
}

async function main() {
  mkdirSync(litAssetsDir, { recursive: true });
  mkdirSync(suiAssetsDir, { recursive: true });
  if (watch) {
    await watchAll();
    return;
  }
  for (const t of targets) {
    await buildOne(t);
  }
  await enforceBundleGates();
}

main().catch((err) => {
  console.error('[bundles] failed:', err);
  process.exit(1);
});
