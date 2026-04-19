import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, workspace_schema, makeEvent, createMigratedStub } from './helpers.js';
import { EventRepository } from '../src/repositories/event.repository.js';

describe('Workspace EventRepository', () => {
  let stub: DurableObjectStub;
  beforeEach(async () => { stub = await createMigratedStub(); });

  describe('push', () => {
    it('should accept and store events', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        const result = await repo.push([makeEvent(), makeEvent()]);
        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(2);
      });
    });

    it('should deduplicate by event ID', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        const event = makeEvent();
        await repo.push([event]);
        expect((await repo.push([event])).value).toBe(0);
      });
    });

    it('should accept empty array', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        const result = await repo.push([]);
        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(0);
      });
    });
  });

  describe('pull', () => {
    it('should return empty from empty store', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        const result = await repo.pull();
        expect(result.value.events).toHaveLength(0);
        expect(result.value.cursor).toBeNull();
      });
    });

    it('should return all events when no cursor', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        await repo.push([makeEvent(), makeEvent(), makeEvent()]);
        const result = await repo.pull();
        expect(result.value.events).toHaveLength(3);
        expect(result.value.cursor).not.toBeNull();
      });
    });

    it('should return events after cursor', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        const [first, second] = [makeEvent(), makeEvent()];
        await repo.push([first, second]);
        const result = await repo.pull(first.id);
        expect(result.value.events).toHaveLength(1);
        expect(result.value.events[0]!.id).toBe(second.id);
      });
    });

    it('should respect limit', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        await repo.push(Array.from({ length: 5 }, () => makeEvent()));
        expect((await repo.pull(undefined, 2)).value.events).toHaveLength(2);
      });
    });

    it('should maintain insertion order', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: workspace_schema }));
        const [a, b, c] = [makeEvent(), makeEvent(), makeEvent()];
        await repo.push([a]);
        await repo.push([b]);
        await repo.push([c]);
        expect((await repo.pull()).value.events.map((e) => e.id)).toEqual([a.id, b.id, c.id]);
      });
    });
  });
});
