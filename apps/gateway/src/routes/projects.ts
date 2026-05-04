import { Hono } from 'hono';

const projects = new Hono<{
  Bindings: Env;
  Variables: { projectId: string };
}>();

/**
 * List projects
 * GET /api/projects
 */
projects.get('/', async (c) => {
  const projectId = c.get('projectId');

  const id = c.env.PLATFORM.idFromName(projectId);
  const stub = c.env.PLATFORM.get(id);
  const facade = await stub.projects();
  const result = await facade.list();

  return c.json(result);
});

export { projects as projectRoutes };
