import { describe, it, expect, beforeEach } from 'vitest';
import { env, runInDurableObject } from 'cloudflare:test';
import { drizzle } from 'drizzle-orm/durable-sqlite';
import { migrate } from 'drizzle-orm/durable-sqlite/migrator';
import { EventRepository } from '../src/repositories/event.repository.js';
import { ActorProjectionRepository } from '../src/repositories/actor-projection.repository.js';
import { SessionProjectionRepository } from '../src/repositories/session-projection.repository.js';
import { project_schema } from '../src/data/schema/index.js';
import migrations from '../migrations/migrations.js';
import type { EventEnvelope } from '@ship-cloud/types';

function makeEvent(overrides: Partial<EventEnvelope> = {}): EventEnvelope {
  return {
    id: crypto.randomUUID(),
    event_type: 'actor.created',
    entity_id: crypto.randomUUID(),
    actor: 'ship',
    payload_json: JSON.stringify({ name: 'test-actor' }),
    version: 1,
    elevated: false,
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

describe('Project EventRepository', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PROJECT.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PROJECT.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      await migrate(db, migrations);
    });
  });

  describe('push', () => {
    it('should accept and store events', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        const events = [makeEvent(), makeEvent()];
        const result = await repo.push(events);

        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(2);
      });
    });

    it('should deduplicate by event ID', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        const event = makeEvent();
        await repo.push([event]);
        const result = await repo.push([event]);

        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(0);
      });
    });

    it('should accept empty array', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        const result = await repo.push([]);
        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(0);
      });
    });
  });

  describe('pull', () => {
    it('should return empty from empty store', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        const result = await repo.pull();

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(0);
        expect(result.value.cursor).toBeNull();
      });
    });

    it('should return all events when no cursor', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        await repo.push([makeEvent(), makeEvent(), makeEvent()]);

        const result = await repo.pull();

        expect(result.value.events).toHaveLength(3);
        expect(result.value.cursor).not.toBeNull();
      });
    });

    it('should return events after cursor', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        const first = makeEvent();
        const second = makeEvent();
        await repo.push([first, second]);

        const result = await repo.pull(first.id);

        expect(result.isSuccess).toBe(true);
        expect(result.value.events).toHaveLength(1);
        expect(result.value.events[0]!.id).toBe(second.id);
      });
    });

    it('should respect limit', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
        const repo = new EventRepository(db);

        await repo.push(Array.from({ length: 5 }, () => makeEvent()));

        const result = await repo.pull(undefined, 2);
        expect(result.value.events).toHaveLength(2);
      });
    });

    it('should maintain insertion order', async () => {
      await runInDurableObject(stub, async (_instance: any, state: any) => {
        const db = drizzle(state.storage, { schema: project_schema });
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

describe('Project ActorProjection', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PROJECT.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PROJECT.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      await migrate(db, migrations);
    });
  });

  it('should create actor from event', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const projection = new ActorProjectionRepository(db);

      const actorId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({
        event_type: 'actor.created',
        entity_id: actorId,
        payload_json: JSON.stringify({ name: 'my-actor' }),
      }));

      const result = await projection.list();
      expect(result.isSuccess).toBe(true);
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(actorId);
      expect(result.value[0]!.name).toBe('my-actor');
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition: created -> updated -> deleted', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const projection = new ActorProjectionRepository(db);

      const actorId = crypto.randomUUID();

      await projection.applyEvent(makeEvent({
        event_type: 'actor.created', entity_id: actorId,
        payload_json: JSON.stringify({ name: 'v1' }),
      }));
      expect((await projection.list()).value[0]!.status).toBe('created');

      await projection.applyEvent(makeEvent({
        event_type: 'actor.woke', entity_id: actorId,
      }));
      expect((await projection.list()).value[0]!.status).toBe('woke');

      await projection.applyEvent(makeEvent({
        event_type: 'actor.slept', entity_id: actorId,
      }));
      expect((await projection.list()).value[0]!.status).toBe('slept');

      await projection.applyEvent(makeEvent({
        event_type: 'actor.stopped', entity_id: actorId,
      }));
      expect((await projection.list()).value[0]!.status).toBe('stopped');
    });
  });

  it('should track multiple actors independently', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const projection = new ActorProjectionRepository(db);

      const a1 = crypto.randomUUID();
      const a2 = crypto.randomUUID();

      await projection.applyEvent(makeEvent({
        event_type: 'actor.created', entity_id: a1,
        payload_json: JSON.stringify({ name: 'actor-a' }),
      }));
      await projection.applyEvent(makeEvent({
        event_type: 'actor.created', entity_id: a2,
        payload_json: JSON.stringify({ name: 'actor-b' }),
      }));

      const result = await projection.list();
      expect(result.value).toHaveLength(2);
    });
  });

  it('should ignore unrelated event types', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const projection = new ActorProjectionRepository(db);

      await projection.applyEvent(makeEvent({
        event_type: 'agent.session.started',
        entity_id: crypto.randomUUID(),
      }));

      const result = await projection.list();
      expect(result.value).toHaveLength(0);
    });
  });
});

describe('Project SessionProjection', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PROJECT.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PROJECT.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      await migrate(db, migrations);
    });
  });

  it('should create session from started event', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const projection = new SessionProjectionRepository(db);

      const sessionId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({
        event_type: 'agent.session.started',
        entity_id: sessionId,
        actor_id: 'actor-1',
      }));

      const result = await projection.list();
      expect(result.isSuccess).toBe(true);
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(sessionId);
      expect(result.value[0]!.status).toBe('started');
      expect(result.value[0]!.actor_id).toBe('actor-1');
    });
  });

  it('should transition: started -> completed', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const projection = new SessionProjectionRepository(db);

      const sessionId = crypto.randomUUID();

      await projection.applyEvent(makeEvent({
        event_type: 'agent.session.started', entity_id: sessionId, actor_id: 'a1',
      }));
      await projection.applyEvent(makeEvent({
        event_type: 'agent.session.completed', entity_id: sessionId,
      }));

      expect((await projection.list()).value[0]!.status).toBe('completed');
    });
  });
});

describe('Project SyncFacade integration', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    const id = env.PROJECT.idFromName(`test-${crypto.randomUUID()}`);
    stub = env.PROJECT.get(id);

    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      await migrate(db, migrations);
    });
  });

  it('push should store events AND update projections', async () => {
    await runInDurableObject(stub, async (_instance: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const eventRepo = new EventRepository(db);
      const actorProjection = new ActorProjectionRepository(db);
      const sessionProjection = new SessionProjectionRepository(db);

      const actorId = crypto.randomUUID();
      const sessionId = crypto.randomUUID();
      const events = [
        makeEvent({ event_type: 'actor.created', entity_id: actorId, payload_json: JSON.stringify({ name: 'agent-1' }) }),
        makeEvent({ event_type: 'agent.session.started', entity_id: sessionId, actor_id: actorId }),
      ];

      const pushResult = await eventRepo.push(events);
      expect(pushResult.value).toBe(2);

      for (const event of events) {
        await actorProjection.applyEvent(event);
        await sessionProjection.applyEvent(event);
      }

      const actors = await actorProjection.list();
      expect(actors.value).toHaveLength(1);
      expect(actors.value[0]!.name).toBe('agent-1');

      const sessions = await sessionProjection.list();
      expect(sessions.value).toHaveLength(1);
      expect(sessions.value[0]!.actor_id).toBe(actorId);

      const pullResult = await eventRepo.pull();
      expect(pullResult.value.events).toHaveLength(2);
    });
  });
});
