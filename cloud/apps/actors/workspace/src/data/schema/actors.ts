import { sqliteTable, text } from 'drizzle-orm/sqlite-core';

/** Projection table built from actor lifecycle events */
export const actors = sqliteTable('actors', {
  id: text('id').primaryKey(),
  name: text('name'),
  status: text('status').notNull().default('created'), // created | woke | slept | stopped | crashed
  created_at: text('created_at').notNull(),
  updated_at: text('updated_at').notNull(),
});
