// Workspace discovery — reads the root `package.json` `workspaces.packages`
// glob list and resolves each entry to a concrete package root. Mirrors the
// (small) subset of bun/npm workspace semantics the aggregator needs: glob
// `*` is the only special char (no `**`), and every match must have a
// `package.json` with a `name` field.

import { readFile, readdir, stat } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { join, resolve } from 'node:path'
import type { WorkspacePackage } from './types.js'

interface RootManifest {
  workspaces?: { packages?: string[] } | string[]
}

async function readJson<T>(path: string): Promise<T | null> {
  try {
    const raw = await readFile(path, 'utf8')
    return JSON.parse(raw) as T
  } catch {
    return null
  }
}

export async function workspaceGlobs(repoRoot: string): Promise<string[]> {
  const manifest = await readJson<RootManifest>(join(repoRoot, 'package.json'))
  if (!manifest) return []
  const ws = manifest.workspaces
  if (Array.isArray(ws)) return ws
  return ws?.packages ?? []
}

/** Expand one workspace glob entry against the repo root. Supports a single
 *  trailing `*` (e.g. `packages/*`); literal paths return [self]. */
async function expandGlob(repoRoot: string, glob: string): Promise<string[]> {
  if (!glob.includes('*')) {
    const abs = resolve(repoRoot, glob)
    return existsSync(abs) ? [abs] : []
  }
  const sepIdx = glob.lastIndexOf('/')
  const parentRel = sepIdx >= 0 ? glob.slice(0, sepIdx) : '.'
  const pattern = sepIdx >= 0 ? glob.slice(sepIdx + 1) : glob
  if (pattern !== '*') {
    // Anchor stricter than `*` (e.g. `pkg-*`) is unused in this repo. If a
    // future glob needs it, swap in a minimatch dep here. For now: explicit
    // failure so the aggregator never silently drops packages.
    throw new Error(`workspace glob "${glob}" uses unsupported pattern "${pattern}"`)
  }
  const parentAbs = resolve(repoRoot, parentRel)
  let entries: string[]
  try {
    entries = await readdir(parentAbs)
  } catch {
    return []
  }
  const out: string[] = []
  for (const name of entries) {
    if (name.startsWith('.')) continue
    const candidate = join(parentAbs, name)
    try {
      const s = await stat(candidate)
      if (s.isDirectory()) out.push(candidate)
    } catch {
      // Race with deletion / dangling symlink — skip.
    }
  }
  return out
}

interface PackageManifest {
  name?: string
  private?: boolean
  customElements?: string
}

/** Walk `workspaces.packages` from the root manifest and return every
 *  package directory that has a parsable `package.json` with a `name`. */
export async function discoverWorkspacePackages(
  repoRoot: string,
): Promise<WorkspacePackage[]> {
  const globs = await workspaceGlobs(repoRoot)
  const seen = new Set<string>()
  const out: WorkspacePackage[] = []
  for (const g of globs) {
    const dirs = await expandGlob(repoRoot, g)
    for (const dir of dirs) {
      if (seen.has(dir)) continue
      seen.add(dir)
      const manifest = await readJson<PackageManifest>(join(dir, 'package.json'))
      if (!manifest?.name) continue
      out.push({
        name: manifest.name,
        root: dir,
        private: manifest.private === true,
        customElements: manifest.customElements,
      })
    }
  }
  out.sort((a, b) => a.name.localeCompare(b.name))
  return out
}

/** Convert an npm name to a daemon pack-id. `@ship/charts` → `charts`;
 *  unscoped `widgets` → `widgets`. */
export function packIdForName(npmName: string): string {
  if (npmName.startsWith('@')) {
    const slash = npmName.indexOf('/')
    if (slash > 0) return npmName.slice(slash + 1)
  }
  return npmName
}
