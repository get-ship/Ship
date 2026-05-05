import { promises as fs } from 'node:fs'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import { loadRouteRegistry, matchRoute, parseManifest, ManifestError, RegistryError } from './index'

let workdir: string

beforeEach(async () => {
  workdir = await mkdtemp(join(tmpdir(), 'ship-router-'))
})

afterEach(async () => {
  await rm(workdir, { recursive: true, force: true })
})

async function writeView(
  name: string,
  opts: {
    manifest?: string
    page?: string | null
    privateTags?: string[]
  } = {},
): Promise<string> {
  const dir = join(workdir, name)
  await fs.mkdir(dir, { recursive: true })
  if (opts.manifest !== undefined) {
    await fs.writeFile(join(dir, 'manifest.jsonc'), opts.manifest)
  }
  if (opts.page !== null) {
    await fs.writeFile(join(dir, 'page.sui'), opts.page ?? '<div>hi</div>')
  }
  for (const tag of opts.privateTags ?? []) {
    const tagDir = join(dir, '_components', tag)
    await fs.mkdir(tagDir, { recursive: true })
    await fs.writeFile(join(tagDir, `${tag}.sui`), '<span></span>')
  }
  return dir
}

describe('parseManifest', () => {
  it('parses a minimal manifest with comments and trailing commas', () => {
    const m = parseManifest(`
      // a route metadata file
      {
        "name": "home", /* required */
        "path": "/",
      }
    `)
    expect(m.name).toBe('home')
    expect(m.path).toBe('/')
  })

  it('parses optional fields', () => {
    const m = parseManifest(`{
      "name": "agents",
      "path": "/agents/[id]",
      "title": "Agent Detail",
      "auth": "user",
      "rail": { "label": "Agents", "icon": "users" },
      "preload": ["ship.agents.summaries()"],
      "events": ["config.changed"]
    }`)
    expect(m.path).toBe('/agents/[id]')
    expect(m.auth).toBe('user')
    expect(m.rail).toEqual({ label: 'Agents', icon: 'users' })
    expect(m.preload).toEqual(['ship.agents.summaries()'])
  })

  it('rejects path without leading slash', () => {
    expect(() => parseManifest('{ "name": "x", "path": "x" }')).toThrow(ManifestError)
  })

  it('rejects unknown auth values', () => {
    expect(() => parseManifest('{ "name": "x", "path": "/", "auth": "owner" }')).toThrow(
      ManifestError,
    )
  })

  it('rejects missing required fields', () => {
    expect(() => parseManifest('{ "name": "x" }')).toThrow(ManifestError)
    expect(() => parseManifest('{ "path": "/" }')).toThrow(ManifestError)
  })
})

describe('loadRouteRegistry', () => {
  it('walks directories with manifest.jsonc and skips legacy directories', async () => {
    await writeView('home', {
      manifest: `{ "name": "home", "path": "/" }`,
    })
    // Legacy view: no manifest → skipped silently.
    const legacyDir = join(workdir, 'counter')
    await fs.mkdir(legacyDir, { recursive: true })
    await fs.writeFile(join(legacyDir, 'view.sui'), '<div>legacy</div>')

    const reg = await loadRouteRegistry(workdir)
    expect(reg.routes).toHaveLength(1)
    expect(reg.routes[0]!.name).toBe('home')
  })

  it('discovers view-private components under _components/', async () => {
    await writeView('home', {
      manifest: `{ "name": "home", "path": "/" }`,
      privateTags: ['home-hero', 'home-promo'],
    })
    const reg = await loadRouteRegistry(workdir)
    expect(reg.routes[0]!.privateComponents).toEqual(['home-hero', 'home-promo'])
  })

  it('rejects view-private components that do not prefix the view name', async () => {
    await writeView('home', {
      manifest: `{ "name": "home", "path": "/" }`,
      privateTags: ['hero'],
    })
    await expect(loadRouteRegistry(workdir)).rejects.toThrow(/prefixed with view name/)
  })

  it('rejects manifests whose name disagrees with the directory', async () => {
    await writeView('home', {
      manifest: `{ "name": "homeBROKEN", "path": "/" }`,
    })
    await expect(loadRouteRegistry(workdir)).rejects.toThrow(RegistryError)
  })

  it('rejects views missing page.sui', async () => {
    await writeView('home', {
      manifest: `{ "name": "home", "path": "/" }`,
      page: null,
    })
    await expect(loadRouteRegistry(workdir)).rejects.toThrow(/missing required page\.sui/)
  })

  it('rejects two views with the same path', async () => {
    await writeView('a', { manifest: `{ "name": "a", "path": "/dup" }` })
    await writeView('b', { manifest: `{ "name": "b", "path": "/dup" }` })
    await expect(loadRouteRegistry(workdir)).rejects.toThrow(/duplicate route path/)
  })
})

describe('matchRoute', () => {
  it('happy path: walks views/, registers manifests, resolves a URL', async () => {
    await writeView('home', {
      manifest: `{ "name": "home", "path": "/", "title": "Home" }`,
    })
    await writeView('agents', {
      manifest: `{ "name": "agents", "path": "/agents/[id]" }`,
    })

    const reg = await loadRouteRegistry(workdir)

    const root = matchRoute(reg, '/')
    expect(root).not.toBeNull()
    expect(root!.entry.name).toBe('home')
    expect(root!.entry.manifest.title).toBe('Home')
    expect(root!.params).toEqual({})

    const agent = matchRoute(reg, '/agents/abc-123')
    expect(agent).not.toBeNull()
    expect(agent!.entry.name).toBe('agents')
    expect(agent!.params).toEqual({ id: 'abc-123' })
  })

  it('returns null for unmatched URLs (404)', async () => {
    await writeView('home', { manifest: `{ "name": "home", "path": "/" }` })
    const reg = await loadRouteRegistry(workdir)
    expect(matchRoute(reg, '/nope')).toBeNull()
    expect(matchRoute(reg, '/a/b/c')).toBeNull()
  })

  it('prefers literal segments over [param] when both match', async () => {
    await writeView('agents', {
      manifest: `{ "name": "agents", "path": "/agents/[id]" }`,
    })
    await writeView('agents-list', {
      manifest: `{ "name": "agents-list", "path": "/agents/list" }`,
    })

    const reg = await loadRouteRegistry(workdir)
    const m = matchRoute(reg, '/agents/list')
    expect(m).not.toBeNull()
    expect(m!.entry.name).toBe('agents-list')
  })

  it('decodes percent-encoded params', async () => {
    await writeView('items', {
      manifest: `{ "name": "items", "path": "/items/[slug]" }`,
    })
    const reg = await loadRouteRegistry(workdir)
    const m = matchRoute(reg, '/items/hello%20world')
    expect(m!.params['slug']).toBe('hello world')
  })
})

describe('real views/home view', () => {
  it('loads the in-tree views/home/ directory and resolves "/"', async () => {
    // The repository ships views/home/ as the canonical proof of the new
    // directory convention. Walk it and resolve "/" to confirm the router
    // sees what the ADR describes — page.sui + manifest.jsonc + _components/.
    // src/router → repo root is four levels up.
    const viewsRoot = join(__dirname, '..', '..', '..', '..', 'views')
    const reg = await loadRouteRegistry(viewsRoot)
    const home = reg.routes.find((r) => r.name === 'home')
    expect(home, 'views/home/ should be discovered').toBeDefined()
    expect(home!.manifest.path).toBe('/')
    expect(home!.privateComponents).toContain('home-hero')

    const m = matchRoute(reg, '/')
    expect(m).not.toBeNull()
    expect(m!.entry.name).toBe('home')
  })
})
