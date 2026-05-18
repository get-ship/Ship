import { Hono } from 'hono';

type MemberRole = 'owner' | 'member';
interface MemberRow { user_id: string; role: string; created_at: string }

const projects = new Hono<{
  Bindings: Env;
  Variables: { projectId: string; userId: string };
}>();

/**
 * List projects for the platform DO this caller is scoped to.
 *
 * GET /v1/projects
 */
projects.get('/', async (c) => {
  const projectId = c.get('projectId');
  const stub = c.env.PLATFORM.get(c.env.PLATFORM.idFromName(projectId));
  const facade = await stub.projects();
  const result = await facade.list();
  return c.json(result);
});

/**
 * List members for the project pinned by `x-project-id`.
 * Caller must be a member (already enforced by requireProjectAuth).
 *
 * GET /v1/projects/:id/members
 */
projects.get('/:id/members', async (c) => {
  const projectId = c.req.param('id');
  if (projectId !== c.get('projectId')) {
    return c.json({ error: 'Path id does not match x-project-id' }, 400);
  }
  const stub = c.env.PLATFORM.get(c.env.PLATFORM.idFromName(projectId));
  const facade = await stub.projects();
  const list = await facade.listMembers();
  return c.json({ members: list });
});

/**
 * Add a member. Owners only.
 *
 * POST /v1/projects/:id/members  { user_id, role? }
 */
projects.post('/:id/members', async (c) => {
  const projectId = c.req.param('id');
  if (projectId !== c.get('projectId')) {
    return c.json({ error: 'Path id does not match x-project-id' }, 400);
  }

  let body: { user_id?: string; role?: MemberRole };
  try {
    body = await c.req.json<{ user_id?: string; role?: MemberRole }>();
  } catch {
    return c.json({ error: 'Invalid JSON body' }, 400);
  }
  if (!body.user_id || typeof body.user_id !== 'string') {
    return c.json({ error: 'Missing user_id' }, 400);
  }
  const role: MemberRole = body.role === 'owner' ? 'owner' : 'member';

  const stub = c.env.PLATFORM.get(c.env.PLATFORM.idFromName(projectId));
  const facade = await stub.projects();
  const list = (await facade.listMembers()) as MemberRow[];
  const caller = list.find((m) => m.user_id === c.get('userId'));
  if (!caller || caller.role !== 'owner') {
    return c.json({ error: 'Only owners may add members' }, 403);
  }

  await facade.addMember(body.user_id, role);
  return c.json({ ok: true, user_id: body.user_id, role });
});

/**
 * Remove a member. Owners only. Refuses to delete the last owner.
 *
 * DELETE /v1/projects/:id/members/:userId
 */
projects.delete('/:id/members/:userId', async (c) => {
  const projectId = c.req.param('id');
  if (projectId !== c.get('projectId')) {
    return c.json({ error: 'Path id does not match x-project-id' }, 400);
  }
  const targetUserId = c.req.param('userId');

  const stub = c.env.PLATFORM.get(c.env.PLATFORM.idFromName(projectId));
  const facade = await stub.projects();
  const list = (await facade.listMembers()) as MemberRow[];
  const caller = list.find((m) => m.user_id === c.get('userId'));
  if (!caller || caller.role !== 'owner') {
    return c.json({ error: 'Only owners may remove members' }, 403);
  }

  const target = list.find((m) => m.user_id === targetUserId);
  if (!target) return c.json({ error: 'Member not found' }, 404);

  if (target.role === 'owner') {
    const owners = list.filter((m) => m.role === 'owner').length;
    if (owners <= 1) {
      return c.json({ error: 'Cannot remove the last owner' }, 409);
    }
  }

  await facade.removeMember(targetUserId);
  return c.json({ ok: true });
});

export { projects as projectRoutes };
