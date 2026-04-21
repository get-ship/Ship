import { getAuthDb } from './d1.js'

async function makeAuth(env: Env) {
  const { betterAuth } = await import('better-auth')

  const d1 = getAuthDb(env)
  let database: ReturnType<typeof import('better-auth/adapters/drizzle').drizzleAdapter> | undefined

  if (d1) {
    const [{ drizzle }, { drizzleAdapter }, schema] = await Promise.all([
      import('drizzle-orm/d1'),
      import('better-auth/adapters/drizzle'),
      import('../data/schema/auth.js'),
    ])
    const db = drizzle(d1, { schema })
    database = drizzleAdapter(db, { provider: 'sqlite', schema })
  }

  return betterAuth({
    ...(database ? { database } : {}),
    socialProviders: {
      github: {
        clientId: env.GITHUB_CLIENT_ID ?? '',
        clientSecret: env.GITHUB_CLIENT_SECRET ?? '',
        scope: ['repo', 'user:email'],
      },
    },
  })
}

type Auth = Awaited<ReturnType<typeof makeAuth>>
let _auth: Auth | undefined
let _authPromise: Promise<Auth> | undefined

export function getAuth(env: Env): Promise<Auth> {
  if (_auth) return Promise.resolve(_auth)
  if (!_authPromise) {
    _authPromise = makeAuth(env).then((a) => {
      _auth = a
      return a
    })
  }
  return _authPromise
}
