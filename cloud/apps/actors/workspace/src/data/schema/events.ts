import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

export const events = sqliteTable('events', {
  id: text('id').primaryKey(),                        // ULID
  event_type: text('event_type').notNull(),           // e.g. "actor.created"
  entity_id: text('entity_id').notNull(),
  actor: text('actor').notNull(),                     // always "ship" for now
  payload_json: text('payload_json').notNull(),       // JSON string
  version: integer('version').notNull(),              // always 1
  correlation_id: text('correlation_id'),
  causation_id: text('causation_id'),
  workspace_id: text('workspace_id'),
  session_id: text('session_id'),
  actor_id: text('actor_id'),
  parent_actor_id: text('parent_actor_id'),
  elevated: integer('elevated', { mode: 'boolean' }).notNull(),
  created_at: text('created_at').notNull(),           // ISO 8601
  cursor: integer('cursor').notNull(),
});
