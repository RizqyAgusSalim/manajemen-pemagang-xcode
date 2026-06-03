-- Migration: Alter project_proposals status column to include 'revised'
-- This fixes the bug where "Perlu Revisi (Revised)" caused a Database Error
-- because the enum was originally created without 'revised' in 001_intern_db.sql.

ALTER TABLE `project_proposals` 
MODIFY COLUMN `status` enum('pending','approved','rejected','revised') NOT NULL DEFAULT 'pending';
