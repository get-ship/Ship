import { eq } from 'drizzle-orm';
import type { DrizzleSqliteDODatabase } from 'drizzle-orm/durable-sqlite';
import { Result } from '@ship-cloud/types';
import type { EventEnvelope } from '@ship-cloud/types';
import {
  WORKSPACE_CREATED,
  WORKSPACE_ACTIVATED,
  WORKSPACE_ARCHIVED,
  WORKSPACE_DELETED,
} from '@ship-cloud/types';
import { workspaces } from '../data/schema/workspaces.js';
import type { platform_schema } from '../data/schema/index.js';

export class WorkspaceProjectionRepository {
  constructor(private db: DrizzleSqliteDODatabase<typeof platform_schema>) {}

  async applyEvent(event: EventEnvelope): Promise<Result<void>> {
    try {
      const now = new Date().toISOString();

      switch (event.event_type) {
        case WORKSPACE_CREATED: {
          const payload = JSON.parse(event.payload_json);
          await this.db.insert(workspaces).values({
            id: event.entity_id,
            name: payload.name ?? null,
            status: 'created',
            created_at: event.created_at,
            updated_at: event.created_at,
          }).onConflictDoNothing();
          break;
        }
        case WORKSPACE_ACTIVATED:
          await this.db.update(workspaces)
            .set({ status: 'activated', updated_at: now })
            .where(eq(workspaces.id, event.entity_id));
          break;
        case WORKSPACE_ARCHIVED:
          await this.db.update(workspaces)
            .set({ status: 'archived', updated_at: now })
            .where(eq(workspaces.id, event.entity_id));
          break;
        case WORKSPACE_DELETED:
          await this.db.update(workspaces)
            .set({ status: 'deleted', updated_at: now })
            .where(eq(workspaces.id, event.entity_id));
          break;
      }

      return Result.ok();
    } catch (error) {
      return Result.fail(
        new Error(`Failed to apply workspace event: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }

  async list(): Promise<Result<typeof workspaces.$inferSelect[]>> {
    try {
      const rows = await this.db.select().from(workspaces);
      return Result.ok(rows);
    } catch (error) {
      return Result.fail(
        new Error(`Failed to list workspaces: ${error instanceof Error ? error.message : String(error)}`),
      );
    }
  }
}
