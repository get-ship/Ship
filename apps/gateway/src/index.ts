import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { errorHandler } from './middleware/on-error.js';
import { syncRoutes, projectRoutes, jobRoutes } from './routes/index.js';
import { authRoutes } from './routes/auth.js';
import { githubRoutes } from './routes/github.js';
import { registryRoutes } from './routes/registry.js';

const CORS_ORIGINS = ['http://localhost:3000', 'https://staging.getship.dev', 'https://getship.dev'];

const app = new Hono<{ Bindings: Env; Variables: { projectId: string } }>()
  .onError(errorHandler)
  .use(
    '/api/*',
    cors({
      origin: CORS_ORIGINS,
      allowHeaders: ['Content-Type', 'Authorization', 'x-project-id'],
      allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
      credentials: true,
    }),
  )
  // Platform routes (project/job proxy to Durable Objects — require x-project-id)
  .use('/api/sync/*', async (c, next) => {
    const projectId = c.req.header('x-project-id');
    if (!projectId) return c.json({ error: 'Missing x-project-id header' }, 400);
    c.set('projectId', projectId);
    return next();
  })
  .use('/api/projects/*', async (c, next) => {
    const projectId = c.req.header('x-project-id');
    if (!projectId) return c.json({ error: 'Missing x-project-id header' }, 400);
    c.set('projectId', projectId);
    return next();
  })
  .use('/api/jobs/*', async (c, next) => {
    const projectId = c.req.header('x-project-id');
    if (!projectId) return c.json({ error: 'Missing x-project-id header' }, 400);
    c.set('projectId', projectId);
    return next();
  })
  .route('/api/sync', syncRoutes)
  .route('/api/projects', projectRoutes)
  .route('/api/jobs', jobRoutes)
  // Auth, GitHub, registry routes (no x-project-id middleware)
  .route('/', authRoutes)
  .route('/', githubRoutes)
  .route('/', registryRoutes);

export default app;
