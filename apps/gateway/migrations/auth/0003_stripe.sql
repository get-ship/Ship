ALTER TABLE `user` ADD COLUMN `stripeCustomerId` text;
--> statement-breakpoint
CREATE TABLE `subscription` (
	`id` text PRIMARY KEY NOT NULL,
	`plan` text NOT NULL,
	`referenceId` text NOT NULL,
	`stripeCustomerId` text,
	`stripeSubscriptionId` text,
	`status` text NOT NULL,
	`periodStart` integer,
	`periodEnd` integer,
	`cancelAtPeriodEnd` integer,
	`cancelAt` integer,
	`canceledAt` integer,
	`endedAt` integer,
	`seats` integer,
	`trialStart` integer,
	`trialEnd` integer,
	`billingInterval` text,
	`stripeScheduleId` text
);
--> statement-breakpoint
CREATE INDEX `subscription_reference_id` ON `subscription` (`referenceId`);
