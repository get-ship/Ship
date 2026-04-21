import { getAuth } from '../auth.js'

export async function getGitHubToken(request: Request, env: Env): Promise<string | null> {
  try {
    const auth = await getAuth(env)
    const result = await auth.api.getAccessToken({
      body: { providerId: 'github' },
      headers: request.headers,
    })
    return result?.accessToken ?? null
  } catch {
    return null
  }
}
