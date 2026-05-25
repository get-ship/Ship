import { readdir, readFile } from 'node:fs/promises'
import { join, relative, sep } from 'node:path'

const SKIP_DIRS = new Set(['node_modules', 'dist', 'target', '.git'])
const CEM_SUFFIX = '.custom-elements.json'

export interface AggregateOptions {
  /** Component source dirs to scan recursively for `*.custom-elements.json`. */
  sources: string[]
  /** Optional schemaVersion override. Defaults to `"1.0.0"`. */
  schemaVersion?: string
  /** Optional readme content. Defaults to `""`. */
  readme?: string
  /** Sink for duplicate-tagName warnings. Defaults to `console.warn`. */
  warn?: (msg: string) => void
}

export interface CemRoot {
  schemaVersion: string
  readme: string
  modules: CemModule[]
}

export interface CemModule {
  kind?: string
  path?: string
  declarations?: CemDeclaration[]
  exports?: unknown[]
  [key: string]: unknown
}

interface CemDeclaration {
  tagName?: string
  [key: string]: unknown
}

export interface CemFile {
  /** Absolute path to the per-component CEM source (used for dedup-warning provenance and stable ordering). */
  filePath: string
  modules: CemModule[]
}

async function walkCemFiles(dir: string, out: string[]): Promise<void> {
  let entries
  try {
    entries = await readdir(dir, { withFileTypes: true })
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue
    const p = join(dir, entry.name)
    if (entry.isDirectory()) {
      await walkCemFiles(p, out)
    } else if (entry.isFile() && entry.name.endsWith(CEM_SUFFIX)) {
      out.push(p)
    }
  }
}

async function readCemFile(filePath: string): Promise<CemFile> {
  const raw = await readFile(filePath, 'utf8')
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    throw new Error(`invalid JSON in ${filePath}: ${msg}`)
  }
  const modules = (parsed as { modules?: CemModule[] })?.modules ?? []
  if (!Array.isArray(modules)) {
    throw new Error(`expected modules[] in ${filePath}, got ${typeof modules}`)
  }
  return { filePath, modules }
}

/**
 * Walk source dirs for per-component CEMs, concatenate their `modules`
 * deterministically (sorted by source-path) and dedupe on `tagName`
 * (first-path-wins).
 *
 * Output is JSON-stable: the same inputs produce byte-identical bytes
 * across runs and platforms.
 */
export async function aggregateCem(opts: AggregateOptions): Promise<CemRoot> {
  const filePaths: string[] = []
  for (const src of opts.sources) {
    await walkCemFiles(src, filePaths)
  }
  filePaths.sort()

  const files: CemFile[] = []
  for (const p of filePaths) {
    files.push(await readCemFile(p))
  }

  return aggregateLoadedCem(files, opts)
}

/**
 * Merge + sort + dedup pipeline for already-parsed per-component CEMs. Use
 * this when the caller has the CEM payloads in memory (e.g. the dev-loop
 * bundler caches them as components are compiled) and just needs the same
 * deterministic merge guarantee as the filesystem-walk variant.
 *
 * `files[]` is sorted by `filePath` so first-path-wins dedup is stable
 * regardless of caller-supplied order. The final `modules[]` is sorted by
 * each module's own `path`.
 */
export function aggregateLoadedCem(
  files: CemFile[],
  opts: Partial<AggregateOptions> = {},
): CemRoot {
  const warn = opts.warn ?? ((msg) => console.warn(msg))
  const schemaVersion = opts.schemaVersion ?? '1.0.0'
  const readme = opts.readme ?? ''

  const sorted = [...files].sort((a, b) => a.filePath.localeCompare(b.filePath))

  const seenTags = new Map<string, string>()
  const modules: CemModule[] = []
  for (const file of sorted) {
    for (const mod of file.modules) {
      const tags = collectTagNames(mod)
      const dupes = tags.filter((t) => seenTags.has(t))
      if (dupes.length > 0) {
        for (const tag of dupes) {
          warn(
            `[aggregate-cem] duplicate tagName "${tag}" in ${file.filePath}; ` +
              `keeping ${seenTags.get(tag)}`,
          )
        }
        continue
      }
      for (const tag of tags) seenTags.set(tag, file.filePath)
      modules.push(mod)
    }
  }

  modules.sort((a, b) => sortKey(a).localeCompare(sortKey(b)))
  return { schemaVersion, readme, modules }
}

function collectTagNames(mod: CemModule): string[] {
  const tags: string[] = []
  for (const decl of mod.declarations ?? []) {
    if (typeof decl.tagName === 'string') tags.push(decl.tagName)
  }
  return tags
}

function sortKey(mod: CemModule): string {
  return typeof mod.path === 'string' ? mod.path : ''
}

/** Pretty-print a CEM root with deterministic 2-space indent + trailing newline. */
export function stringifyCem(root: CemRoot): string {
  return JSON.stringify(root, null, 2) + '\n'
}

/** Convert a path under `packageRoot` (or absolute) to absolute form. */
export function resolveCemSource(packageRoot: string, source: string): string {
  if (source.startsWith(sep) || /^[a-zA-Z]:[\\/]/.test(source)) return source
  return join(packageRoot, source)
}

/** Relative-path helper for warning/diagnostic output. */
export function relCemPath(packageRoot: string, p: string): string {
  return relative(packageRoot, p) || p
}
