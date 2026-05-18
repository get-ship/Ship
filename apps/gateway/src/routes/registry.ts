import { Hono } from 'hono'
import { z } from 'zod'
import { createRegistryRepositories } from '../data/registry.repository.js'
import { getRegistryDb, nanoid } from '../lib/d1.js'
import { requireSession } from '../lib/session-auth.js'
import { checkRateLimit, rateLimitResponse } from '../lib/rate-limit.js'
import { getUser } from '../lib/github/github-app.js'
import { getGitHubToken } from '../lib/github/github-token.js'
import {
  parseGithubUrl,
  fetchFileFromGitHub,
  fetchShipManifest,
  parseShipManifest,
  resolveGitHubRef,
} from '../lib/registry/registry-github.js'
import { scanSkillContent } from '../lib/skill-scan.js'
import { computeContentHash } from '../lib/content-hash.js'
import { isNewerVersion } from '../lib/semver.js'
import { verifySignature, getPayloadAgeMs } from '../lib/webhook-verify.js'

const registry = new Hono<{ Bindings: Env }>()

const TOML_MAX_BYTES = 102400
const SKILL_MAX_BYTES = 51200
const SKILL_ID_RE = /^[a-z0-9][a-z0-9\-_]{0,62}[a-z0-9]$/i
const WEBHOOK_MAX_AGE_MS = 5 * 60 * 1000
const MAX_LIMIT = 100
const DEFAULT_LIMIT = 20
const VALID_SCOPES = new Set(['official', 'unofficial', 'community'])
const VALID_SORTS = new Set(['installs', 'recent', 'name'])

// ── Search ────────────────────────────────────────────────────────────────────

registry.get('/v1/registry/search', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_SEARCH', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const rawQuery = c.req.query('q') ?? null
  if (rawQuery !== null && rawQuery.length > 200) return c.json({ error: 'Query too long (max 200 characters)' }, 400)

  const scope = c.req.query('scope') || undefined
  const sort = c.req.query('sort') || undefined

  if (scope && !VALID_SCOPES.has(scope)) return c.json({ error: `Invalid scope: ${scope}` }, 400)
  if (sort && !VALID_SORTS.has(sort)) return c.json({ error: `Invalid sort: ${sort}` }, 400)

  const pageParam = parseInt(c.req.query('page') || '1', 10)
  const limitParam = parseInt(c.req.query('limit') || String(DEFAULT_LIMIT), 10)
  const page = Number.isFinite(pageParam) && pageParam > 0 ? pageParam : 1
  const limit = Number.isFinite(limitParam) && limitParam > 0 ? Math.min(limitParam, MAX_LIMIT) : DEFAULT_LIMIT

  const repos = createRegistryRepositories(d1)
  const sortOrder = (sort ?? 'installs') as 'installs' | 'recent' | 'name'
  const result = await repos.searchPackages(rawQuery || undefined, scope, page, limit, sortOrder)

  const cappedTotal = Math.min(result.total, 1000)
  return c.json(
    {
      packages: result.packages.map((p) => ({
        id: p.id, path: p.path, name: p.name, description: p.description,
        scope: p.scope, latestVersion: p.latestVersion, updatedAt: p.updatedAt,
        installs: p.installs, stars: p.stars, deprecatedBy: p.deprecatedBy,
        repoUrl: p.repoUrl, claimedBy: p.claimedBy,
      })),
      total: cappedTotal,
      ...(result.total > 1000 ? { hasMore: true } : {}),
      page: result.page,
    },
    200,
    { 'Cache-Control': 'public, max-age=60', 'Vary': 'Accept' },
  )
})

// ── Duplicates ────────────────────────────────────────────────────────────────

registry.get('/v1/registry/duplicates', async (c) => {
  const hash = c.req.query('hash')
  if (!hash) return c.json({ error: 'Missing hash query parameter' }, 400)
  if (!/^sha256:[a-f0-9]{64}$/.test(hash)) return c.json({ error: 'Invalid hash format. Expected sha256:<64 hex chars>' }, 400)

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const { results } = await d1.prepare(
    `SELECT p.path as package_path, p.name as package_name, ps.name as skill_name
     FROM package_skills ps JOIN packages p ON ps.package_id = p.id
     WHERE ps.content_hash = ?`,
  ).bind(hash).all<{ package_path: string; package_name: string; skill_name: string }>()

  return c.json({ hash, packages: (results ?? []).map((r) => ({ path: r.package_path, name: r.package_name, skill_name: r.skill_name })) })
})

// ── Package detail ────────────────────────────────────────────────────────────

registry.get('/v1/registry/:path{.+}', async (c) => {
  // Disambiguate from /star, /install, /deprecate below by checking suffix
  const rawPath = c.req.param('path')
  if (rawPath.endsWith('/star') || rawPath.endsWith('/install') || rawPath.endsWith('/deprecate')) {
    return c.notFound()
  }

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const packagePath = decodeURIComponent(rawPath)
  const repos = createRegistryRepositories(d1)
  const pkg = await repos.getPackage(packagePath)
  if (!pkg) return c.json({ error: `Package not found: ${packagePath}` }, 404)

  const [versions, skills] = await Promise.all([repos.getPackageVersions(pkg.id), repos.getPackageSkills(pkg.id)])
  return c.json({
    package: pkg,
    versions: versions.map((v) => ({
      id: v.id, version: v.version, gitTag: v.gitTag, commitSha: v.commitSha,
      skills: v.skillsJson ? JSON.parse(v.skillsJson) : [],
      agents: v.agentsJson ? JSON.parse(v.agentsJson) : [],
      indexedAt: v.indexedAt,
    })),
    skills: skills.map((s) => ({ id: s.id, skillId: s.skillId, name: s.name, description: s.description, contentHash: s.contentHash })),
  })
})

// ── Star ──────────────────────────────────────────────────────────────────────

registry.post('/v1/registry/:path{.+}/star', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_STAR', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const auth = await requireSession(c.req.raw, c.env)
  if (auth instanceof Response) return auth

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const packagePath = decodeURIComponent(c.req.param('path'))
  const repos = createRegistryRepositories(d1)
  const pkg = await repos.getPackage(packagePath)
  if (!pkg) return c.json({ error: `Package not found: ${packagePath}` }, 404)

  const stars = await repos.incrementStars(pkg.id)
  return c.json({ stars, starred: true })
})

// ── Install ───────────────────────────────────────────────────────────────────

registry.post('/v1/registry/:path{.+}/install', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_INSTALL', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const packagePath = decodeURIComponent(c.req.param('path'))
  const repos = createRegistryRepositories(d1)
  const pkg = await repos.getPackage(packagePath)
  if (!pkg) return c.json({ error: `Package not found: ${packagePath}` }, 404)

  const installs = await repos.incrementInstalls(pkg.id)
  return c.json({ installs })
})

// ── Deprecate ─────────────────────────────────────────────────────────────────

registry.post('/v1/registry/:path{.+}/deprecate', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_CLAIM', 3600, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const auth = await requireSession(c.req.raw, c.env)
  if (auth instanceof Response) return auth

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  let body: { deprecated_by: string }
  try { body = await c.req.json() } catch { return c.json({ error: 'Invalid JSON body' }, 400) }
  if (!body.deprecated_by || typeof body.deprecated_by !== 'string') return c.json({ error: 'Missing deprecated_by field' }, 400)

  const packagePath = decodeURIComponent(c.req.param('path'))
  const repos = createRegistryRepositories(d1)
  const pkg = await repos.getPackage(packagePath)
  if (!pkg) return c.json({ error: `Package not found: ${packagePath}` }, 404)
  if (pkg.claimedBy !== auth.sub) return c.json({ error: 'Only the package owner can deprecate this package' }, 403)

  await repos.deprecatePackage(pkg.id, body.deprecated_by)
  return c.json({ ok: true, deprecated_by: body.deprecated_by })
})

// ── Claim ─────────────────────────────────────────────────────────────────────

registry.post('/v1/registry/claim', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_CLAIM', 3600, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const auth = await requireSession(c.req.raw, c.env)
  if (auth instanceof Response) return auth

  const ghToken = await getGitHubToken(c.req.raw, c.env)
  if (!ghToken) return c.json({ error: 'GitHub authentication required. Connect your GitHub account first.' }, 401)

  let body: { package_path: string }
  try { body = await c.req.json() } catch { return c.json({ error: 'Invalid JSON body' }, 400) }
  if (!body.package_path || typeof body.package_path !== 'string') return c.json({ error: 'Missing package_path' }, 400)

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const repos = createRegistryRepositories(d1)
  const pkg = await repos.getPackage(body.package_path)
  if (!pkg) return c.json({ error: 'Package not found' }, 404)
  if (pkg.claimedBy === auth.sub) return c.json({ claimed: true, package_path: pkg.path })
  if (pkg.claimedBy) return c.json({ error: 'Package already claimed by another user' }, 409)

  let repoOwner: string, repoName: string
  try {
    const parsed = new URL(pkg.repoUrl)
    if (parsed.hostname !== 'github.com') throw new Error('Not GitHub')
    const parts = parsed.pathname.replace(/^\//, '').split('/')
    repoOwner = parts[0]
    repoName = parts[1].replace(/\.git$/, '')
  } catch {
    return c.json({ error: 'Package has no valid GitHub repo URL' }, 422)
  }

  const ghUser = await getUser(ghToken)
  const permRes = await fetch(`https://api.github.com/repos/${repoOwner}/${repoName}/collaborators/${ghUser.login}/permission`, {
    headers: { Authorization: `Bearer ${ghToken}`, Accept: 'application/vnd.github.v3+json', 'User-Agent': 'ship-studio/1.0' },
  })
  let permitted = false
  if (permRes.ok) {
    const permData = (await permRes.json()) as { permission: string; role_name: string }
    permitted = permData.permission === 'admin' || permData.permission === 'write' || permData.role_name === 'admin'
  }

  if (!permitted) return c.json({ error: `You must be an admin or maintainer of ${repoOwner}/${repoName} to claim this package` }, 403)

  const newScope = pkg.scope === 'unofficial' ? 'community' : pkg.scope
  const claimed = await repos.claimPackage(pkg.path, auth.sub, newScope)
  if (!claimed) return c.json({ error: 'Package already claimed by another user' }, 409)

  return c.json({ claimed: true, package_path: pkg.path })
})

// ── Publish ───────────────────────────────────────────────────────────────────

const PublishInput = z.object({
  repo_url: z.string().min(1),
  tag: z.string().optional(),
})

registry.post('/v1/registry/publish', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_PUBLISH', 3600, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  let body: unknown
  try { body = await c.req.json() } catch { return c.json({ error: 'Invalid JSON body' }, 400) }

  const parsed = PublishInput.safeParse(body)
  if (!parsed.success) {
    const msg = parsed.error.issues.map((i) => i.message).join('; ')
    return c.json({ error: msg }, 400)
  }

  const { repo_url, tag } = parsed.data
  const ghParsed = parseGithubUrl(repo_url)
  if (!ghParsed) return c.json({ error: 'Malformed GitHub URL — expected https://github.com/owner/repo' }, 400)

  const d1 = getRegistryDb(c.env)
  if (!d1) return c.json({ error: 'Database unavailable' }, 503)

  const sessionResult = await requireSession(c.req.raw, c.env)
  if (sessionResult instanceof Response) return sessionResult
  const session = sessionResult
  const ref = tag || 'HEAD'

  const manifest = await fetchShipManifest(ghParsed.owner, ghParsed.repo, ref)
  if (!manifest) return c.json({ error: 'No .ship/ship.jsonc found in this repository.' }, 422)

  if (new TextEncoder().encode(manifest.content).length > TOML_MAX_BYTES) return c.json({ error: 'Ship manifest exceeds maximum size of 100KB' }, 422)

  const toml = parseShipManifest(manifest.content, manifest.format)
  if (!toml.module) return c.json({ error: '.ship/ship.jsonc missing [module] section with name and version' }, 422)

  const repos = createRegistryRepositories(d1)
  const packagePath = `github.com/${ghParsed.owner}/${ghParsed.repo}`

  const existing = await repos.getPackage(packagePath)
  if (existing?.claimedBy && existing.claimedBy !== session.sub) return c.json({ error: 'Package already claimed by another user' }, 409)

  if (existing) {
    const existingVersions = await repos.getPackageVersions(existing.id)
    const duplicate = existingVersions.find((v) => v.gitTag === ref)
    if (duplicate) {
      const skills = await repos.getPackageSkills(existing.id, duplicate.id)
      return c.json({ package_id: existing.id, version: duplicate.version, skills_indexed: skills.length })
    }
  }

  const now = Date.now()
  const incomingVersion = toml.module.version || null
  const shouldUpdateVersion = !existing?.latestVersion || !incomingVersion || isNewerVersion(incomingVersion, existing.latestVersion)
  const latestVersion = shouldUpdateVersion ? incomingVersion : existing?.latestVersion ?? null
  const pkg = await repos.upsertPackage({
    id: existing?.id || nanoid(),
    path: packagePath,
    scope: 'community',
    name: toml.module.name,
    description: toml.module.description || null,
    repoUrl: repo_url,
    defaultBranch: 'main',
    latestVersion,
    contentHash: null,
    sourceType: 'native',
    claimedBy: session.sub || existing?.claimedBy || null,
    deprecatedBy: existing?.deprecatedBy || null,
    stars: existing?.stars ?? 0,
    installs: existing?.installs ?? 0,
    indexedAt: existing?.indexedAt ?? now,
    updatedAt: now,
  })

  const commitSha = await resolveGitHubRef(ghParsed.owner, ghParsed.repo, ref) || ref
  const versionId = nanoid()
  const skillIds: string[] = toml.exports?.skills ?? []
  const agentNames: string[] = toml.exports?.agents ?? []

  const version = await repos.createPackageVersion({
    id: versionId,
    packageId: pkg.id,
    version: toml.module.version || '0.0.0',
    gitTag: tag || 'HEAD',
    commitSha,
    contentHash: null,
    skillsJson: JSON.stringify(skillIds),
    agentsJson: JSON.stringify(agentNames),
    indexedAt: now,
  })

  for (const skillId of skillIds) {
    if (!SKILL_ID_RE.test(skillId)) return c.json({ error: `Invalid skill ID: ${skillId}` }, 422)
  }

  let skillsIndexed = 0
  const skillHashes: string[] = []
  for (const skillId of skillIds) {
    const content = await fetchFileFromGitHub(ghParsed.owner, ghParsed.repo, `.ship/skills/${skillId}.md`, ref)
    if (!content) continue
    if (new TextEncoder().encode(content).length > SKILL_MAX_BYTES) continue

    const scan = scanSkillContent(content)
    if (!scan.safe) return c.json({ error: 'Skill content flagged by security scan', warnings: scan.warnings.map((w) => `[${skillId}] ${w}`) }, 400)

    const hash = await computeContentHash(content)
    skillHashes.push(hash)
    await repos.createPackageSkill({
      id: nanoid(),
      packageId: pkg.id,
      versionId: version.id,
      skillId,
      name: skillId,
      description: extractSkillDescription(content),
      contentHash: hash,
    })
    skillsIndexed++
  }

  if (skillHashes.length > 0) {
    const combinedHash = await computeContentHash(skillHashes.sort().join('\0'))
    await repos.updatePackageVersionHash(version.id, combinedHash)
  }

  return c.json({ package_id: pkg.id, version: version.version, skills_indexed: skillsIndexed })
})

// ── Webhook ───────────────────────────────────────────────────────────────────

registry.post('/v1/registry/webhook', async (c) => {
  const secret = c.env.GITHUB_WEBHOOK_SECRET
  if (!secret) return c.json({ error: 'Webhook secret not configured' }, 500)

  const signature = c.req.header('x-hub-signature-256')
  const body = await c.req.text()
  if (!signature || !(await verifySignature(secret, body, signature))) return c.json({ error: 'Invalid webhook signature' }, 401)

  const event = c.req.header('x-github-event')
  const payload = JSON.parse(body) as Record<string, unknown>

  const payloadAge = getPayloadAgeMs(payload)
  if (payloadAge !== null && payloadAge > WEBHOOK_MAX_AGE_MS) return c.json({ error: 'Webhook payload too old' }, 400)

  if (event === 'create' && payload.ref_type === 'tag') return handleTagCreate(payload, c.env)
  if (event === 'installation' || event === 'installation_repositories') return handleInstallation(payload, c.env)

  return c.json({ status: 'ignored' })
})

async function handleTagCreate(payload: Record<string, unknown>, env: Env): Promise<Response> {
  const repo = payload.repository as Record<string, unknown> | undefined
  if (!repo) return Response.json({ error: 'Missing repository in payload' }, { status: 400 })

  const fullName = repo.full_name as string
  const tag = payload.ref as string
  const defaultBranch = (repo.default_branch as string) || 'main'
  const repoUrl = `https://github.com/${fullName}`
  const [owner, repoName] = fullName.split('/')

  const manifest = await fetchShipManifest(owner, repoName, tag)
  if (!manifest) return Response.json({ status: 'skipped', reason: 'no .ship/ship.jsonc at tag' })

  const toml = parseShipManifest(manifest.content, manifest.format)
  if (!toml.module?.name) return Response.json({ status: 'skipped', reason: 'ship.jsonc missing module name' })

  const d1 = getRegistryDb(env)
  if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

  const repos = createRegistryRepositories(d1)
  const packagePath = `github.com/${fullName}`
  const now = Date.now()

  const existing = await repos.getPackage(packagePath)
  const incomingVersion = toml.module.version || tag
  const shouldUpdateVersion = !existing?.latestVersion || isNewerVersion(incomingVersion, existing.latestVersion)
  const latestVersion = shouldUpdateVersion ? incomingVersion : existing?.latestVersion ?? null

  const pkg = await repos.upsertPackage({
    id: existing?.id || nanoid(),
    path: packagePath,
    scope: existing?.scope || 'community',
    name: toml.module.name,
    description: toml.module.description || null,
    repoUrl,
    defaultBranch,
    latestVersion,
    contentHash: null,
    sourceType: 'native',
    claimedBy: existing?.claimedBy || null,
    deprecatedBy: existing?.deprecatedBy || null,
    stars: existing?.stars ?? 0,
    installs: existing?.installs ?? 0,
    indexedAt: existing?.indexedAt ?? now,
    updatedAt: now,
  })

  const skillIds = toml.exports?.skills || []
  const agentNames = toml.exports?.agents || []
  const commitSha = await resolveGitHubRef(owner, repoName, tag) || tag

  try {
    await repos.createPackageVersion({
      id: nanoid(),
      packageId: pkg.id,
      version: toml.module.version || tag,
      gitTag: tag,
      commitSha,
      contentHash: null,
      skillsJson: JSON.stringify(skillIds),
      agentsJson: JSON.stringify(agentNames),
      indexedAt: now,
    })
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    if (msg.includes('UNIQUE constraint')) return Response.json({ status: 'duplicate', package_id: pkg.id, version: toml.module.version || tag })
    throw err
  }

  return Response.json({ status: 'indexed', package_id: pkg.id, version: toml.module.version || tag })
}

async function handleInstallation(payload: Record<string, unknown>, env: Env): Promise<Response> {
  const action = payload.action as string
  const installation = payload.installation as Record<string, unknown> | undefined
  if (!installation?.id) return Response.json({ status: 'acknowledged', action })

  const installationId = installation.id as number
  const account = installation.account as Record<string, unknown> | undefined
  const accountLogin = (account?.login as string) ?? 'unknown'
  const accountType = (account?.type as string) ?? 'User'

  const d1 = getRegistryDb(env)
  if (!d1) return Response.json({ status: 'acknowledged', action })

  const now = Date.now()

  if (action === 'created') {
    const rawRepos = payload.repositories
    const repoList = (Array.isArray(rawRepos) ? rawRepos as Array<Record<string, unknown>> : []).map((r) => ({
      id: r.id as number,
      full_name: r.full_name as string,
      private: r.private as boolean,
    }))
    await d1.prepare(
      `INSERT INTO github_installations (id, installation_id, account_login, account_type, repos_json, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(installation_id) DO UPDATE SET account_login = excluded.account_login, repos_json = excluded.repos_json, updated_at = excluded.updated_at`,
    ).bind(nanoid(), installationId, accountLogin, accountType, JSON.stringify(repoList), now, now).run()
    return Response.json({ status: 'stored', action, installation_id: installationId })
  }

  if (action === 'deleted') {
    await d1.prepare('DELETE FROM github_installations WHERE installation_id = ?').bind(installationId).run()
    return Response.json({ status: 'removed', action, installation_id: installationId })
  }

  if (action === 'added' || action === 'removed') {
    const existing = await d1.prepare('SELECT repos_json FROM github_installations WHERE installation_id = ?').bind(installationId).first<{ repos_json: string }>()
    if (existing) {
      let currentRepos = JSON.parse(existing.repos_json) as Array<{ id: number; full_name: string; private: boolean }>
      if (action === 'added') {
        const added = (payload.repositories_added as Array<Record<string, unknown>>) ?? []
        for (const r of added) {
          if (!currentRepos.some((cr) => cr.id === r.id)) currentRepos.push({ id: r.id as number, full_name: r.full_name as string, private: r.private as boolean })
        }
      } else {
        const removed = (payload.repositories_removed as Array<Record<string, unknown>>) ?? []
        const removedIds = new Set(removed.map((r) => r.id as number))
        currentRepos = currentRepos.filter((cr) => !removedIds.has(cr.id))
      }
      await d1.prepare('UPDATE github_installations SET repos_json = ?, updated_at = ? WHERE installation_id = ?').bind(JSON.stringify(currentRepos), now, installationId).run()
    }
    return Response.json({ status: 'updated', action, installation_id: installationId })
  }

  return Response.json({ status: 'acknowledged', action })
}

function extractSkillDescription(content: string): string | null {
  const match = content.match(/^---\n([\s\S]*?)\n---/)
  if (!match) return null
  const descLine = match[1].split('\n').find((l) => l.startsWith('description:'))
  if (!descLine) return null
  return descLine.replace('description:', '').trim().replace(/^["']|["']$/g, '')
}

export { registry as registryRoutes }
