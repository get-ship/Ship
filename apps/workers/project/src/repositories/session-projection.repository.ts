import { eq } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import { sessions } from '../data/schema/sessions.js';
import type { project_schema } from '../data/schema/index.js';

// Canonical session event types (#26 spawn-intent: agent.session.* retired).
// Using string literals — these match the canonical session.* family registered
// in agent-def/src/event_registrations.rs.
const SESSION_STARTED = 'session.started';
const SESSION_COMPLETED = 'session.completed';
const SESSION_FAILED = 'session.failed';
const SESSION_CANCELLED = 'session.cancelled';

export class SessionProjectionRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof project_schema>) {}

  async applyEvent(event: EventEnvelope): Promise<Result<void>> {
    try {
      const now = new Date().toISOString();

      switch (event.event_type) {
        case SESSION_STARTED: {
          await this.db.insert(sessions).values({
            id: event.entity_id,
            actor_id: event.actor_id ?? null,
            status: 'started',
            created_at: event.created_at,
            updated_at: event.created_at,
          }).onConflictDoNothing();
          break;
        }
        case SESSION_COMPLETED:
          await this.db.update(sessions)
            .set({ status: 'completed', updated_at: now })
            .where(eq(sessions.id, event.entity_id));
          break;
        case SESSION_FAILED:
          await this.db.update(sessions)
            .set({ status: 'failed', updated_at: now })
            .where(eq(sessions.id, event.entity_id));
          break;
        case SESSION_CANCELLED:
          await this.db.update(sessions)
            .set({ status: 'cancelled', updated_at: now })
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
