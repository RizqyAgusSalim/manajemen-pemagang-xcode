-- Migration: Create project_proposals table for title submission feature
-- This table stores project title proposals submitted by interns

CREATE TABLE IF NOT EXISTS `project_proposals` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `judul_project` varchar(255) NOT NULL,
  `deskripsi_project` text DEFAULT NULL,
  `catatan_mahasiswa` text DEFAULT NULL,
  `status` enum('pending','approved','rejected','revised') DEFAULT 'pending',
  `tanggal_pengajuan` timestamp NOT NULL DEFAULT current_timestamp(),
  `catatan_reviewer` text DEFAULT NULL,
  `reviewed_by` char(36) DEFAULT NULL,
  `reviewed_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `fk_proposals_intern` (`intern_id`),
  KEY `fk_proposals_reviewer` (`reviewed_by`),
  KEY `idx_status` (`status`),
  KEY `idx_tanggal_pengajuan` (`tanggal_pengajuan`),
  CONSTRAINT `fk_proposals_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_proposals_reviewer` FOREIGN KEY (`reviewed_by`) REFERENCES `users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
