import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { errorHandler } from './middleware/on-error.js';
import { syncRoutes, projectRoutes, jobRoutes } from './routes/index.js';
import { authRoutes } from './routes/auth.js';
import { githubRoutes } from './routes/github.js';
import { registryRoutes } from './routes/registry.js';
import { opsRoutes } from './routes/ops.js';
import { waitlistRoutes } from './routes/waitlist.js';

const CORS_ORIGINS = [
  'http://localhost:3000',
  'http://localhost:4321', // marketing site (Astro dev)
  'https://staging.getship.dev',
  'https://getship.dev',
];

// All non-operational routes live under /v1/*. The host is already api.getship.dev,
// so an /api/ prefix would be redundant. /healthz and /readyz stay unversioned
// per ops convention.
const requireProjectId = async (
  c: { req: { header: (n: string) => string | undefined }; set: (k: 'projectId', v: string) => void; json: (b: unknown, s: number) => Response },
  next: () => Promise<void>,
) => {
  const projectId = c.req.header('x-project-id');
  if (!projectId) return c.json({ error: 'Missing x-project-id header' }, 400);
  c.set('projectId', projectId);
  return next();
};

const app = new Hono<{ Bindings: Env; Variables: { projectId: string } }>()
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
  .use('/v1/sync/*', requireProjectId)
  .use('/v1/projects/*', requireProjectId)
  .use('/v1/jobs/*', requireProjectId)
  .route('/v1/sync', syncRoutes)
  .route('/v1/projects', projectRoutes)
  .route('/v1/jobs', jobRoutes)
  // Auth, GitHub, registry routes (no x-project-id middleware) — paths declared
  // inside the route modules using absolute /v1/... prefixes.
  .route('/', authRoutes)
  .route('/', githubRoutes)
  .route('/', registryRoutes)
  // Always-on: ops + waitlist (no feature flag, no auth)
  .route('/', opsRoutes)
  .route('/', waitlistRoutes);

export default app;
