import { Hono } from 'hono';
import type { EventEnvelope } from '@ship-cloud/types';

const sync = new Hono<{
  Bindings: Env;
  Variables: { projectId: string };
}>();

/**
 * Push events to a project's PlatformDO
 * POST /api/sync/platform/push
 */
sync.post('/platform/push', async (c) => {
  const projectId = c.get('projectId');
  const body = await c.req.json<{ events: EventEnvelope[] }>();

  if (!body.events || !Array.isArray(body.events)) {
    return c.json({ error: 'Missing events array' }, 400);
  }

  const id = c.env.PLATFORM.idFromName(projectId);
  const stub = c.env.PLATFORM.get(id);
  const syncFacade = await stub.sync();
  const result = await syncFacade.push(body.events);

  return c.json(result);
});

/**
 * Pull events from a project's PlatformDO
 * GET /api/sync/platform/pull?since=<cursor>&limit=<n>
 */
sync.get('/platform/pull', async (c) => {
  const projectId = c.get('projectId');
  const since = c.req.query('since');
  const limit = c.req.query('limit');

  const id = c.env.PLATFORM.idFromName(projectId);
  const stub = c.env.PLATFORM.get(id);
  const syncFacade = await stub.sync();
  const result = await syncFacade.pull(
    since || undefined,
    limit ? parseInt(limit, 10) : undefined,
  );

  return c.json(result);
});

/**
 * Push events to a WorkspaceDO
 * POST /api/sync/workspace/:workspaceId/push
 */
sync.post('/workspace/:workspaceId/push', async (c) => {
  const workspaceId = c.req.param('workspaceId');
  const body = await c.req.json<{ events: EventEnvelope[] }>();

  if (!body.events || !Array.isArray(body.events)) {
    return c.json({ error: 'Missing events array' }, 400);
  }

  const id = c.env.PROJECT.idFromName(workspaceId);
  const stub = c.env.PROJECT.get(id);
  const syncFacade = await stub.sync();
  const result = await syncFacade.push(body.events);

  return c.json(result);
});

/**
 * Pull events from a WorkspaceDO
 * GET /api/sync/workspace/:workspaceId/pull?since=<cursor>&limit=<n>
 */
sync.get('/workspace/:workspaceId/pull', async (c) => {
  const workspaceId = c.req.param('workspaceId');
  const since = c.req.query('since');
  const limit = c.req.query('limit');

  const id = c.env.PROJECT.idFromName(workspaceId);
  const stub = c.env.PROJECT.get(id);
  const syncFacade = await stub.sync();
  const result = await syncFacade.pull(
    since || undefined,
    limit ? parseInt(limit, 10) : undefined,
  );

  return c.json(result);
});

export { sync as syncRoutes };
