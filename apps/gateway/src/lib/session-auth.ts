import { getAuth } from './auth.js'

export interface SessionUser {
  sub: string
  org: string
}

export async function requireSession(
  request: Request,
  env: Env,
): Promise<SessionUser | Response> {
  try {
    const auth = await getAuth(env)
    const sessionResponse = await auth.api.getSession({
      headers: request.headers,
    })

    if (sessionResponse?.user?.id) {
      return {
        sub: sessionResponse.user.id,
        org: sessionResponse.user.id,
      }
    }
  } catch {
    // Session check failed
  }

  return Response.json(
    { error: 'Authentication required' },
    { status: 401 },
  )
}

export async function optionalSession(
  request: Request,
  env: Env,
): Promise<SessionUser | null> {
  try {
    const auth = await getAuth(env)
    const sessionResponse = await auth.api.getSession({
      headers: request.headers,
    })

    if (sessionResponse?.user?.id) {
      return {
        sub: sessionResponse.user.id,
        org: sessionResponse.user.id,
      }
    }
  } catch {
    // Not authenticated
  }

  return null
}
