export function getAuthDb(env: Env): D1Database | null {
  return env.AUTH_DB ?? null
}

export function getRegistryDb(env: Env): D1Database | null {
  return env.REGISTRY_DB ?? null
}

export function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}
