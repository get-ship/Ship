import { eq } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import { WORKSPACE_EVENT_TYPES } from '@ship-cloud/types';
import { actors } from '../data/schema/actors.js';
import type { workspace_schema } from '../data/schema/index.js';

export class ActorProjectionRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof workspace_schema>) {}

  async applyEvent(event: EventEnvelope): Promise<Result<void>> {
    try {
      const now = new Date().toISOString();

      switch (event.event_type) {
        case WORKSPACE_EVENT_TYPES.ACTOR_CREATED: {
          const payload = JSON.parse(event.payload_json);
          await this.db.insert(actors).values({
            id: event.entity_id,
            name: payload.name ?? null,
            status: 'created',
            created_at: event.created_at,
            updated_at: event.created_at,
          }).onConflictDoNothing();
          break;
        }
        case WORKSPACE_EVENT_TYPES.ACTOR_UPDATED: {
          const payload = JSON.parse(event.payload_json);
          await this.db.update(actors)
            .set({ name: payload.name ?? undefined, status: 'updated', updated_at: now })
            .where(eq(actors.id, event.entity_id));
          break;
        }
        case WORKSPACE_EVENT_TYPES.ACTOR_DELETED:
          await this.db.update(actors)
            .set({ status: 'deleted', updated_at: now })
            .where(eq(actors.id, event.entity_id));
          break;
      }

      return Result.ok();
    } catch (error) {
      return Result.fail(
        new Error(`Failed to apply actor event: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async list(): Promise<Result<typeof actors.$inferSelect[]>> {
    try {
      const rows = await this.db.select().from(actors);
      return Result.ok(rows);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to list actors: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }
}
