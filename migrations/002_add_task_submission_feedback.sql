-- Migration: add missing task submission and feedback columns
-- This fixes runtime errors for tasks when the current database schema is outdated.

ALTER TABLE `tasks`
  ADD COLUMN IF NOT EXISTS `submission_file` varchar(255) DEFAULT NULL,
  ADD COLUMN IF NOT EXISTS `feedback` text DEFAULT NULL;
