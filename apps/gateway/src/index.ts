import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { authMiddleware } from './middleware/auth.js';
import { errorHandler } from './middleware/on-error.js';
import { syncRoutes, workspaceRoutes, jobRoutes } from './routes/index.js';

const app = new Hono<{
  Bindings: Env;
  Variables: { projectId: string };
}>()
  .onError(errorHandler)
  .use(
    '/api/*',
    cors({
      origin: ['http://localhost:3000', 'https://staging.getship.dev', 'https://getship.dev'],
      allowHeaders: ['Content-Type', 'Authorization', 'x-project-id'],
      allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
      credentials: true,
    }),
  )
  .use('/api/*', authMiddleware)
  .route('/api/sync', syncRoutes)
  .route('/api/workspaces', workspaceRoutes)
  .route('/api/jobs', jobRoutes)
  .get('/health', (c) => {
    return c.json({ status: 'ok', timestamp: new Date().toISOString() });
  });

export default app;

export type AppType = typeof app;
