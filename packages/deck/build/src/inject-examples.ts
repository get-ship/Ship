// Inject co-located `<tag>.examples.ts` into the component's CEM declaration.
//
// Examples are authored next to a component (the source of truth) but the
// Studio explorer, web-qa fixtures, and docs read them from the aggregated
// custom-elements manifest — the standard, pack-portable channel that also
// works for third-party packs (they ship a CEM; this writes the `examples`
// custom property onto each component's declaration).
//
// The Rust compiler can't execute the TS examples, so this runs in the bun
// build layer (bun imports `.examples.ts` natively). Run it after the
// per-component CEMs are emitted and before `aggregate-cem`.

import { readdir, readFile, writeFile } from 'node:fs/promises'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const SKIP_DIRS = new Set(['node_modules', 'dist', 'target', '.git'])
const EXAMPLES_SUFFIX = '.examples.ts'

export interface InjectOptions {
  /** Source dirs to scan recursively for `<tag>.examples.ts`. */
  sources: string[]
  /**
   * Map an examples file path to the CEM file that should carry its examples.
   * Defaults to the sui-out mirror: `.ship/sui-out/<repo-rel>/<tag>.custom-elements.json`.
   */
  cemFor: (examplesPath: string) => string
  warn?: (msg: string) => void
}

export interface InjectResult {
  injected: number
  tags: string[]
}

async function walk(dir: string, out: string[]): Promise<void> {
  let entries
  try {
    entries = await readdir(dir, { withFileTypes: true })
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue
    const p = join(dir, entry.name)
    if (entry.isDirectory()) await walk(p, out)
    else if (entry.isFile() && entry.name.endsWith(EXAMPLES_SUFFIX)) out.push(p)
  }
}

/** `<tag>` from `…/<tag>.examples.ts`. */
function tagOf(examplesPath: string): string {
  const file = examplesPath.split(/[\\/]/).pop() ?? ''
  return file.slice(0, -EXAMPLES_SUFFIX.length)
}

interface CemDecl { tagName?: string; examples?: unknown; [k: string]: unknown }
interface CemModule { declarations?: CemDecl[]; [k: string]: unknown }

/**
 * For every `<tag>.examples.ts` under `sources`, import its `examples` export
 * and write it onto the `tagName === <tag>` declaration in the mapped CEM.
 * Idempotent: re-running overwrites the `examples` property in place.
 */
export async function injectExamples(opts: InjectOptions): Promise<InjectResult> {
  const warn = opts.warn ?? ((m) => console.warn(m))
  const found: string[] = []
  for (const src of opts.sources) await walk(src, found)
  found.sort()

  const tags: string[] = []
  for (const exPath of found) {
    const tag = tagOf(exPath)
    const cemPath = opts.cemFor(exPath)

    let examples: unknown
    try {
      // Dynamic import resolves relative to this module; use an absolute
      // file URL so caller-supplied relative source dirs work.
      const mod = (await import(pathToFileURL(resolve(exPath)).href)) as { examples?: unknown }
      examples = mod.examples
    } catch (err) {
      warn(`[inject-examples] cannot import ${exPath}: ${String(err)}`)
      continue
    }
    if (!Array.isArray(examples) || examples.length === 0) {
      warn(`[inject-examples] ${exPath} exports no non-empty \`examples\` array`)
      continue
    }

    let raw: string
    try {
      raw = await readFile(cemPath, 'utf8')
    } catch {
      warn(`[inject-examples] no CEM for ${tag} at ${cemPath} (compile it first)`)
      continue
    }
    const cem = JSON.parse(raw) as { modules?: CemModule[] }
    const decl = (cem.modules ?? [])
      .flatMap((m) => m.declarations ?? [])
      .find((d) => d.tagName === tag)
    if (!decl) {
      warn(`[inject-examples] no declaration tagName="${tag}" in ${cemPath}`)
      continue
    }

    decl.examples = examples
    await writeFile(cemPath, JSON.stringify(cem, null, 2) + '\n')
    tags.push(tag)
  }
  return { injected: tags.length, tags }
}

/**
 * Pack-dist mapping: `<pack>/src/**\/<tag>.examples.ts` → `<pack>/dist/custom-elements.json`.
 * This is the CEM that ships with the installed pack and the daemon aggregates,
 * so injecting here is what makes examples reach the explorer (and third-party packs).
 */
export function packDistCemFor(examplesPath: string): string {
  const m = examplesPath.match(/^(.*)[\\/]src[\\/].*$/)
  const packRoot = m ? m[1] : examplesPath
  return join(packRoot, 'dist', 'custom-elements.json')
}

/** Default mirror mapping: repo-relative source → `.ship/sui-out/<rel>` CEM. */
export function suiOutCemFor(repoRoot: string): (examplesPath: string) => string {
  return (examplesPath) => {
    const rel = examplesPath.startsWith(repoRoot)
      ? examplesPath.slice(repoRoot.length).replace(/^[\\/]/, '')
      : examplesPath
    return join(repoRoot, '.ship', 'sui-out', rel).replace(
      new RegExp(`${EXAMPLES_SUFFIX.replace('.', '\\.')}$`),
      '.custom-elements.json',
    )
  }
}
