import { Hono } from 'hono'
import { requireSession } from '../lib/session-auth.js'
import { getRegistryDb } from '../lib/d1.js'
import { getUser, listRepos, createConfigPr } from '../lib/github/github-app.js'
import { getGitHubToken } from '../lib/github/github-token.js'
import { parseGithubUrl, extractLibrary } from '../lib/github/github-import.js'
import { fetchRepoFiles } from '../lib/github/fetch-repo-files.js'
import { libraryToShipFiles } from '../lib/ship-config.js'

const github = new Hono<{ Bindings: Env }>()

github.get('/api/github/repos', async (c) => {
  const token = await getGitHubToken(c.req.raw, c.env)
  if (!token) return c.json({ error: 'Not authenticated' }, 401)

  const page = Number(c.req.query('page') ?? '1')
  try {
    const [user, repos] = await Promise.all([getUser(token), listRepos(token, page)])
    return c.json({
      user: { login: user.login, avatar_url: user.avatar_url },
      repos: repos.map((r) => ({
        full_name: r.full_name,
        name: r.name,
        owner: r.owner.login,
        private: r.private,
        default_branch: r.default_branch,
        description: r.description,
      })),
    })
  } catch (err) {
    return c.json({ error: err instanceof Error ? err.message : 'Failed to fetch repos' }, 502)
  }
})

github.get('/api/github/installations', async (c) => {
  const session = await requireSession(c.req.raw, c.env)
  if (session instanceof Response) return session

  const db = getRegistryDb(c.env)
  if (!db) return c.json({ installations: [] })

  try {
    const rows = await db
      .prepare(`SELECT installation_id, account_login, account_type, repos_json FROM github_installations ORDER BY updated_at DESC`)
      .all<{ installation_id: number; account_login: string; account_type: string; repos_json: string }>()

    const installations = (rows.results ?? []).map((row) => ({
      installation_id: row.installation_id,
      account_login: row.account_login,
      account_type: row.account_type,
      repos: JSON.parse(row.repos_json) as unknown[],
    }))
    return c.json({ installations })
  } catch {
    return c.json({ installations: [] })
  }
})

github.get('/api/github/repos-config', async (c) => {
  const token = await getGitHubToken(c.req.raw, c.env)
  if (!token) return c.json({ error: 'Not authenticated' }, 401)

  const owner = c.req.query('owner')
  const repo = c.req.query('repo')
  if (!owner || !repo) return c.json({ error: 'Missing owner or repo query parameter' }, 400)

  try {
    const files = await fetchRepoFiles(owner, repo, token)
    if (files === 'not_found') return c.json({ error: 'Repository not found or no access' }, 404)
    const library = extractLibrary(files)
    if (!library) return c.json({ error: 'No agent config found in this repository', files_checked: Object.keys(files) }, 422)
    return c.json(library)
  } catch (err) {
    return c.json({ error: err instanceof Error ? err.message : 'Failed to fetch repo config' }, 502)
  }
})

github.post('/api/github/import', async (c) => {
  let body: unknown
  try { body = await c.req.json() } catch { return c.json({ error: 'Invalid JSON body' }, 400) }

  if (!body || typeof body !== 'object' || !('url' in body) || typeof (body as Record<string, unknown>).url !== 'string') {
    return c.json({ error: 'Missing or invalid url field' }, 400)
  }

  const url = (body as { url: string }).url
  const parsed = parseGithubUrl(url)
  if (!parsed) return c.json({ error: 'Malformed GitHub URL' }, 400)

  const result = await fetchRepoFiles(parsed.owner, parsed.repo)
  if (result === 'not_found') return c.json({ error: 'Repository not found' }, 404)

  const library = extractLibrary(result)
  if (!library) return c.json({ error: 'No extractable agent config found in this repository' }, 422)

  return c.json(library)
})

github.post('/api/github/create-pr', async (c) => {
  const token = await getGitHubToken(c.req.raw, c.env)
  if (!token) return c.json({ error: 'Not authenticated' }, 401)

  let body: { owner: string; repo: string; default_branch: string; mode_name?: string }
  try { body = await c.req.json() } catch { return c.json({ error: 'Invalid JSON' }, 400) }

  const { owner, repo, default_branch, mode_name } = body
  if (!owner || !repo || !default_branch) return c.json({ error: 'Missing owner, repo, or default_branch' }, 400)

  try {
    const repoFiles = await fetchRepoFiles(owner, repo, token)
    if (repoFiles === 'not_found') return c.json({ error: 'Repository not found or no access' }, 404)

    const library = extractLibrary(repoFiles)
    const shipFiles = libraryToShipFiles(
      library ?? { modes: [], active_agent: null, mcp_servers: [], skills: [], rules: [], agent_profiles: [], claude_team_agents: [], env: {}, available_models: [], provider_defaults: {} },
      mode_name ?? 'default',
    )
    const pr = await createConfigPr(token, owner, repo, default_branch, shipFiles)
    return c.json({ html_url: pr.html_url, number: pr.number })
  } catch (err) {
    return c.json({ error: err instanceof Error ? err.message : 'PR creation failed' }, 502)
  }
})

github.post('/api/github/disconnect', async (c) => {
  const session = await requireSession(c.req.raw, c.env)
  if (session instanceof Response) return session
  return c.json({ ok: true, action: 'reauthenticate' })
})

export { github as githubRoutes }
