CREATE TABLE `events` (
  `id` text PRIMARY KEY NOT NULL,
  `event_type` text NOT NULL,
  `entity_id` text NOT NULL,
  `actor` text NOT NULL,
  `payload_json` text NOT NULL,
  `version` integer NOT NULL,
  `causation_id` text,
  `workspace_id` text,
  `session_id` text,
  `actor_id` text,
  `parent_actor_id` text,
  `target_actor_id` text,
  `elevated` integer NOT NULL,
  `created_at` text NOT NULL,
  `cursor` integer NOT NULL
);

CREATE TABLE `actors` (
  `id` text PRIMARY KEY NOT NULL,
  `name` text,
  `status` text NOT NULL DEFAULT 'created',
  `created_at` text NOT NULL,
  `updated_at` text NOT NULL
);

CREATE TABLE `sessions` (
  `id` text PRIMARY KEY NOT NULL,
  `actor_id` text,
  `status` text NOT NULL DEFAULT 'started',
  `created_at` text NOT NULL,
  `updated_at` text NOT NULL
);
