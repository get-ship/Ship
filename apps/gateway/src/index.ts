import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { errorHandler } from './middleware/on-error.js';
import { requireProjectAuth } from './middleware/project-auth.js';
import { syncRoutes, projectRoutes, jobRoutes } from './routes/index.js';
import { authRoutes } from './routes/auth.js';
import { githubRoutes } from './routes/github.js';
import { registryRoutes } from './routes/registry.js';
import { opsRoutes } from './routes/ops.js';
import { waitlistRoutes } from './routes/waitlist.js';
import { connectAuthHandler } from './lib/connect-router.js';

const CORS_ORIGINS = [
  'http://localhost:3000',
  'http://localhost:4321', // marketing site (Astro dev)
  'https://staging.getship.dev',
  'https://getship.dev',
];

// All non-operational routes live under /v1/*. The host is already api.getship.dev,
// so an /api/ prefix would be redundant. /healthz and /readyz stay unversioned
// per ops convention.

const app = new Hono<{ Bindings: Env; Variables: { projectId: string; userId: string } }>()
  .onError(errorHandler)
  .use(
    '/v1/*',
    cors({
      origin: CORS_ORIGINS,
      allowHeaders: ['Content-Type', 'Authorization', 'x-project-id'],
      allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
      credentials: true,
    }),
  )
  .use('/v1/sync/*', requireProjectAuth)
  .use('/v1/projects/*', requireProjectAuth)
  .use('/v1/jobs/*', requireProjectAuth)
  .route('/v1/sync', syncRoutes)
  .route('/v1/projects', projectRoutes)
  .route('/v1/jobs', jobRoutes)
  // Connect-RPC AuthService — typed RPCs for token exchange, refresh,
  // daemon-attest, and identity (replaces the equivalent REST handlers).
  // Convention: `/<package>.<Service>/<Method>`. Versioning lives in the
  // proto package, hence no /v1/ prefix on these paths.
  .all('/ship.gateway.v1.AuthService/*', (c) => connectAuthHandler(c.req.raw, c.env))
  // Auth, GitHub, registry routes (no x-project-id middleware) — paths declared
  // inside the route modules using absolute /v1/... prefixes. authRoutes here
  // still owns Better Auth catch-all + CLI redirects + JWKS — non-RPC shapes.
  .route('/', authRoutes)
  .route('/', githubRoutes)
  .route('/', registryRoutes)
  // Always-on: ops + waitlist (no feature flag, no auth)
  .route('/', opsRoutes)
  .route('/', waitlistRoutes);

export default app;
