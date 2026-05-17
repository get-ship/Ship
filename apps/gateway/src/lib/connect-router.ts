/**
 * Connect-RPC fetch adapter for Cloudflare Workers / Hono.
 *
 * `@connectrpc/connect` ships `createConnectRouter` (which produces typed
 * `UniversalHandler`s per registered method) and convenience fetch wrappers,
 * but the wrappers don't expose `contextValues` — we need that to inject
 * the per-request Cloudflare `Env` bindings into handlers. So this adapter
 * goes one level deeper: convert fetch Request → UniversalServerRequest,
 * attach contextValues, invoke the handler, convert back.
 *
 * The cost over `createFetchHandler` is a single hashmap lookup per request.
 */
import { createConnectRouter, createContextValues } from '@connectrpc/connect'
import {
  universalServerRequestFromFetch,
  universalServerResponseToFetch,
} from '@connectrpc/connect/protocol'
import type { UniversalHandler } from '@connectrpc/connect/protocol'
import { AuthService } from '../proto/gen/ship/gateway/v1/auth_pb.js'
import { authServiceImpl } from '../services/auth-impl.js'
import { envContextKey } from './connect-context.js'

const router = createConnectRouter()
router.service(AuthService, authServiceImpl)

const handlersByPath: ReadonlyMap<string, UniversalHandler> = new Map(
  router.handlers.map((h) => [h.requestPath, h]),
)

export async function connectAuthHandler(req: Request, env: Env): Promise<Response> {
  const path = new URL(req.url).pathname
  const handler = handlersByPath.get(path)
  if (!handler) return new Response('connect: method not found', { status: 404 })

  const universalReq = universalServerRequestFromFetch(req, {})
  universalReq.contextValues = createContextValues().set(envContextKey, env)
  const universalRes = await handler(universalReq)
  return universalServerResponseToFetch(universalRes)
}
