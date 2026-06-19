import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, platform_schema, makeEvent, createMigratedStub } from './helpers.js';
import { EventRepository } from '../src/repositories/event.repository.js';
import { JobProjectionRepository } from '../src/repositories/job-projection.repository.js';
import { PIPELINE_RUN_STARTED, PIPELINE_RUN_DISPATCHED, PIPELINE_RUN_COMPLETED, PIPELINE_RUN_FAILED } from '@ship-cloud/types';

describe('Platform JobProjection', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('should create job from created event', async () => {
    await runInDurableObject(stub, async (_: unknown, state: { storage: DurableObjectStorage }) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({
        event_type: PIPELINE_RUN_STARTED, entity_id: jobId,
      }));

      const result = await projection.list();
      expect(result.isSuccess).toBe(true);
      expect(result.value).toHaveLength(1);
      expect(result.value[0]!.status).toBe('created');
    });
  });

  it('should transition: created -> dispatched -> completed', async () => {
    await runInDurableObject(stub, async (_: unknown, state: { storage: DurableObjectStorage }) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId }));
      expect((await projection.list()).value[0]!.status).toBe('created');

      await projection.applyEvent(makeEvent({ event_type: PIPELINE_RUN_DISPATCHED, entity_id: jobId }));
      expect((await projection.list()).value[0]!.status).toBe('dispatched');

      await projection.applyEvent(makeEvent({ event_type: PIPELINE_RUN_COMPLETED, entity_id: jobId }));
      expect((await projection.list()).value[0]!.status).toBe('completed');
    });
  });

  it('should transition: created -> dispatched -> failed', async () => {
    await runInDurableObject(stub, async (_: unknown, state: { storage: DurableObjectStorage }) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const projection = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();
      await projection.applyEvent(makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId }));
      await projection.applyEvent(makeEvent({ event_type: PIPELINE_RUN_DISPATCHED, entity_id: jobId }));
      await projection.applyEvent(makeEvent({ event_type: PIPELINE_RUN_FAILED, entity_id: jobId }));

      expect((await projection.list()).value[0]!.status).toBe('failed');
    });
  });
});

describe('Platform SyncFacade integration', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  it('push should store events AND update projections', async () => {
    await runInDurableObject(stub, async (_: unknown, state: { storage: DurableObjectStorage }) => {
      const db = drizzle(state.storage, { schema: platform_schema });
      const eventRepo = new EventRepository(db);
      const jobProj = new JobProjectionRepository(db);

      const jobId = crypto.randomUUID();
      const events = [
        makeEvent({ event_type: PIPELINE_RUN_STARTED, entity_id: jobId }),
      ];

      expect((await eventRepo.push(events)).value).toBe(1);
      for (const event of events) {
        await jobProj.applyEvent(event);
      }

      expect((await jobProj.list()).value[0]!.status).toBe('created');
      expect((await eventRepo.pull()).value.events).toHaveLength(1);
    });
  });
});
