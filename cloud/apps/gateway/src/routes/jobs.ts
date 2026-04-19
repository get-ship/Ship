import { Hono } from 'hono';

const jobs = new Hono<{
  Bindings: Env;
  Variables: { projectId: string };
}>();

/**
 * List jobs for a project
 * GET /api/jobs
 */
jobs.get('/', async (c) => {
  const projectId = c.get('projectId');

  const id = c.env.PLATFORM.idFromName(projectId);
  const stub = c.env.PLATFORM.get(id);
  const facade = await stub.jobs();
  const result = await facade.list();

  return c.json(result);
});

export { jobs as jobRoutes };
