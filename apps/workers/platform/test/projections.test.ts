import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, platform_schema, makeEvent, createMigratedStub } from './helpers.js';
import { EventRepository } from '../src/repositories/event.repository.js';
import { ProjectProjectionRepository } from '../src/repositories/project-projection.repository.js';
import { JobProjectionRepository } from '../src/repositories/job-projection.repository.js';
import { PIPELINE_RUN_STARTED, PIPELINE_RUN_DISPATCHED, PIPELINE_RUN_COMPLETED, PIPELINE_RUN_FAILED } from '@ship-cloud/types';

describe('Platform ProjectProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create project from event', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ProjectProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const wsId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({
        event_type: 'workspace.created', entity_id: wsId,
        payload_json: JSON.stringify({ name: 'my-workspace' }),
      }));
      const result = await proj.list();
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.id).toBe(wsId);
      expect(result.value[0]!.name).toBe('my-workspace');
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition: created -> activated -> archived -> deleted', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ProjectProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const wsId = crypto.randomUUID();

      await proj.applyEvent(makeEvent({ event_type: 'workspace.created', entity_id: wsId, payload_json: JSON.stringify({ name: 'ws' }) }));
      expect((await proj.list()).value[0]!.status).toBe('created');

      await proj.applyEvent(makeEvent({ event_type: 'workspace.activated', entity_id: wsId }));
      expect((await proj.list()).value[0]!.status).toBe('activated');

      await proj.applyEvent(makeEvent({ event_type: 'workspace.archived', entity_id: wsId }));
      expect((await proj.list()).value[0]!.status).toBe('archived');

      await proj.applyEvent(makeEvent({ event_type: 'workspace.deleted', entity_id: wsId }));
      expect((await proj.list()).value[0]!.status).toBe('deleted');
    });
  });

  it('should ignore duplicate created events (idempotent)', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ProjectProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const wsId = crypto.randomUUID();
      const event = makeEvent({ event_type: 'workspace.created', entity_id: wsId, payload_json: JSON.stringify({ name: 'ws' }) });
      await proj.applyEvent(event);
      await proj.applyEvent(event);
      expect((await proj.list()).value).toHaveLength(1);
    });
  });

  it('should track multiple projects independently', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ProjectProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const [ws1, ws2] = [crypto.randomUUID(), crypto.randomUUID()];
      await proj.applyEvent(makeEvent({ event_type: 'workspace.created', entity_id: ws1, payload_json: JSON.stringify({ name: 'alpha' }) }));
      await proj.applyEvent(makeEvent({ event_type: 'workspace.created', entity_id: ws2, payload_json: JSON.stringify({ name: 'beta' }) }));
      await proj.applyEvent(makeEvent({ event_type: 'workspace.activated', entity_id: ws1 }));
      const result = await proj.list();
      expect(result.value).toHaveLength(2);
      expect(result.value.find((w) => w.id === ws1)!.status).toBe('activated');
      expect(result.value.find((w) => w.id === ws2)!.status).toBe('created');
    });
  });

  it('should ignore unrelated event types', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new ProjectProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: crypto.randomUUID() }));
      expect((await proj.list()).value).toHaveLength(0);
    });
  });
});

describe('Platform JobProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create job from created event', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new JobProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const jobId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId }));
      const result = await proj.list();
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition: created -> dispatched -> completed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new JobProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const jobId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId }));
      expect((await proj.list()).value[0]!.status).toBe('created');
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_DISPATCHED, entity_id: jobId }));
      expect((await proj.list()).value[0]!.status).toBe('dispatched');
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_COMPLETED, entity_id: jobId }));
      expect((await proj.list()).value[0]!.status).toBe('completed');
    });
  });

  it('should transition: created -> dispatched -> failed', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const proj = new JobProjectionRepository(drizzle(state.storage, { schema: platform_schema }));
      const jobId = crypto.randomUUID();
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId }));
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_DISPATCHED, entity_id: jobId }));
      await proj.applyEvent(makeEvent({ event_type: PIPELINE_RUN_FAILED, entity_id: jobId }));
      expect((await proj.list()).value[0]!.status).toBe('failed');
    });
  });
});

describe('Platform SyncFacade integration', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('push should store events AND update projections', async () => {
    await runInDurableObject(stub, async (_: any, state: any) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const eventRepo = new EventRepository(db);
      const projectProj = new ProjectProjectionRepository(db);
      const jobProj = new JobProjectionRepository(db);

      const [wsId, jobId] = [crypto.randomUUID(), crypto.randomUUID()];
      const events = [
        makeEvent({ event_type: 'workspace.created', entity_id: wsId, payload_json: JSON.stringify({ name: 'ws-1' }) }),
        makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId, space_id: wsId }),
      ];

      expect((await eventRepo.push(events)).value).toBe(2);
      for (const event of events) {
        await projectProj.applyEvent(event);
        await jobProj.applyEvent(event);
      }

      expect((await projectProj.list()).value).toHaveLength(1);
      expect((await jobProj.list()).value[0]!.status).toBe('created');
      expect((await eventRepo.pull()).value.events).toHaveLength(2);
    });
  });
});
