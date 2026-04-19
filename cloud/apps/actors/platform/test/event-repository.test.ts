import { describe, it, expect, beforeEach } from 'vitest';
import { runInDurableObject, drizzle, platform_schema, makeEvent, createMigratedStub } from './helpers.js';
import { EventRepository } from '../src/repositories/event.repository.js';

describe('Platform EventRepository', () => {
  let stub: DurableObjectStub;

  beforeEach(async () => {
    stub = await createMigratedStub();
  });

  describe('push', () => {
    it('should accept and store events', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const result = await repo.push([makeEvent(), makeEvent()]);
        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(2);
      });
    });

    it('should deduplicate by event ID', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const event = makeEvent();
        await repo.push([event]);
        const result = await repo.push([event]);
        expect(result.value).toBe(0);
        expect((await repo.pull()).value.events).toHaveLength(1);
      });
    });

    it('should deduplicate within a single batch', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const event = makeEvent();
        const result = await repo.push([event, event]);
        expect(result.value).toBe(1);
      });
    });

    it('should accept empty array', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const result = await repo.push([]);
        expect(result.isSuccess).toBe(true);
        expect(result.value).toBe(0);
      });
    });

    it('should preserve all optional fields on round-trip', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const event = makeEvent({
          causation_id: 'cause-1',
          workspace_id: 'ws-1', session_id: 'sess-1',
          actor_id: 'actor-1', parent_actor_id: 'parent-1',
          target_actor_id: 'target-1', elevated: true,
        });
        await repo.push([event]);
        const stored = (await repo.pull()).value.events[0]!;

        expect(stored.id).toBe(event.id);
        expect(stored.causation_id).toBe('cause-1');
        expect(stored.target_actor_id).toBe('target-1');
        expect(stored.workspace_id).toBe('ws-1');
        expect(stored.session_id).toBe('sess-1');
        expect(stored.actor_id).toBe('actor-1');
        expect(stored.parent_actor_id).toBe('parent-1');
        expect(stored.elevated).toBe(true);
      });
    });
  });

  describe('pull', () => {
    it('should return empty from empty store', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const result = await repo.pull();
        expect(result.value.events).toHaveLength(0);
        expect(result.value.cursor).toBeNull();
      });
    });

    it('should return all events when no cursor', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        await repo.push([makeEvent(), makeEvent(), makeEvent()]);
        const result = await repo.pull();
        expect(result.value.events).toHaveLength(3);
        expect(result.value.cursor).not.toBeNull();
      });
    });

    it('should return events after cursor', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const [first, second, third] = [makeEvent(), makeEvent(), makeEvent()];
        await repo.push([first, second, third]);
        const result = await repo.pull(first.id);
        expect(result.value.events).toHaveLength(2);
        expect(result.value.events[0]!.id).toBe(second.id);
        expect(result.value.events[1]!.id).toBe(third.id);
      });
    });

    it('should return empty when cursor is the last event', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const event = makeEvent();
        await repo.push([event]);
        const result = await repo.pull(event.id);
        expect(result.value.events).toHaveLength(0);
        expect(result.value.cursor).toBeNull();
      });
    });

    it('should return all events when cursor ID is unknown', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        await repo.push([makeEvent(), makeEvent()]);
        const result = await repo.pull('nonexistent-id');
        expect(result.value.events).toHaveLength(2);
      });
    });

    it('should respect limit', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        await repo.push(Array.from({ length: 5 }, () => makeEvent()));
        const result = await repo.pull(undefined, 2);
        expect(result.value.events).toHaveLength(2);
      });
    });

    it('should maintain insertion order across multiple pushes', async () => {
      await runInDurableObject(stub, async (_: any, state: any) => {
        const repo = new EventRepository(drizzle(state.storage, { schema: platform_schema }));
        const [a, b, c] = [makeEvent(), makeEvent(), makeEvent()];
        await repo.push([a]);
        await repo.push([b]);
        await repo.push([c]);
        expect((await repo.pull()).value.events.map((e) => e.id)).toEqual([a.id, b.id, c.id]);
      });
    });
  });
});
