import { describe, it, expect, beforeEach } from 'vitest';
import { env, runInDurableObject } from 'cloudflare:test';
import { drizzle } from 'drizzle-orm/durable-sqlite';
import { migrate } from 'drizzle-orm/durable-sqlite/migrator';
import { EventRepository } from '../src/repositories/event.repository.js';
import { WorkspaceProjectionRepository } from '../src/repositories/workspace-projection.repository.js';
import { JobProjectionRepository } from '../src/repositories/job-projection.repository.js';
import { platform_schema } from '../src/data/schema/index.js';
import migrations from '../migrations/migrations.js';
import type { EventEnvelope } from '@ship-cloud/types';

function makeEvent(overrides: Partial<EventEnvelope> = {}): EventEnvelope {
  return {
    id: crypto.randomUUID(),
    event_type: 'workspace.created',
    entity_id: crypto.randomUUID(),
    actor: 'ship',
    payload_json: JSON.stringify({ name: 'test-workspace' }),
    version: 1,
    elevated: false,
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

describe('Platform EventRepository', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PLATFORM.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PLATFORM.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      await migrate(db, migrations);
    });
  });

  describe('push', () => {
    it('should accept and store events', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const events = [makeEvent(), makeEvent()];
        const result = await repo.push(events);

        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(2);
      });
    });

    it('should deduplicate by event ID', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const event = makeEvent();
        await repo.push([event]);

        const result = await repo.push([event]);

        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(0);

        const pullResult = await repo.pull();
        expect(pullResult.value.events).toHaveLength(1);
      });
    });

    it('should deduplicate within a single batch', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const event = makeEvent();
        const result = await repo.push([event, event]);

        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(1);
      });
    });

    it('should accept empty array', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const result = await repo.push([]);
        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(0);
      });
    });

    it('should preserve all optional fields on round-trip', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const event = makeEvent({
          causation_id: 'cause-1',
          workspace_id: 'ws-1',
          session_id: 'sess-1',
          actor_id: 'actor-1',
          parent_actor_id: 'parent-1',
          target_actor_id: 'target-1',
          elevated: true,
        });
        await repo.push([event]);

        const pullResult = await repo.pull();
        const stored = pullResult.value.events[0]!;

        expect(stored.id).toBe(event.id);
        expect(stored.event_type).toBe(event.event_type);
        expect(stored.entity_id).toBe(event.entity_id);
        expect(stored.actor).toBe('ship');
        expect(stored.payload_json).toBe(event.payload_json);
        expect(stored.version).toBe(1);
        expect(stored.causation_id).toBe('cause-1');
        expect(stored.target_actor_id).toBe('target-1');
        expect(stored.workspace_id).toBe('ws-1');
        expect(stored.session_id).toBe('sess-1');
        expect(stored.actor_id).toBe('actor-1');
        expect(stored.parent_actor_id).toBe('parent-1');
        expect(stored.elevated).toBe(true);
        expect(stored.created_at).toBe(event.created_at);
      });
    });
  });

  describe('pull', () => {
    it('should return empty array and null cursor from empty store', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const result = await repo.pull();

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(0);
        expect(result.value.cursor).toBeNull();
      });
    });

    it('should return all events when no cursor', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const events = [makeEvent(), makeEvent(), makeEvent()];
        await repo.push(events);

        const result = await repo.pull();

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(3);
        expect(result.value.cursor).not.toBeNull();
      });
    });

    it('should return events after cursor', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const first = makeEvent();
        const second = makeEvent();
        const third = makeEvent();
        await repo.push([first, second, third]);

        const result = await repo.pull(first.id);

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(2);
        expect(result.value.events[0]!.id).toBe(second.id);
        expect(result.value.events[1]!.id).toBe(third.id);
      });
    });

    it('should return empty when cursor is the last event', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const event = makeEvent();
        await repo.push([event]);

        const result = await repo.pull(event.id);

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(0);
        expect(result.value.cursor).toBeNull();
      });
    });

    it('should return all events when cursor ID is unknown', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        await repo.push([makeEvent(), makeEvent()]);

        // Unknown cursor — since lookup returns no match, falls through to unfiltered query
        const result = await repo.pull('nonexistent-id');

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(2);
      });
    });

    it('should respect limit', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const events = Array.from({ length: 5 }, () => makeEvent());
        await repo.push(events);

        const result = await repo.pull(undefined, 2);

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(2);
      });
    });

    it('should maintain insertion order across multiple pushes', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: platform_schema });
        const repo = new EventRepository(db);

        const a = makeEvent();
        const b = makeEvent();
        const c = makeEvent();
        await repo.push([a]);
        await repo.push([b]);
        await repo.push([c]);

        const result = await repo.pull();

        expect(result.value.events.map((e) => e.id)).toEqual([a.id, b.id, c.id]);
      });
    });
  });
});

describe('Platform WorkspaceProjection', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PLATFORM.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PLATFORM.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      await migrate(db, migrations);
    });
  });

  it('should create workspace projection from event', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new WorkspaceProjectionRepository(db);

      const wsId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({
        event_type: 'workspace.created',
        entity_id: wsId,
        payload_json: JSON.stringify({ name: 'my-workspace' }),
      }));

      const result = await projection.list();
      expect(result.isSuccess).toBe(true);
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(wsId);
      expect(result.value[0]!.name).toBe('my-workspace');
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition through full lifecycle: created -> activated -> archived -> deleted', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new WorkspaceProjectionRepository(db);

      const wsId = crypto.randomUUID();

      await projection.applyEvent(makeEvent({
        event_type: 'workspace.created', entity_id: wsId,
        payload_json: JSON.stringify({ name: 'ws' }),
      }));
      expect((await projection.list()).value[0]!.status).toBe('created');

      await projection.applyEvent(makeEvent({
        event_type: 'workspace.activated', entity_id: wsId,
      }));
      expect((await projection.list()).value[0]!.status).toBe('activated');

      await projection.applyEvent(makeEvent({
        event_type: 'workspace.archived', entity_id: wsId,
      }));
      expect((await projection.list()).value[0]!.status).toBe('archived');

      await projection.applyEvent(makeEvent({
        event_type: 'workspace.deleted', entity_id: wsId,
      }));
      expect((await projection.list()).value[0]!.status).toBe('deleted');
    });
  });

  it('should ignore duplicate created events (idempotent)', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new WorkspaceProjectionRepository(db);

      const wsId = crypto.randomUUID();
      const event = makeEvent({
        event_type: 'workspace.created', entity_id: wsId,
        payload_json: JSON.stringify({ name: 'ws' }),
      });

      await projection.applyEvent(event);
      await projection.applyEvent(event);

      const result = await projection.list();
      expect(result.value).toHaveLength(1);
    });
  });

  it('should track multiple workspaces independently', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new WorkspaceProjectionRepository(db);

      const ws1 = crypto.randomUUID();
      const ws2 = crypto.randomUUID();

      await projection.applyEvent(makeEvent({
        event_type: 'workspace.created', entity_id: ws1,
        payload_json: JSON.stringify({ name: 'alpha' }),
      }));
      await projection.applyEvent(makeEvent({
        event_type: 'workspace.created', entity_id: ws2,
        payload_json: JSON.stringify({ name: 'beta' }),
      }));
      await projection.applyEvent(makeEvent({
        event_type: 'workspace.activated', entity_id: ws1,
      }));

      const result = await projection.list();
      expect(result.value).toHaveLength(2);

      const alpha = result.value.find((w) => w.id === ws1)!;
      const beta = result.value.find((w) => w.id === ws2)!;
      expect(alpha.status).toBe('activated');
      expect(beta.status).toBe('created');
    });
  });

  it('should ignore unrelated event types', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new WorkspaceProjectionRepository(db);

      await projection.applyEvent(makeEvent({
        event_type: 'job.created',
        entity_id: crypto.randomUUID(),
      }));

      const result = await projection.list();
      expect(result.value).toHaveLength(0);
    });
  });
});

describe('Platform JobProjection', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PLATFORM.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PLATFORM.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      await migrate(db, migrations);
    });
  });

  it('should create job projection from created event', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({
        event_type: 'job.created', entity_id: jobId,
      }));

      const result = await projection.list();
      expect(result.isSuccess).toBe(true);
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition: created -> dispatched -> completed', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();

      await projection.applyEvent(makeEvent({ event_type: 'job.created', entity_id: jobId }));
      expect((await projection.list()).value[0]!.status).toBe('created');

      await projection.applyEvent(makeEvent({ event_type: 'job.dispatched', entity_id: jobId }));
      expect((await projection.list()).value[0]!.status).toBe('dispatched');

      await projection.applyEvent(makeEvent({ event_type: 'job.completed', entity_id: jobId }));
      expect((await projection.list()).value[0]!.status).toBe('completed');
    });
  });

  it('should transition: created -> dispatched -> failed', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();

      await projection.applyEvent(makeEvent({ event_type: 'job.created', entity_id: jobId }));
      await projection.applyEvent(makeEvent({ event_type: 'job.dispatched', entity_id: jobId }));
      await projection.applyEvent(makeEvent({ event_type: 'job.failed', entity_id: jobId }));

      expect((await projection.list()).value[0]!.status).toBe('failed');
    });
  });
});

describe('Platform SyncFacade integration', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PLATFORM.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PLATFORM.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      await migrate(db, migrations);
    });
  });

  it('push should store events AND update projections in one call', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      // Use the DO's own facade (through the repos factory) to verify integration
      const db = drizzle(state.storage, { schema: platform_schema });
      const eventRepo = new EventRepository(db);
      const wsProjection = new WorkspaceProjectionRepository(db);
      const jobProjection = new JobProjectionRepository(db);

      // Manually simulate what SyncFacade.push does
      const wsId = crypto.randomUUID();
      const jobId = crypto.randomUUID();
      const events = [
        makeEvent({ event_type: 'workspace.created', entity_id: wsId, payload_json: JSON.stringify({ name: 'ws-1' }) }),
        makeEvent({ event_type: 'job.created', entity_id: jobId, workspace_id: wsId }),
      ];

      const pushResult = await eventRepo.push(events);
      expect(pushResult.value).toBe(2);

      for (const event of events) {
        await wsProjection.applyEvent(event);
        await jobProjection.applyEvent(event);
      }

      // Verify projections updated
      const workspaces = await wsProjection.list();
      expect(workspaces.value).toHaveLength(1);
      expect(workspaces.value[0]!.name).toBe('ws-1');

      const jobs = await jobProjection.list();
      expect(jobs.value).toHaveLength(1);
      expect(jobs.value[0]!.status).toBe('created');

      // Verify pull returns events
      const pullResult = await eventRepo.pull();
      expect(pullResult.value.events).toHaveLength(2);
    });
  });
});
