import { Hono } from 'hono';

const workspaces = new Hono<{
  Bindings: Env;
  Variables: { projectId: string };
}>();

/**
 * List workspaces for a project
 * GET /api/workspaces
 */
workspaces.get('/', async (c) => {
  const projectId = c.get('projectId');

  const id = c.env.PLATFORM.idFromName(projectId);
  const stub = c.env.PLATFORM.get(id);
  const facade = await stub.workspaces();
  const result = await facade.list();

  return c.json(result);
});

export { workspaces as workspaceRoutes };
