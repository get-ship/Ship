import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

/** Projection table built from job lifecycle events */
export const jobs = sqliteTable('jobs', {
  id: text('id').primaryKey(),
  status: text('status').notNull().default('queued'), // queued | started | completed | failed
  workspace_id: text('workspace_id'),
  created_at: text('created_at').notNull(),
  updated_at: text('updated_at').notNull(),
});
