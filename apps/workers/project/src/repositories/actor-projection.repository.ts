import { eq } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import {
  ACTOR_CREATED,
  ACTOR_WOKE,
  ACTOR_SLEPT,
  ACTOR_STOPPED,
  ACTOR_CRASHED,
} from '@ship-cloud/types';
import { actors } from '../data/schema/actors.js';
import type { workspace_schema } from '../data/schema/index.js';

export class ActorProjectionRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof workspace_schema>) {}

  async applyEvent(event: EventEnvelope): Promise<Result<void>> {
    try {
      const now = new Date().toISOString();

      switch (event.event_type) {
        case ACTOR_CREATED: {
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
        case ACTOR_WOKE:
          await this.db.update(actors)
            .set({ status: 'woke', updated_at: now })
            .where(eq(actors.id, event.entity_id));
          break;
        case ACTOR_SLEPT:
          await this.db.update(actors)
            .set({ status: 'slept', updated_at: now })
            .where(eq(actors.id, event.entity_id));
          break;
        case ACTOR_STOPPED:
          await this.db.update(actors)
            .set({ status: 'stopped', updated_at: now })
            .where(eq(actors.id, event.entity_id));
          break;
        case ACTOR_CRASHED:
          await this.db.update(actors)
            .set({ status: 'crashed', updated_at: now })
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
