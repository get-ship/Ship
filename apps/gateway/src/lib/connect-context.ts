/**
 * Connect-RPC context keys shared between the fetch adapter (which
 * injects them) and the service implementations (which read them).
 *
 * Split out so the adapter and the impls don't import each other —
 * they both import this leaf module.
 */
import { createContextKey } from '@connectrpc/connect'

/**
 * Cloudflare Worker `Env` bindings, injected by the fetch adapter on
 * every Connect call. Read inside handlers via `ctx.values.get(envContextKey)`.
 */
export const envContextKey = createContextKey<Env | null>(null)
