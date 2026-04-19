import { eq, gt, sql } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import { events } from '../data/schema/events.js';
import type { workspace_schema } from '../data/schema/index.js';

export class EventRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof workspace_schema>) {}

  async push(incoming: EventEnvelope[]): Promise<Result<number>> {
    try {
      let accepted = 0;

      for (const event of incoming) {
        const existing = await this.db
          .select({ id: events.id })
          .from(events)
          .where(eq(events.id, event.id))
          .limit(1);

        if (existing.length > 0) continue;

        const maxCursor = await this.db
          .select({ max: sql<number>`COALESCE(MAX(${events.cursor}), 0)` })
          .from(events);
        const nextCursor = (maxCursor[0]?.max ?? 0) + 1;

        await this.db.insert(events).values({
          id: event.id,
          event_type: event.event_type,
          entity_id: event.entity_id,
          actor: event.actor,
          payload_json: event.payload_json,
          version: event.version,
          correlation_id: event.correlation_id ?? null,
          causation_id: event.causation_id ?? null,
          workspace_id: event.workspace_id ?? null,
          session_id: event.session_id ?? null,
          actor_id: event.actor_id ?? null,
          parent_actor_id: event.parent_actor_id ?? null,
          elevated: event.elevated,
          created_at: event.created_at,
          cursor: nextCursor,
        });

        accepted++;
      }

      return Result.ok(accepted);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to push events: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async pull(since?: string, limit = 100): Promise<Result<{ events: EventEnvelope[]; cursor: string | null }>> {
    try {
      let afterCursor: number | undefined;
      if (since) {
        const sinceEvent = await this.db
          .select({ cursor: events.cursor })
          .from(events)
          .where(eq(events.id, since))
          .limit(1);

        if (sinceEvent.length > 0) {
          afterCursor = sinceEvent[0]!.cursor;
        }
      }

      const rows = afterCursor !== undefined
        ? await this.db.select().from(events).where(gt(events.cursor, afterCursor)).orderBy(events.cursor).limit(limit)
        : await this.db.select().from(events).orderBy(events.cursor).limit(limit);

      const envelopes: EventEnvelope[] = rows.map((row) => ({
        id: row.id,
        event_type: row.event_type,
        entity_id: row.entity_id,
        actor: row.actor,
        payload_json: row.payload_json,
        version: row.version,
        correlation_id: row.correlation_id ?? undefined,
        causation_id: row.causation_id ?? undefined,
        workspace_id: row.workspace_id ?? undefined,
        session_id: row.session_id ?? undefined,
        actor_id: row.actor_id ?? undefined,
        parent_actor_id: row.parent_actor_id ?? undefined,
        elevated: row.elevated,
        created_at: row.created_at,
      }));

      const lastEvent = envelopes[envelopes.length - 1];
      const cursor = lastEvent?.id ?? null;

      return Result.ok({ events: envelopes, cursor });
    } catch (error) {
      return Result.fail(
        new Error(`Failed to pull events: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }
}
