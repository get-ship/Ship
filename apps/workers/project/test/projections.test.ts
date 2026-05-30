import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, project_schema, makeEvent, createMigratedStub } from './helpers.js';
import { EventRepository } from '../src/repositories/event.repository.js';
import { ActorProjectionRepository } from '../src/repositories/actor-projection.repository.js';
import { SessionProjectionRepository } from '../src/repositories/session-projection.repository.js';

describe('Project ActorProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create actor from event', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const actorId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({
        event_type: 'actor.created', entity_id: actorId,
        payload_json: JSON.stringify({ name: 'my-actor' }),
      }));
      const result = await proj.list();
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(actorId);
      expect(result.value[0]!.name).toBe('my-actor');
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition: created -> woke -> slept -> stopped', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const actorId = crypto.randomUUID();

      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: actorId, payload_json: JSON.stringify({ name: 'v1' }) }));
      expect((await proj.list()).value[0]!.status).toBe('created');

      await proj.applyEvent(makeEvent({ event_type: 'actor.woke', entity_id: actorId }));
      expect((await proj.list()).value[0]!.status).toBe('woke');

      await proj.applyEvent(makeEvent({ event_type: 'actor.slept', entity_id: actorId }));
      expect((await proj.list()).value[0]!.status).toBe('slept');

      await proj.applyEvent(makeEvent({ event_type: 'actor.stopped', entity_id: actorId }));
      expect((await proj.list()).value[0]!.status).toBe('stopped');
    });
  });

  it('should transition crashed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const actorId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: actorId, payload_json: JSON.stringify({ name: 'v1' }) }));
      await proj.applyEvent(makeEvent({ event_type: 'actor.crashed', entity_id: actorId }));
      expect((await proj.list()).value[0]!.status).toBe('crashed');
    });
  });

  it('should track multiple actors independently', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const [a1, a2] = [crypto.randomUUID(), crypto.randomUUID()];
      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: a1, payload_json: JSON.stringify({ name: 'a' }) }));
      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: a2, payload_json: JSON.stringify({ name: 'b' }) }));
      expect((await proj.list()).value).toHaveLength(2);
    });
  });

  it('should ignore unrelated event types', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.started', entity_id: crypto.randomUUID() }));
      expect((await proj.list()).value).toHaveLength(0);
    });
  });
});

describe('Project SessionProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create session from started event', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.started', entity_id: sessionId, actor_id: 'actor-1' }));
      const result = await proj.list();
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(sessionId);
      expect(result.value[0]!.status).toBe('started');
      expect(result.value[0]!.actor_id).toBe('actor-1');
    });
  });

  it('should transition: started -> completed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.started', entity_id: sessionId, actor_id: 'a1' }));
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.completed', entity_id: sessionId }));
      expect((await proj.list()).value[0]!.status).toBe('completed');
    });
  });

  it('should transition: started -> failed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.started', entity_id: sessionId, actor_id: 'a1' }));
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.failed', entity_id: sessionId }));
      expect((await proj.list()).value[0]!.status).toBe('failed');
    });
  });

  it('should transition: started -> cancelled', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: project_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.started', entity_id: sessionId, actor_id: 'a1' }));
      await proj.applyEvent(makeEvent({ event_type: 'agent.session.cancelled', entity_id: sessionId }));
      expect((await proj.list()).value[0]!.status).toBe('cancelled');
    });
  });
});

describe('Project SyncFacade integration', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('push should store events AND update projections', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const db = drizzle(state.storage, { schema: project_schema });
      const eventRepo = new EventRepository(db);
      const actorProj = new ActorProjectionRepository(db);
      const sessionProj = new SessionProjectionRepository(db);

      const [actorId, sessionId] = [crypto.randomUUID(), crypto.randomUUID()];
      const events = [
        makeEvent({ event_type: 'actor.created', entity_id: actorId, payload_json: JSON.stringify({ name: 'agent-1' }) }),
        makeEvent({ event_type: 'agent.session.started', entity_id: sessionId, actor_id: actorId }),
      ];

      expect((await eventRepo.push(events)).value).toBe(2);
      for (const event of events) {
        await actorProj.applyEvent(event);
        await sessionProj.applyEvent(event);
      }

      expect((await actorProj.list()).value[0]!.name).toBe('agent-1');
      expect((await sessionProj.list()).value[0]!.actor_id).toBe(actorId);
      expect((await eventRepo.pull()).value.events).toHaveLength(2);
    });
  });
});
