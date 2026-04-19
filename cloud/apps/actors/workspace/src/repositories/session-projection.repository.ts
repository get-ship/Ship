import { eq } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import { WORKSPACE_EVENT_TYPES } from '@ship-cloud/types';
import { sessions } from '../data/schema/sessions.js';
import type { workspace_schema } from '../data/schema/index.js';

export class SessionProjectionRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof workspace_schema>) {}

  async applyEvent(event: EventEnvelope): Promise<Result<void>> {
    try {
      const now = new Date().toISOString();

      switch (event.event_type) {
        case WORKSPACE_EVENT_TYPES.SESSION_STARTED: {
          await this.db.insert(sessions).values({
            id: event.entity_id,
            actor_id: event.actor_id ?? null,
            status: 'started',
            created_at: event.created_at,
            updated_at: event.created_at,
          }).onConflictDoNothing();
          break;
        }
        case WORKSPACE_EVENT_TYPES.SESSION_COMPLETED:
          await this.db.update(sessions)
            .set({ status: 'completed', updated_at: now })
            .where(eq(sessions.id, event.entity_id));
          break;
        case WORKSPACE_EVENT_TYPES.SESSION_FAILED:
          await this.db.update(sessions)
            .set({ status: 'failed', updated_at: now })
            .where(eq(sessions.id, event.entity_id));
          break;
      }

      return Result.ok();
    } catch (error) {
      return Result.fail(
        new Error(`Failed to apply session event: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async list(): Promise<Result<typeof sessions.$inferSelect[]>> {
    try {
      const rows = await this.db.select().from(sessions);
      return Result.ok(rows);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to list sessions: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }
}
