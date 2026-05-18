-- Cascade-delete dependents when a user row is removed. Better Auth's
-- /delete-user endpoint calls internalAdapter.deleteUser(id) directly; with
-- ON DELETE NO ACTION (the previous default) that left orphan rows in
-- session, account, apikey. SQLite cannot ALTER a foreign key in place, so
-- each table is recreated with the new constraint and data is copied across.
--
-- subscription.referenceId is intentionally NOT made an FK here: the column
-- is plain text (Better Auth's Stripe plugin keeps it generic so it can hold
-- an organization id when that plugin is in use). The deleteUser hook in
-- src/lib/auth.ts deletes the Stripe customer before the user row goes, and
-- the subscription rows live downstream of Stripe — cleaning them up belongs
-- to the Stripe webhook, not this cascade.

CREATE TABLE `__new_session` (
	`id` text PRIMARY KEY NOT NULL,
	`expiresAt` integer NOT NULL,
	`token` text NOT NULL,
	`createdAt` integer NOT NULL,
	`updatedAt` integer NOT NULL,
	`ipAddress` text,
	`userAgent` text,
	`userId` text NOT NULL,
	FOREIGN KEY (`userId`) REFERENCES `user`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_session` SELECT `id`, `expiresAt`, `token`, `createdAt`, `updatedAt`, `ipAddress`, `userAgent`, `userId` FROM `session`;
--> statement-breakpoint
DROP TABLE `session`;
--> statement-breakpoint
ALTER TABLE `__new_session` RENAME TO `session`;
--> statement-breakpoint
CREATE UNIQUE INDEX `session_token_unique` ON `session` (`token`);
--> statement-breakpoint

CREATE TABLE `__new_account` (
	`id` text PRIMARY KEY NOT NULL,
	`accountId` text NOT NULL,
	`providerId` text NOT NULL,
	`userId` text NOT NULL,
	`accessToken` text,
	`refreshToken` text,
	`idToken` text,
	`accessTokenExpiresAt` integer,
	`refreshTokenExpiresAt` integer,
	`scope` text,
	`password` text,
	`createdAt` integer NOT NULL,
	`updatedAt` integer NOT NULL,
	FOREIGN KEY (`userId`) REFERENCES `user`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_account` SELECT `id`, `accountId`, `providerId`, `userId`, `accessToken`, `refreshToken`, `idToken`, `accessTokenExpiresAt`, `refreshTokenExpiresAt`, `scope`, `password`, `createdAt`, `updatedAt` FROM `account`;
--> statement-breakpoint
DROP TABLE `account`;
--> statement-breakpoint
ALTER TABLE `__new_account` RENAME TO `account`;
--> statement-breakpoint

CREATE TABLE `__new_apikey` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text,
	`start` text,
	`prefix` text,
	`key` text NOT NULL,
	`userId` text NOT NULL,
	`refillInterval` integer,
	`refillAmount` integer,
	`lastRefillAt` integer,
	`enabled` integer,
	`rateLimitEnabled` integer,
	`rateLimitTimeWindow` integer,
	`rateLimitMax` integer,
	`requestCount` integer,
	`remaining` integer,
	`lastRequest` integer,
	`expiresAt` integer,
	`createdAt` integer NOT NULL,
	`updatedAt` integer NOT NULL,
	`permissions` text,
	`metadata` text,
	FOREIGN KEY (`userId`) REFERENCES `user`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_apikey` SELECT `id`, `name`, `start`, `prefix`, `key`, `userId`, `refillInterval`, `refillAmount`, `lastRefillAt`, `enabled`, `rateLimitEnabled`, `rateLimitTimeWindow`, `rateLimitMax`, `requestCount`, `remaining`, `lastRequest`, `expiresAt`, `createdAt`, `updatedAt`, `permissions`, `metadata` FROM `apikey`;
--> statement-breakpoint
DROP TABLE `apikey`;
--> statement-breakpoint
ALTER TABLE `__new_apikey` RENAME TO `apikey`;
--> statement-breakpoint
CREATE INDEX `apikey_user` ON `apikey` (`userId`);
