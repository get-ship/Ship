import type { Context, Next } from 'hono';

/**
 * Stubbed auth middleware.
 * For now, extracts project-id and user-id from headers.
 * Will be replaced with Better Auth session extraction.
 */
export const authMiddleware = async (c: Context, next: Next) => {
  // TODO: Replace with Better Auth session extraction
  const projectId = c.req.header('x-project-id');

  if (!projectId) {
    return c.json({ error: 'Missing x-project-id header' }, 400);
  }

  c.set('projectId', projectId);

  await next();
};
