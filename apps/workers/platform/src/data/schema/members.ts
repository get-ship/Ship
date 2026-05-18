import { sqliteTable, text } from 'drizzle-orm/sqlite-core';

/**
 * Membership for the project this DO instance represents.
 * Project id is implicit (one Platform DO instance per project), so we only
 * need user_id + role here. First authenticated caller to a brand-new DO
 * becomes the sole 'owner'; subsequent calls verify membership before any
 * sync/projects/jobs operation runs.
 */
export const members = sqliteTable('members', {
  user_id: text('user_id').primaryKey(),
  role: text('role').notNull().default('owner'),
  created_at: text('created_at').notNull(),
});
