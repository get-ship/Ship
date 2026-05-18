import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, platform_schema, createMigratedStub } from './helpers.js';
import { MembersRepository } from '../src/repositories/members.repository.js';

describe('Platform MembersRepository', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('starts empty', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      const r = await repo.list();
      expect(r.value).toEqual([]);
      expect((await repo.count()).value).toBe(0);
    });
  });

  it('adds a member and reports membership', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      await repo.add('user-1', 'owner');
      expect((await repo.isMember('user-1')).value).toBe(true);
      expect((await repo.isMember('other')).value).toBe(false);
      const list = (await repo.list()).value;
      expect(list).toHaveLength(1);
      expect(list[0]!.user_id).toBe('user-1');
      expect(list[0]!.role).toBe('owner');
    });
  });

  it('add is idempotent on existing user', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      await repo.add('user-1', 'owner');
      await repo.add('user-1', 'member');
      const list = (await repo.list()).value;
      expect(list).toHaveLength(1);
      // First write wins (onConflictDoNothing) — role stays 'owner'.
      expect(list[0]!.role).toBe('owner');
    });
  });

  it('removes a member', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      await repo.add('user-1', 'owner');
      await repo.remove('user-1');
      expect((await repo.isMember('user-1')).value).toBe(false);
    });
  });

  it('claimOrCheck on empty DO grants ownership and reports claimed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      const r = await repo.claimOrCheck('creator');
      expect(r.value).toEqual({ authorized: true, claimed: true });
      expect((await repo.isMember('creator')).value).toBe(true);
    });
  });

  it('claimOrCheck after first claim rejects strangers but accepts members', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      await repo.claimOrCheck('creator');
      const stranger = await repo.claimOrCheck('intruder');
      expect(stranger.value).toEqual({ authorized: false, claimed: false });
      const owner = await repo.claimOrCheck('creator');
      expect(owner.value).toEqual({ authorized: true, claimed: false });
    });
  });

  it('claimOrCheck for an added regular member is authorized', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const repo = new MembersRepository(drizzle(state.storage, { schema: platform_schema }));
      await repo.claimOrCheck('owner-1');
      await repo.add('member-1', 'member');
      const r = await repo.claimOrCheck('member-1');
      expect(r.value).toEqual({ authorized: true, claimed: false });
    });
  });
});
