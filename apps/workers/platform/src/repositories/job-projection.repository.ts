import { eq } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import {
  PIPELINE_RUN_STARTED,
  PIPELINE_RUN_DISPATCHED,
  PIPELINE_RUN_COMPLETED,
  PIPELINE_RUN_FAILED,
} from '@ship-cloud/types';
import { jobs } from '../data/schema/jobs.js';
import type { platform_schema } from '../data/schema/index.js';

export class JobProjectionRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof platform_schema>) {}

  async applyEvent(event: EventEnvelope): Promise<Result<void>> {
    try {
      const now = new Date().toISOString();

      switch (event.event_type) {
        case PIPELINE_RUN_STARTED: {
          await this.db.insert(jobs).values({
            id: event.entity_id,
            status: 'created',
            workspace_id: event.space_id ?? null,
            created_at: event.created_at,
            updated_at: event.created_at,
          }).onConflictDoNothing();
          break;
        }
        case PIPELINE_RUN_DISPATCHED:
          await this.db.update(jobs)
            .set({ status: 'dispatched', updated_at: now })
            .where(eq(jobs.id, event.entity_id));
          break;
        case PIPELINE_RUN_COMPLETED:
          await this.db.update(jobs)
            .set({ status: 'completed', updated_at: now })
            .where(eq(jobs.id, event.entity_id));
          break;
        case PIPELINE_RUN_FAILED:
          await this.db.update(jobs)
            .set({ status: 'failed', updated_at: now })
            .where(eq(jobs.id, event.entity_id));
          break;
      }

      return Result.ok();
    } catch (error) {
      return Result.fail(
        new Error(`Failed to apply job event: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async list(): Promise<Result<typeof jobs.$inferSelect[]>> {
    try {
      const rows = await this.db.select().from(jobs);
      return Result.ok(rows);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to list jobs: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }
}
