CREATE TABLE `members` (
  `user_id` text PRIMARY KEY NOT NULL,
  `role` text NOT NULL DEFAULT 'owner',
  `created_at` text NOT NULL
);
