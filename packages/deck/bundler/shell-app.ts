#!/usr/bin/env bun
//! Deck's shell-app build — produces the shell SPA production bundle
//! (`apps/shell/dist`) that the Tauri webview loads as its `frontendDist`.
//! This is the deck-owned replacement for `vite build`; vite stays only as the
//! dev server (HMR + daemon proxy). Dogfoods deck as the metaframework: deck
//! builds the app, like Next builds its app.
//!
//! Why an explicit `main.ts` entry rather than Bun's native HTML entrypoint:
//! Bun's HTML bundler tries to resolve EVERY `href`/`src` in the document as a
//! bundle input, including root-absolute public assets (`/favicon.ico`,
//! `/ship-logo.svg`) and daemon-served stylesheets (`/api/assets/view-tokens.css`).
//! Those are passthrough/runtime URLs, not build inputs. Rather than fight that,
//! we bundle the two real `/src/*` entries (the module + the chrome stylesheet),
//! copy `public/` verbatim, and rewrite only those two refs in the HTML — every
//! other tag (importmap, the `/_sui/*` loader IIFEs, public hrefs) is preserved
//! byte-for-byte.
//!
//! Engine note: the deck bundler uses `Bun.build`, whose optimizer trails
//! rollup (vite) — the shell chunk is ~2x rollup's gzipped size. That is
//! immaterial here because the consumer (the Tauri webview) loads `dist` from
//! local disk, not over the network; gzip transfer never happens. It WOULD
//! matter for a network-served app, which is why this is recorded as a known
//! deck-engine characteristic in `.artifacts/asset-architecture.md`.

import { copyFile, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { basename, join, resolve } from 'node:path'

export interface ShellBuildOptions {
  /** Shell app root — the dir holding `index.html`, `src/`, `public/`. */
  shellRoot: string
  /** Output dir for the built SPA. Defaults to `<shellRoot>/dist`. */
  outDir?: string
  /** Minify the JS/CSS chunks (default true). */
  minify?: boolean
}

export interface ShellBuildResult {
  outDir: string
  jsHref: string
  cssHref: string
  jsBytes: number
  cssBytes: number
}

/**
 * Derive the external module specifiers from the document's `<script
 * type="importmap">`. The importmap is the single source of truth for which
 * bare specifiers the browser resolves to daemon `/_sui/*` URLs at runtime, so
 * the build must externalize exactly those — never a hand-kept second list that
 * could drift from the map (#157's SSOT discipline). Returns the import keys.
 */
export function importmapExternals(html: string): string[] {
  const m = html.match(/<script\s+type="importmap"\s*>([\s\S]*?)<\/script>/i)
  if (!m) throw new Error('shell-app: no <script type="importmap"> in index.html')
  let map: { imports?: Record<string, string> }
  try {
    map = JSON.parse(m[1]!)
  } catch (err) {
    throw new Error(`shell-app: importmap is not valid JSON: ${String(err)}`)
  }
  const keys = Object.keys(map.imports ?? {})
  if (keys.length === 0) throw new Error('shell-app: importmap has no imports')
  return keys
}

/** Bundle one entry to `outDir/assets/[name]-[hash].<ext>`; return its href. */
async function bundleEntry(
  entry: string,
  external: string[],
  outDir: string,
  minify: boolean,
): Promise<{ href: string; bytes: number }> {
  const result = await Bun.build({
    entrypoints: [entry],
    external,
    outdir: outDir,
    format: 'esm',
    target: 'browser',
    splitting: true,
    minify,
    sourcemap: 'none',
    naming: {
      entry: 'assets/[name]-[hash].[ext]',
      chunk: 'assets/[name]-[hash].[ext]',
      asset: 'assets/[name]-[hash].[ext]',
    },
  })
  if (!result.success) {
    const logs = result.logs.map((l) => `[${l.level}] ${l.message}`).join('\n')
    throw new Error(`shell-app: Bun.build failed for ${entry}:\n${logs || '(no logs)'}`)
  }
  // The entry-point output for JS; the lone emitted file for a CSS entry.
  const out =
    result.outputs.find((o) => o.kind === 'entry-point') ??
    result.outputs.find((o) => o.path.endsWith('.css')) ??
    result.outputs[0]
  if (!out) throw new Error(`shell-app: Bun.build produced no output for ${entry}`)
  return { href: '/' + out.path.slice(outDir.length + 1), bytes: (await out.text()).length }
}

/**
 * Build the shell SPA. Requires the `.sui` sources to already be compiled to
 * disk (`sui-build`) — the shell graph imports the compiled artifacts, matching
 * the contract `vite build` consumed. Steps: bundle the JS entry, bundle the
 * chrome stylesheet, copy `public/`, then rewrite the two `/src/*` refs in
 * `index.html` to the hashed asset hrefs.
 */
export async function buildShellApp(opts: ShellBuildOptions): Promise<ShellBuildResult> {
  const shellRoot = resolve(opts.shellRoot)
  const outDir = resolve(opts.outDir ?? join(shellRoot, 'dist'))
  const minify = opts.minify ?? true

  const indexPath = join(shellRoot, 'index.html')
  if (!existsSync(indexPath)) throw new Error(`shell-app: no index.html at ${indexPath}`)
  const html = await readFile(indexPath, 'utf8')

  // The two real build inputs the HTML references under /src.
  const jsEntry = join(shellRoot, 'src/main.ts')
  const cssEntry = join(shellRoot, 'src/chrome/layout.css')
  for (const [label, p] of [['main.ts', jsEntry], ['layout.css', cssEntry]] as const) {
    if (!existsSync(p)) throw new Error(`shell-app: expected ${label} at ${p}`)
  }

  const external = importmapExternals(html)

  // Empty the output dir first (vite's `emptyOutDir`) so hashed chunks from a
  // previous build don't accumulate as orphans. Guard against a mis-pointed
  // outDir: only wipe a conventional build dir (`dist` or under `.deck`).
  const base = basename(outDir)
  if (existsSync(outDir) && (base === 'dist' || outDir.includes('/.deck'))) {
    await rm(outDir, { recursive: true, force: true })
  }
  await mkdir(join(outDir, 'assets'), { recursive: true })

  const js = await bundleEntry(jsEntry, external, outDir, minify)
  // The stylesheet has no JS deps; externals are irrelevant but harmless.
  const css = await bundleEntry(cssEntry, external, outDir, minify)

  // Copy public/ verbatim — favicons, logos referenced by root-absolute href.
  const publicDir = join(shellRoot, 'public')
  if (existsSync(publicDir)) {
    for (const f of await readdir(publicDir, { withFileTypes: true })) {
      if (f.isFile()) await copyFile(join(publicDir, f.name), join(outDir, f.name))
    }
  }

  // Rewrite ONLY the two /src refs; leave importmap, loader IIFEs, public hrefs.
  const rewritten = html
    .replace('/src/main.ts', js.href)
    .replace('/src/chrome/layout.css', css.href)
  await writeFile(join(outDir, 'index.html'), rewritten)

  return { outDir, jsHref: js.href, cssHref: css.href, jsBytes: js.bytes, cssBytes: css.bytes }
}

// ── CLI ───────────────────────────────────────────────────────────────────────
// `bun shell-app.ts --shell-root apps/shell [--out <dir>] [--no-minify]`. The
// `.sui` sources must be compiled first (`sui-build`); the shell `build` script
// chains them. Used by `apps/shell/package.json` in place of `vite build`.
if (import.meta.main) {
  const argv = process.argv
  const arg = (flag: string): string | undefined => {
    const i = argv.indexOf(flag)
    return i !== -1 && i + 1 < argv.length ? argv[i + 1] : undefined
  }
  const shellRoot = arg('--shell-root') ?? process.cwd()
  const outDir = arg('--out')
  const minify = !argv.includes('--no-minify')
  try {
    const r = await buildShellApp({ shellRoot, outDir, minify })
    process.stdout.write(
      `✓ shell-app → ${r.outDir}\n  js  ${r.jsHref} (${r.jsBytes} bytes)\n  css ${r.cssHref} (${r.cssBytes} bytes)\n`,
    )
  } catch (err) {
    process.stderr.write(`✗ shell-app: ${err instanceof Error ? err.message : String(err)}\n`)
    process.exit(1)
  }
}
