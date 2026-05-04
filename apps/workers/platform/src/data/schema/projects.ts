import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

/** Projection table built from project lifecycle events */
export const projects = sqliteTable('projects', {
  id: text('id').primaryKey(),
  name: text('name'),
  status: text('status').notNull().default('created'), // created | activated | archived | deleted
  created_at: text('created_at').notNull(),
  updated_at: text('updated_at').notNull(),
});
