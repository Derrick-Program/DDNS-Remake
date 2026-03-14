-- Your SQL goes here
CREATE TABLE `users`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`username` TEXT NOT NULL,
	`password_hash` TEXT NOT NULL
);

CREATE TABLE `devices`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`user_id` INTEGER NOT NULL,
	`device_identifier` TEXT NOT NULL,
	`token_hash` TEXT NOT NULL,
	`last_seen_ip` TEXT,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
);

CREATE TABLE `domains`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`device_id` INTEGER NOT NULL,
	`hostname` TEXT NOT NULL,
	`current_ip` TEXT,
	`is_active` BOOL NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`device_id`) REFERENCES `devices`(`id`)
);

