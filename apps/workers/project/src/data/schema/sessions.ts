import { sqliteTable, text } from 'drizzle-orm/sqlite-core';

/** Projection table built from session lifecycle events */
export const sessions = sqliteTable('sessions', {
  id: text('id').primaryKey(),
  actor_id: text('actor_id'),
  status: text('status').notNull().default('started'), // started | completed | failed | cancelled
  created_at: text('created_at').notNull(),
  updated_at: text('updated_at').notNull(),
});
