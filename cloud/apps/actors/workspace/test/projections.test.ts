import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, workspace_schema, makeEvent, createMigratedStub } from './helpers.js';
import { EventRepository } from '../src/repositories/event.repository.js';
import { ActorProjectionRepository } from '../src/repositories/actor-projection.repository.js';
import { SessionProjectionRepository } from '../src/repositories/session-projection.repository.js';

describe('Workspace ActorProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create actor from event', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
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

  it('should transition: created -> updated -> deleted', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
      const actorId = crypto.randomUUID();

      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: actorId, payload_json: JSON.stringify({ name: 'v1' }) }));
      expect((await proj.list()).value[0]!.status).toBe('created');

      await proj.applyEvent(makeEvent({ event_type: 'actor.updated', entity_id: actorId, payload_json: JSON.stringify({ name: 'v2' }) }));
      const updated = (await proj.list()).value[0]!;
      expect(updated.status).toBe('updated');
      expect(updated.name).toBe('v2');

      await proj.applyEvent(makeEvent({ event_type: 'actor.deleted', entity_id: actorId }));
      expect((await proj.list()).value[0]!.status).toBe('deleted');
    });
  });

  it('should track multiple actors independently', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
      const [a1, a2] = [crypto.randomUUID(), crypto.randomUUID()];
      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: a1, payload_json: JSON.stringify({ name: 'a' }) }));
      await proj.applyEvent(makeEvent({ event_type: 'actor.created', entity_id: a2, payload_json: JSON.stringify({ name: 'b' }) }));
      expect((await proj.list()).value).toHaveLength(2);
    });
  });

  it('should ignore unrelated event types', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ActorProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
      await proj.applyEvent(makeEvent({ event_type: 'session.started', entity_id: crypto.randomUUID() }));
      expect((await proj.list()).value).toHaveLength(0);
    });
  });
});

describe('Workspace SessionProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create session from started event', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'session.started', entity_id: sessionId, actor_id: 'actor-1' }));
      const result = await proj.list();
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(sessionId);
      expect(result.value[0]!.status).toBe('started');
      expect(result.value[0]!.actor_id).toBe('actor-1');
    });
  });

  it('should transition: started -> completed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'session.started', entity_id: sessionId, actor_id: 'a1' }));
      await proj.applyEvent(makeEvent({ event_type: 'session.completed', entity_id: sessionId }));
      expect((await proj.list()).value[0]!.status).toBe('completed');
    });
  });

  it('should transition: started -> failed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new SessionProjectionRepository(drizzle(state.storage, { schema: workspace_schema }));
      const sessionId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: 'session.started', entity_id: sessionId, actor_id: 'a1' }));
      await proj.applyEvent(makeEvent({ event_type: 'session.failed', entity_id: sessionId }));
      expect((await proj.list()).value[0]!.status).toBe('failed');
    });
  });
});

describe('Workspace SyncFacade integration', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('push should store events AND update projections', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const db = drizzle(state.storage, { schema: workspace_schema });
      const eventRepo = new EventRepository(db);
      const actorProj = new ActorProjectionRepository(db);
      const sessionProj = new SessionProjectionRepository(db);

      const [actorId, sessionId] = [crypto.randomUUID(), crypto.randomUUID()];
      const events = [
        makeEvent({ event_type: 'actor.created', entity_id: actorId, payload_json: JSON.stringify({ name: 'agent-1' }) }),
        makeEvent({ event_type: 'session.started', entity_id: sessionId, actor_id: actorId }),
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
