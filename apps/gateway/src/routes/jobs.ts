import { Hono } from 'hono';

const jobs = new Hono<{
  Bindings: Env;
  Variables: { spaceId: string };
}>();

/**
 * List jobs for a project
 * GET /api/jobs
 */
jobs.get('/', async (c) => {
  const spaceId = c.get('spaceId');

  const id = c.env.PLATFORM.idFromName(spaceId);
  const stub = c.env.PLATFORM.get(id);
  const facade = await stub.jobs();
  const result = await facade.list();

  return c.json(result);
});

export { jobs as jobRoutes };
