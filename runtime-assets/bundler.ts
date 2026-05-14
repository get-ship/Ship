#!/usr/bin/env bun
// Bun worker — owns the asset bundle pipeline for the Ship daemon.
//
// Protocol: one JSON line per message on each direction.
//   shipd → worker   stdin
//   worker → shipd   stdout
//   worker → shipd   stdout (unsolicited Events while watching)
//
// Start:  bun run ~/.ship/runtime/bundler.ts
// Stop:   send { id, kind: 'shutdown' } or close stdin

import { existsSync, lstatSync, mkdirSync, readdirSync, watch } from 'node:fs'
import { mkdtemp, rm } from 'node:fs/promises'
import { homedir, tmpdir } from 'node:os'
import { basename, dirname, isAbsolute, join, resolve } from 'node:path'
import { createInterface } from 'node:readline'

// ── Protocol types ───────────────────────────────────────────────────────────

type Request =
  | { id: string; kind: 'build'; target: string }
  | { id: string; kind: 'build_all' }
  | { id: string; kind: 'watch_start' }
  | { id: string; kind: 'watch_stop' }
  | { id: string; kind: 'shutdown' }

type Response =
  | { id: string; kind: 'bundle'; target: string; bytes: number; outPath: string; manifest: BundleManifest }
  | { id: string; kind: 'ack' }
  | { id: string; kind: 'error'; target?: string; message: string; stack?: string }

type Event =
  | { kind: 'module_changed'; target: string; outPath: string; bytes: number }
  | { kind: 'compile_error'; target: string; message: string; line?: number; col?: number; file?: string }
  | { kind: 'ready' }

interface BundleManifest {
  target: string
  outPath: string
  bytes: number
  inputs: string[]
}

// ── Repo layout ──────────────────────────────────────────────────────────────

const repoRoot = process.env.SHIP_REPO_ROOT ?? process.cwd()
const litAssetsDir = resolve(repoRoot, 'apps/shipd/assets')
const suiAssetsDir = resolve(repoRoot, 'crates/ship-sui-server/assets')

function assetsDirFor(name: string): string {
  if (name.startsWith('sui-') || name.startsWith('views/')) return suiAssetsDir
  return litAssetsDir
}

// ── Target definitions ───────────────────────────────────────────────────────

interface BundleTarget {
  name: string
  /** Real file path, or `virtual:<id>` for synthetic entries (see virtualEntry). */
  entry: string
  external?: string[]
  outFile?: string
  outDirOverride?: string
  /** When set, suiPlugin treats `entry` as a virtual id and serves this string
   *  as its contents. Imports inside should use absolute paths so they resolve
   *  regardless of where the virtual id "lives". */
  virtualEntry?: string
}

// Islands aggregate one bundle per directory under packages/ui-components/src/islands/.
// The bundle is built from a synthetic entry that imports every .sui in the
// directory by absolute path — Bun.build's plugin handles each .sui via onLoad,
// and the relative `import setup from "./<stem>.controller.ts"` inside each
// compiled output resolves against the .sui's own directory. No precompile-to-
// disk step is needed.
function discoverIslandBundles(): BundleTarget[] {
  const islandsDir = resolve(repoRoot, 'packages/ui-components/src/islands')
  if (!existsSync(islandsDir)) return []
  const out: BundleTarget[] = []
  for (const e of readdirSync(islandsDir, { withFileTypes: true })) {
    if (!e.isDirectory()) continue
    const dir = resolve(islandsDir, e.name)
    const suiPaths = readdirSync(dir, { withFileTypes: true })
      .filter((f) => f.isFile() && f.name.endsWith('.sui'))
      .map((f) => resolve(dir, f.name))
      .sort()
    if (suiPaths.length === 0) continue
    out.push({
      name: `sui-islands/${e.name}`,
      // virtualEntry contents are written to a tmp file at build time. The
      // imports use absolute paths so the tmp location is irrelevant.
      entry: '<virtual>',
      virtualEntry: suiPaths.map((p) => `import ${JSON.stringify(p)}`).join('\n') + '\n',
      external: ['@ship/sui-runtime', '@ship/sui-dom'],
      outFile: `sui-islands/${e.name}.js`,
    })
  }
  return out
}

// Views: one bundle per discovered view, entrypoint is the leaf .sui file.
// The plugin's onLoad transforms the .sui via sui-compiler, and the
// resulting `import setup from "./controller.ts"` resolves to the .sui's
// own directory.
//
// Two layouts are honored, both rooted at the workspace (SHIP_REPO_ROOT
// is the daemon's attached workspace, NOT the Ship repo):
//
// - `<workspace>/views/<name>/page.sui`        — top-level app convention
// - `<workspace>/.ship/views/<name>/index.sui` — colocated with .ship/ config
//
// Top-level wins when both exist (mirrors the resolver's priority order,
// which lets workspace `views/` shadow `.ship/views/` by name).
function discoverViewBundles(): BundleTarget[] {
  const out: BundleTarget[] = []
  const seen = new Set<string>()
  const candidates: Array<{ dir: string; leaf: string }> = [
    { dir: resolve(repoRoot, 'views'), leaf: 'page.sui' },
    { dir: resolve(repoRoot, '.ship', 'views'), leaf: 'index.sui' },
  ]
  for (const { dir, leaf } of candidates) {
    if (!existsSync(dir)) continue
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      if (!e.isDirectory()) continue
      if (seen.has(e.name)) continue
      const entry = resolve(dir, e.name, leaf)
      if (!existsSync(entry)) continue
      seen.add(e.name)
      out.push({
        name: `views/${e.name}/view`,
        entry,
        external: ['@ship/sui-runtime', '@ship/sui-runtime/template-hooks', '@ship/sui-dom', '@ship/sdk-client'],
        outFile: `views/${e.name}/view.js`,
      })
    }
  }
  return out
}

const TARGETS: BundleTarget[] = [
  { name: 'ship-elements', entry: 'packages/ship-set/src/web-components/index.ts' },
  { name: 'ship-code-editor', entry: 'packages/ship-set/src/web-components/islands/ship-code-editor.ts' },
  { name: 'ship-markdown-editor', entry: 'packages/ship-set/src/web-components/islands/ship-markdown-editor.ts' },
  { name: 'ship-terminal', entry: 'packages/ship-set/src/web-components/islands/ship-terminal.ts' },
  { name: 'sui-runtime', entry: 'packages/sui-runtime/src/index.ts' },
  { name: 'sui-dom', entry: 'packages/sui-dom/src/index.ts', external: ['@ship/sui-runtime'] },
  { name: 'sui-sdk-client', entry: 'packages/ship-sdk-client/src/index.ts', external: ['@ship/sui-runtime'] },
  {
    name: 'sui-components',
    entry: 'packages/ui-components/src/index.ts',
    external: ['@ship/sui-runtime'],
  },
  ...discoverIslandBundles(),
  ...discoverViewBundles(),
]

// ── sui-compiler integration ──────────────────────────────────────────────────

function locateCompiler(): string {
  const home = join(homedir(), '.ship', 'runtime', 'sui-compiler')
  if (existsSync(home)) return home

  let cur = isAbsolute(repoRoot) ? repoRoot : resolve(repoRoot)
  let topmost: string | null = null
  for (let i = 0; i < 16; i++) {
    if (existsSync(join(cur, 'Cargo.toml'))) topmost = cur
    const parent = dirname(cur)
    if (parent === cur) break
    cur = parent
  }
  if (topmost) {
    const release = join(topmost, 'target', 'release', 'sui-compiler')
    if (existsSync(release)) return release
    const debug = join(topmost, 'target', 'debug', 'sui-compiler')
    if (existsSync(debug)) return debug
  }
  throw new Error('bundler: cannot locate sui-compiler; run `ship runtime install`')
}

const compilerPath = locateCompiler()

// CEM cache, populated as components are compiled. Aggregated into
// sui-custom-elements.json after each build_all. Keyed by .sui path so
// repeat compiles overwrite cleanly.
const cemCache = new Map<string, unknown>()

async function compileSuiFile(
  filePath: string,
  mode: 'component' | 'view',
  stem: string,
): Promise<string> {
  const out = await mkdtemp(join(tmpdir(), 'sui-bun-'))
  try {
    const args = ['build', filePath, '--out', out, '--no-dts']
    if (mode === 'component') args.push('--component-name', stem)
    const proc = Bun.spawn({ cmd: [compilerPath, ...args], stdout: 'pipe', stderr: 'pipe', stdin: 'ignore' })
    const [stderr, code] = await Promise.all([new Response(proc.stderr).text(), proc.exited])
    if (code !== 0) {
      const msg = `sui-compiler exit ${code} for ${filePath}\n${stderr.trim()}`
      const locMatch = stderr.match(/(\d+):(\d+)/)
      const err = Object.assign(new Error(msg), {
        compileErrorLine: locMatch ? Number(locMatch[1]) : undefined,
        compileErrorCol: locMatch ? Number(locMatch[2]) : undefined,
        compileErrorFile: filePath,
      })
      throw err
    }
    const outFile = join(out, `${stem}.js`)
    const code_ = await Bun.file(outFile).text()
    // Capture the CEM if the compiler emitted one (component mode does).
    const cemFile = join(out, `${stem}.custom-elements.json`)
    if (existsSync(cemFile)) {
      try {
        cemCache.set(filePath, JSON.parse(await Bun.file(cemFile).text()))
      } catch {
        // Malformed CEM — skip; aggregation tolerates missing entries.
      }
    }
    return code_
  } finally {
    await rm(out, { recursive: true, force: true })
  }
}

// Merge every cached CEM into one document the daemon serves as
// /api/assets/custom-elements.json. Empty modules array is a valid
// fallback when no components have been compiled yet.
async function aggregateCem(): Promise<void> {
  const out = { schemaVersion: '1.0.0', readme: '', modules: [] as unknown[] }
  for (const cem of cemCache.values()) {
    const modules = (cem as { modules?: unknown[] })?.modules
    if (Array.isArray(modules)) out.modules.push(...modules)
  }
  const target = resolve(repoRoot, 'crates/ship-ui-framework/assets/sui-custom-elements.json')
  mkdirSync(dirname(target), { recursive: true })
  await Bun.write(target, JSON.stringify(out))
}

// ── Bun plugin for .sui + virtual entries ────────────────────────────────────

function detectMode(absPath: string): { mode: 'component' | 'view'; stem: string } {
  const stem = basename(absPath, '.sui')
  const parent = basename(dirname(absPath))
  const mode: 'component' | 'view' = stem === parent && stem.length > 0 ? 'component' : 'view'
  return { mode, stem }
}

function suiPlugin() {
  return {
    name: 'sui',
    setup(build: { onResolve: any; onLoad: any }) {
      // Redirect on-disk `.compiled.js` imports back to their `.sui` source
      // so the plugin transforms them fresh. Post dist-relocation, imports
      // name the artifact next to its source (`<tag>/<tag>.compiled.js`
      // shadowing `<tag>/<tag>.sui`) — the actual artifact lives in the
      // project mirror tree at `<repo-root>/.ship/sui-out/`, but the
      // bundler reads the .sui directly so no on-disk shadow is needed.
      build.onResolve({ filter: /\.compiled\.js$/ }, ({ path: id, importer }: any) => {
        const abs = isAbsolute(id) ? id : resolve(dirname(importer ?? ''), id)
        const stem = basename(abs, '.compiled.js')
        const sui = resolve(dirname(abs), `${stem}.sui`)
        if (existsSync(sui)) return { path: sui }
        return null
      })
      build.onLoad({ filter: /\.sui$/ }, async ({ path }: { path: string }) => {
        const { mode, stem } = detectMode(path)
        const code = await compileSuiFile(path, mode, stem)
        return { contents: code, loader: 'js' }
      })
    },
  }
}

// ── Build a single target ─────────────────────────────────────────────────────

const inMemoryBundles = new Map<string, { bytes: number; content: string; outPath: string }>()

async function buildTarget(
  target: BundleTarget,
): Promise<{ bytes: number; outPath: string; manifest: BundleManifest }> {
  const outDir = target.outDirOverride ?? assetsDirFor(target.name)
  const outFile = target.outFile ?? `${target.name}.js`
  const outPath = join(outDir, outFile)
  mkdirSync(dirname(outPath), { recursive: true })

  // Synthetic entries (sui-islands aggregates) need a real on-disk file so
  // Bun.build can derive a root directory. We write the virtual contents to
  // a temp dir; the imports inside use absolute paths so the temp location
  // is irrelevant to resolution.
  let entrypoint: string
  let tmpEntryDir: string | null = null
  if (target.virtualEntry) {
    tmpEntryDir = await mkdtemp(join(tmpdir(), 'sui-entry-'))
    entrypoint = join(tmpEntryDir, '_entry.ts')
    await Bun.write(entrypoint, target.virtualEntry)
  } else {
    entrypoint = resolve(repoRoot, target.entry)
  }

  try {
    let result: Awaited<ReturnType<typeof Bun.build>>
    try {
      result = await Bun.build({
        entrypoints: [entrypoint],
        external: target.external,
        format: 'esm',
        target: 'browser',
        minify: false,
        sourcemap: 'none',
        plugins: [suiPlugin()],
      })
    } catch (err: any) {
      // Bun.build throws bare `Bundle failed` when the pipeline blows up
      // before logs accumulate. Surface the underlying message.
      const msg = err?.message ?? String(err)
      throw new Error(`Bun.build threw for ${target.name}: ${msg}`)
    }

    if (!result.success) {
      const msgs = result.logs.length
        ? result.logs.map((l) => `[${l.level}] ${l.message}`).join('\n')
        : '(no log output from Bun.build)'
      throw new Error(`Bun.build failed for ${target.name}:\n${msgs}`)
    }

    const [artifact] = result.outputs
    if (!artifact) throw new Error(`Bun.build produced no output for ${target.name}`)

    const content = await artifact.text()
    const bytes = content.length
    await Bun.write(outPath, content)

    inMemoryBundles.set(target.name, { bytes, content, outPath })

    return {
      bytes,
      outPath,
      manifest: { target: target.name, outPath, bytes, inputs: [entrypoint] },
    }
  } finally {
    if (tmpEntryDir) await rm(tmpEntryDir, { recursive: true, force: true })
  }
}

// ── File watcher ──────────────────────────────────────────────────────────────

type FSWatcher = ReturnType<typeof watch>
let activeWatchers: FSWatcher[] = []

// Debounce: skip rebuild requests within 150ms of the last one per target
const rebuildDebounce = new Map<string, ReturnType<typeof setTimeout>>()

function debounceRebuild(targetName: string, cb: () => void) {
  const existing = rebuildDebounce.get(targetName)
  if (existing) clearTimeout(existing)
  rebuildDebounce.set(targetName, setTimeout(() => {
    rebuildDebounce.delete(targetName)
    cb()
  }, 150))
}

function targetsForFile(absPath: string): BundleTarget[] {
  return TARGETS.filter((t) => {
    if (t.precompile?.some((p) => p.path === absPath)) return true
    const entryAbs = resolve(repoRoot, t.entry)
    if (entryAbs === absPath) return true
    // Rough heuristic: same package directory
    const entryDir = dirname(entryAbs)
    return absPath.startsWith(entryDir + '/')
  })
}

// Per-directory watch. We deliberately avoid `recursive: true` because Bun's
// Linux implementation walks symlinks and aborts the entire subtree the first
// time it hits a broken one — common in monorepos with stale `node_modules/`
// symlinks. Walking real subdirectories ourselves keeps each watcher isolated.
function watchDirRecursive(root: string, onFile: (absPath: string) => void) {
  const SKIP = new Set(['node_modules', '.git', '.ship', 'target', 'dist', '.cache'])

  function attach(dir: string) {
    let entries: import('node:fs').Dirent[]
    try {
      entries = readdirSync(dir, { withFileTypes: true })
    } catch {
      return
    }
    let w: FSWatcher
    try {
      w = watch(dir, (event, filename) => {
        if (!filename) return
        const child = join(dir, filename)
        // New directory created? Attach a watcher to it too.
        if (event === 'rename') {
          try {
            const st = lstatSync(child)
            if (st.isDirectory() && !SKIP.has(filename)) attach(child)
          } catch {
            // child was deleted between event and stat — ignore
          }
        }
        onFile(child)
      })
    } catch {
      return
    }
    w.on('error', (err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err)
      process.stderr.write(`[bundler] watch error on ${dir}: ${msg}\n`)
    })
    activeWatchers.push(w)
    for (const e of entries) {
      if (!e.isDirectory()) continue
      if (SKIP.has(e.name)) continue
      attach(join(dir, e.name))
    }
  }
  attach(root)
}

function startWatching() {
  if (activeWatchers.length > 0) return

  const watchDirs: string[] = []
  for (const d of ['packages', 'views', '.ship/views']) {
    const abs = resolve(repoRoot, d)
    if (existsSync(abs)) watchDirs.push(abs)
  }
  if (watchDirs.length === 0) return

  for (const dir of watchDirs) {
    watchDirRecursive(dir, (absPath) => {
      const ext = absPath.split('.').pop()
      if (!['ts', 'tsx', 'js', 'sui'].includes(ext ?? '')) return

      const affected = targetsForFile(absPath)
      const toRebuild = affected.length > 0 ? affected : TARGETS

      for (const target of toRebuild) {
        debounceRebuild(target.name, async () => {
          try {
            const { bytes, outPath } = await buildTarget(target)
            emit({ kind: 'module_changed', target: target.name, outPath, bytes })
          } catch (err: any) {
            const msg: string = err?.message ?? String(err)
            emit({
              kind: 'compile_error',
              target: target.name,
              message: msg,
              line: err?.compileErrorLine,
              col: err?.compileErrorCol,
              file: err?.compileErrorFile ?? absPath,
            })
          }
        })
      }
    })
  }
}

function stopWatching() {
  for (const w of activeWatchers) {
    try { w.close() } catch {}
  }
  activeWatchers = []
  for (const t of rebuildDebounce.values()) clearTimeout(t)
  rebuildDebounce.clear()
}

// ── IPC I/O ───────────────────────────────────────────────────────────────────

function emit(msg: Response | Event) {
  process.stdout.write(JSON.stringify(msg) + '\n')
}

// ── Main loop ─────────────────────────────────────────────────────────────────

// CLI mode: `--all` runs a one-shot build of the sui-* family + views and
// exits. This is the asset set `crates/ship-sui-server/build.rs` requires
// via its ASSETS const, so the root `bun run build:bundles` script can fan
// out here in CI without scripting the stdin/stdout JSON protocol.
//
// The `ship-*` (lit) targets are intentionally excluded — they belong to
// `@ship/set`'s own build path (`bun run --filter '@ship/set' build` + `... icons`)
// and depend on a generated lucide registry that lives outside this
// bundler's dependency graph. Watch mode still rebuilds them on edit.
if (process.argv.includes('--all')) {
  let failed = false
  const allTargets = TARGETS.filter(
    (t) => t.name.startsWith('sui-') || t.name.startsWith('views/'),
  )
  for (const target of allTargets) {
    try {
      const { bytes } = await buildTarget(target)
      process.stdout.write(`✓ ${target.name} (${bytes} bytes)\n`)
    } catch (err: any) {
      failed = true
      process.stderr.write(`✗ ${target.name}: ${err?.message ?? String(err)}\n`)
    }
  }
  try {
    await aggregateCem()
    process.stdout.write(`✓ sui-custom-elements.json (${cemCache.size} components)\n`)
  } catch (err: any) {
    failed = true
    process.stderr.write(`✗ aggregateCem: ${err?.message ?? String(err)}\n`)
  }
  process.exit(failed ? 1 : 0)
}

emit({ kind: 'ready' })

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity })

rl.on('line', async (line) => {
  const trimmed = line.trim()
  if (!trimmed) return

  let req: Request
  try {
    req = JSON.parse(trimmed)
  } catch {
    return
  }

  switch (req.kind) {
    case 'build': {
      const target = TARGETS.find((t) => t.name === req.target)
      if (!target) {
        emit({ id: req.id, kind: 'error', message: `unknown target: ${req.target}` })
        return
      }
      try {
        const { bytes, outPath, manifest } = await buildTarget(target)
        emit({ id: req.id, kind: 'bundle', target: req.target, bytes, outPath, manifest })
      } catch (err: any) {
        emit({
          id: req.id,
          kind: 'error',
          target: req.target,
          message: err?.message ?? String(err),
          stack: err?.stack,
        })
      }
      break
    }

    case 'build_all': {
      for (const target of TARGETS) {
        try {
          const { bytes, outPath, manifest } = await buildTarget(target)
          emit({ id: req.id, kind: 'bundle', target: target.name, bytes, outPath, manifest })
        } catch (err: any) {
          emit({
            id: req.id,
            kind: 'error',
            target: target.name,
            message: err?.message ?? String(err),
            stack: err?.stack,
          })
        }
      }
      try {
        await aggregateCem()
      } catch (err: any) {
        emit({
          id: req.id,
          kind: 'error',
          message: `aggregateCem failed: ${err?.message ?? String(err)}`,
          stack: err?.stack,
        })
      }
      break
    }

    case 'watch_start':
      startWatching()
      emit({ id: req.id, kind: 'ack' })
      break

    case 'watch_stop':
      stopWatching()
      emit({ id: req.id, kind: 'ack' })
      break

    case 'shutdown':
      stopWatching()
      emit({ id: req.id, kind: 'ack' })
      process.exit(0)
  }
})

rl.on('close', () => {
  stopWatching()
  process.exit(0)
})
