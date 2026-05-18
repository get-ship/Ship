import type { Context, Next } from 'hono';
import { requireSession } from '../lib/session-auth.js';

/**
 * Gating for /v1/sync/*, /v1/projects/*, /v1/jobs/*.
 *
 * 1. Require a Better Auth session (cookie) or API key. 401 if missing.
 * 2. Require an `x-project-id` header. 400 if missing.
 * 3. Authorize the caller against the project: brand-new project DOs
 *    claim the caller as 'owner' on first contact, established projects
 *    require an existing membership row. 403 otherwise.
 *
 * On success, sets `userId` and `projectId` on context for downstream
 * handlers.
 */
export const requireProjectAuth = async (
  c: Context<{ Bindings: Env; Variables: { projectId: string; userId: string } }>,
  next: Next,
) => {
  const session = await requireSession(c.req.raw, c.env);
  if (session instanceof Response) return session;

  const projectId = c.req.header('x-project-id');
  if (!projectId) return c.json({ error: 'Missing x-project-id header' }, 400);

  try {
    const id = c.env.PLATFORM.idFromName(projectId);
    const stub = c.env.PLATFORM.get(id);
    const projects = await stub.projects();
    const { authorized } = await projects.claimOrCheck(session.sub);
    if (!authorized) {
      return c.json({ error: 'Forbidden — not a member of this project' }, 403);
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return c.json({ error: `Membership check failed: ${msg}` }, 500);
  }

  c.set('userId', session.sub);
  c.set('projectId', projectId);
  await next();
};
