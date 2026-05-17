import { Hono } from 'hono'
import { requireSession } from '../lib/session-auth.js'
import { getRegistryDb } from '../lib/d1.js'
import { getUser, listRepos } from '../lib/github/github-app.js'
import { getGitHubToken } from '../lib/github/github-token.js'
import { extractLibrary } from '../lib/github/github-import.js'
import { fetchRepoFiles } from '../lib/github/fetch-repo-files.js'

const github = new Hono<{ Bindings: Env }>()

github.get('/v1/github/repos', async (c) => {
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

github.get('/v1/github/installations', async (c) => {
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

github.get('/v1/github/repos-config', async (c) => {
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

github.post('/v1/github/disconnect', async (c) => {
  const session = await requireSession(c.req.raw, c.env)
  if (session instanceof Response) return session
  return c.json({ ok: true, action: 'reauthenticate' })
})

export { github as githubRoutes }
