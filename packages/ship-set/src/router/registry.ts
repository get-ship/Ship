import { promises as fs } from 'node:fs'
import { join } from 'node:path'
import { parseManifest, ManifestError } from './manifest'
import type { RouteEntry, RouteRegistry } from './types'

const MANIFEST_FILENAME = 'manifest.jsonc'
const PAGE_FILENAME = 'page.sui'
const PRIVATE_COMPONENTS_DIR = '_components'

export class RegistryError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'RegistryError'
  }
}

async function readMaybe(path: string): Promise<string | null> {
  try {
    return await fs.readFile(path, 'utf8')
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === 'ENOENT') return null
    throw e
  }
}

async function listDirs(root: string): Promise<string[]> {
  const entries = await fs.readdir(root, { withFileTypes: true })
  return entries
    .filter((e) => e.isDirectory() && !e.name.startsWith('.') && !e.name.startsWith('_'))
    .map((e) => e.name)
}

async function discoverPrivateComponents(viewDir: string, viewName: string): Promise<string[]> {
  const dir = join(viewDir, PRIVATE_COMPONENTS_DIR)
  let entries: import('node:fs').Dirent[]
  try {
    entries = await fs.readdir(dir, { withFileTypes: true })
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === 'ENOENT') return []
    throw e
  }
  const tags: string[] = []
  for (const e of entries) {
    if (!e.isDirectory()) continue
    if (e.name.startsWith('.')) continue
    const tag = e.name
    const suiPath = join(dir, tag, `${tag}.sui`)
    if ((await readMaybe(suiPath)) === null) {
      throw new RegistryError(
        `view-private component "${tag}" missing ${tag}.sui (looked at ${suiPath})`,
      )
    }
    if (!tag.startsWith(`${viewName}-`)) {
      throw new RegistryError(
        `view-private component "${tag}" must be prefixed with view name "${viewName}-" to avoid registry collisions`,
      )
    }
    tags.push(tag)
  }
  return tags.sort()
}

async function loadRoute(viewsDir: string, name: string): Promise<RouteEntry | null> {
  const dir = join(viewsDir, name)
  const manifestPath = join(dir, MANIFEST_FILENAME)
  const text = await readMaybe(manifestPath)
  if (text === null) {
    // No manifest → not a view in the new convention; skip silently. The
    // walker tolerates legacy view.sui directories alongside new-convention
    // ones during the migration window.
    return null
  }

  const manifest = parseManifest(text, manifestPath)
  if (manifest.name !== name) {
    throw new RegistryError(
      `${manifestPath}: manifest.name "${manifest.name}" does not match directory "${name}"`,
    )
  }

  const pagePath = join(dir, PAGE_FILENAME)
  if ((await readMaybe(pagePath)) === null) {
    throw new RegistryError(`${dir}: missing required ${PAGE_FILENAME}`)
  }

  const privateComponents = await discoverPrivateComponents(dir, name)
  return { manifest, dir, name, privateComponents }
}

/**
 * Walk a `views/` directory and build a route registry from each child's
 * `manifest.jsonc`. Children without a manifest are skipped silently to
 * support coexistence with legacy `view.sui` directories during migration.
 *
 * Throws `RegistryError` for malformed manifests, missing `page.sui`, or
 * `_components/` violations. Surface those — they're authoring bugs.
 */
export async function loadRouteRegistry(viewsDir: string): Promise<RouteRegistry> {
  const names = await listDirs(viewsDir)
  const routes: RouteEntry[] = []
  for (const name of names) {
    const route = await loadRoute(viewsDir, name)
    if (route !== null) routes.push(route)
  }

  // Reject duplicate paths early — two views answering the same URL is a
  // configuration bug the router can't resolve at runtime.
  const seen = new Map<string, string>()
  for (const r of routes) {
    const existing = seen.get(r.manifest.path)
    if (existing !== undefined) {
      throw new RegistryError(
        `duplicate route path "${r.manifest.path}": ${existing} and ${r.name}`,
      )
    }
    seen.set(r.manifest.path, r.name)
  }

  return { routes }
}

export { ManifestError }
