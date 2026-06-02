-- phpMyAdmin SQL Dump
-- version 5.2.1
-- https://www.phpmyadmin.net/
--
-- Host: 127.0.0.1
-- Waktu pembuatan: 02 Jun 2026 pada 04.34
-- Versi server: 10.4.32-MariaDB
-- Versi PHP: 8.2.12

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `intern_db`
--

-- --------------------------------------------------------

--
-- Struktur dari tabel `attendances`
--

CREATE TABLE `attendances` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `date` date NOT NULL,
  `attendance_time` time DEFAULT NULL,
  `start_time` time DEFAULT NULL,
  `end_time` time DEFAULT NULL,
  `description` text DEFAULT NULL,
  `status` enum('Hadir','Izin','Alfa','Sakit','pending','approved','rejected') DEFAULT 'pending',
  `confirmed_by` char(36) DEFAULT NULL,
  `confirmed_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `attendances`
--

INSERT INTO `attendances` (`id`, `intern_id`, `date`, `attendance_time`, `start_time`, `end_time`, `description`, `status`, `confirmed_by`, `confirmed_at`, `created_at`) VALUES
('0193338f-40c6-46ed-adf6-508efe8f0015', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-05-12', '07:00:00', '07:00:00', '16:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:02:01', '2026-05-12 02:28:53'),
('31c87142-47e9-4f7d-81f2-8ecd9eebfe8d', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-05-18', '07:00:00', '07:00:00', '16:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:59', '2026-05-18 01:37:59'),
('76b310d3-d60d-445e-854c-1e11129ebc47', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-04-30', '07:00:00', '07:00:00', '16:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:40', '2026-04-30 07:56:12'),
('a177cde7-bf00-4cbc-a35d-61d79f34eb25', '30e98192-3ee7-11f1-b558-4ec2e12205b2', '2026-05-26', '16:47:00', '16:47:00', '00:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:47', '2026-05-26 09:47:15'),
('a61a5cc6-8d73-4b0f-8601-e1bb1f23d430', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-05-19', '08:00:00', '08:00:00', '16:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:56', '2026-05-19 01:19:18'),
('c0d25c13-d6a2-4cb0-85a3-dc022dfd2feb', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-05-20', '07:25:00', '07:25:00', '16:29:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:50', '2026-05-20 00:25:34'),
('cb6ca47f-f6b4-4983-b908-d346f0331a76', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-04-29', '07:00:00', '07:00:00', '16:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:36', '2026-04-29 05:52:41'),
('cd5d7be0-9319-4aad-9109-05c86d0ad85f', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-05-26', '07:00:00', '07:00:00', '17:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:01:44', '2026-05-26 13:56:25'),
('dc83dab5-39c9-4a78-9b74-2e38524f497d', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-04-27', '08:15:00', '08:15:00', '16:00:00', NULL, 'approved', '1a538468-edf1-4d27-b102-c44c05720415', '2026-05-26 14:02:04', '2026-04-27 01:27:24');

-- --------------------------------------------------------

--
-- Struktur dari tabel `divisions`
--

CREATE TABLE `divisions` (
  `id` int(11) NOT NULL,
  `name` varchar(100) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `divisions`
--

INSERT INTO `divisions` (`id`, `name`) VALUES
(1, 'Cyber Security Consultant'),
(4, 'Designer'),
(5, 'IT Network & Hardware'),
(6, 'Network Engineer'),
(2, 'Programmer (Front End / Back End)'),
(3, 'Public Relation');

-- --------------------------------------------------------

--
-- Struktur dari tabel `documents`
--

CREATE TABLE `documents` (
  `id` char(36) NOT NULL,
  `user_id` char(36) NOT NULL,
  `doc_type` enum('cv','report','certificate') NOT NULL,
  `file_path` varchar(500) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- --------------------------------------------------------

--
-- Struktur dari tabel `evaluations`
--

CREATE TABLE `evaluations` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `supervisor_id` char(36) NOT NULL,
  `discipline_score` int(11) DEFAULT 0,
  `performance_score` int(11) DEFAULT 0,
  `attitude_score` int(11) DEFAULT 0,
  `final_score` int(11) DEFAULT 0,
  `feedback` text DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `evaluations`
--

INSERT INTO `evaluations` (`id`, `intern_id`, `supervisor_id`, `discipline_score`, `performance_score`, `attitude_score`, `final_score`, `feedback`, `created_at`) VALUES
('0243dcd8-4928-4d24-ab5e-0cafb5260e05', '30e98192-3ee7-11f1-b558-4ec2e12205b2', '1a538468-edf1-4d27-b102-c44c05720415', 90, 90, 87, 89, 'Bagus', '2026-05-28 15:26:39'),
('644c3104-e976-4897-9fbf-265475c6a64d', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '1a538468-edf1-4d27-b102-c44c05720415', 80, 80, 70, 76, 'bagus', '2026-05-22 01:25:57'),
('88c31184-fc3a-4a11-b975-0af7cc677b1c', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '1a538468-edf1-4d27-b102-c44c05720415', 80, 80, 90, 83, NULL, '2026-05-22 01:47:26');

-- --------------------------------------------------------

--
-- Struktur dari tabel `interns`
--

CREATE TABLE `interns` (
  `id` char(36) NOT NULL,
  `user_id` char(36) NOT NULL,
  `university` varchar(150) DEFAULT NULL,
  `major` varchar(100) DEFAULT NULL,
  `start_date` date DEFAULT NULL,
  `end_date` date DEFAULT NULL,
  `status` enum('active','completed','on_leave') DEFAULT 'active',
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `nama_lengkap` varchar(255) DEFAULT NULL,
  `nim` varchar(50) DEFAULT NULL,
  `division` varchar(100) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `interns`
--

INSERT INTO `interns` (`id`, `user_id`, `university`, `major`, `start_date`, `end_date`, `status`, `created_at`, `nama_lengkap`, `nim`, `division`) VALUES
('29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'aa10b59e-946f-4514-8446-453f42e6007c', 'Politeknik Negeri Lampung', 'D3 Manajemen Informatika', '2026-02-16', '2026-06-05', 'active', '2026-04-23 07:36:33', 'Muhammad Jawir', '23753930', 'Programmer (Front End / Back End)'),
('30e98192-3ee7-11f1-b558-4ec2e12205b2', '33c92461-768b-4ec1-957f-5a7f325ef3c6', 'Politeknik Negeri Madiun', 'D3 Teknologi Rekayasa Cyber Security', '2026-05-01', '2026-05-30', 'active', '2026-04-23 07:36:45', 'Rizqy Agus Salim', '23753035', 'Cyber Security Consultant'),
('6d81f729-3a24-4c14-a9bc-3ff0665270e2', 'aafc8af4-c479-437f-a2f1-e2362bf4a706', 'Politeknik Negeri Lampung', 'D4 Teknologi Rekayasa Cyber Security', '2026-05-22', '2026-05-30', 'active', '2026-05-21 17:55:25', 'Nandito putra siagian', '123456789', 'Programmer (Front End / Back End)');

-- --------------------------------------------------------

--
-- Struktur dari tabel `intern_jadwal`
--

CREATE TABLE `intern_jadwal` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `jadwal_id` varchar(36) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `intern_jadwal`
--

INSERT INTO `intern_jadwal` (`id`, `intern_id`, `jadwal_id`, `created_at`) VALUES
('046d4368-59b9-4303-8a1c-2ced15667efa', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'fa4145a3-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:39'),
('08592955-0085-48cc-a5e4-311b534ef943', '30e98192-3ee7-11f1-b558-4ec2e12205b2', 'fa4145a3-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:42'),
('0df7b882-26bb-464f-9b65-9bca0f2b14c0', '6d81f729-3a24-4c14-a9bc-3ff0665270e2', 'fa414251-4759-11f1-b558-4ec2e12205b2', '2026-05-22 01:18:35'),
('270347a0-6180-48cc-bb6d-28d06755961b', '6d81f729-3a24-4c14-a9bc-3ff0665270e2', 'fa4147e9-4759-11f1-b558-4ec2e12205b2', '2026-05-22 01:18:49'),
('54588d84-1c54-43e3-83e2-b26d8c4393eb', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'fa4147e9-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:58'),
('75da14cd-6d72-44a5-b02f-1240f81fbfe2', '6d81f729-3a24-4c14-a9bc-3ff0665270e2', 'fa41453f-4759-11f1-b558-4ec2e12205b2', '2026-05-22 01:18:38'),
('7961c64d-0dc5-4831-a85a-172264916be2', '6d81f729-3a24-4c14-a9bc-3ff0665270e2', 'fa4145a3-4759-11f1-b558-4ec2e12205b2', '2026-05-22 01:18:41'),
('a5939c60-52c2-11f1-ac62-11e344aa86a3', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'fa40e764-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:05:38'),
('a5939e29-52c2-11f1-ac62-11e344aa86a3', '30e98192-3ee7-11f1-b558-4ec2e12205b2', 'fa40e764-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:05:38'),
('aa4b5dfd-a612-499a-a4fe-22f78f51d195', '6d81f729-3a24-4c14-a9bc-3ff0665270e2', 'fa414688-4759-11f1-b558-4ec2e12205b2', '2026-05-22 01:18:46'),
('bc362cf2-8497-442f-aa0e-ebb229b0f48c', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'fa41453f-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:28'),
('bce93972-8953-4b20-bb1a-0320b1f339dd', '30e98192-3ee7-11f1-b558-4ec2e12205b2', 'fa41453f-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:30'),
('ec071872-c9cf-4d96-beb8-e82579db7bac', '30e98192-3ee7-11f1-b558-4ec2e12205b2', 'fa4147e9-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:54:00'),
('f1d88507-d1d7-4409-a884-a50b860717ac', '30e98192-3ee7-11f1-b558-4ec2e12205b2', 'fa414688-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:52'),
('fefe8908-45f8-4d89-ba41-d61a2820e979', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'fa414688-4759-11f1-b558-4ec2e12205b2', '2026-05-18 14:53:49');

-- --------------------------------------------------------

--
-- Struktur dari tabel `jadwal_magang`
--

CREATE TABLE `jadwal_magang` (
  `id` varchar(36) NOT NULL,
  `hari` enum('Senin','Selasa','Rabu','Kamis','Jumat') NOT NULL,
  `kelompok` varchar(100) NOT NULL,
  `shift` enum('Pagi','Siang') NOT NULL DEFAULT 'Pagi',
  `keterangan` text DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `jadwal_magang`
--

INSERT INTO `jadwal_magang` (`id`, `hari`, `kelompok`, `shift`, `keterangan`, `created_at`) VALUES
('fa40e764-4759-11f1-b558-4ec2e12205b2', 'Senin', 'Kelompok A', 'Pagi', 'Senin Pagi 08.00 - 12.00', '2026-05-04 01:37:06'),
('fa414251-4759-11f1-b558-4ec2e12205b2', 'Senin', 'Kelompok B', 'Siang', 'Senin Siang 13.00 - 17.00', '2026-05-04 01:37:06'),
('fa4144b6-4759-11f1-b558-4ec2e12205b2', 'Selasa', 'Kelompok C', 'Pagi', 'Selasa Pagi 08.00 - 12.00', '2026-05-04 01:37:06'),
('fa41453f-4759-11f1-b558-4ec2e12205b2', 'Selasa', 'Kelompok D', 'Siang', 'Selasa Siang 13.00 - 17.00', '2026-05-04 01:37:06'),
('fa4145a3-4759-11f1-b558-4ec2e12205b2', 'Rabu', 'Kelompok E', 'Pagi', 'Rabu Pagi 08.00 - 12.00', '2026-05-04 01:37:06'),
('fa414601-4759-11f1-b558-4ec2e12205b2', 'Rabu', 'Kelompok F', 'Siang', 'Rabu Siang 13.00 - 17.00', '2026-05-04 01:37:06'),
('fa414688-4759-11f1-b558-4ec2e12205b2', 'Kamis', 'Kelompok G', 'Pagi', 'Kamis Pagi 08.00 - 12.00', '2026-05-04 01:37:06'),
('fa4146d7-4759-11f1-b558-4ec2e12205b2', 'Kamis', 'Kelompok H', 'Siang', 'Kamis Siang 13.00 - 17.00', '2026-05-04 01:37:06'),
('fa41478b-4759-11f1-b558-4ec2e12205b2', 'Jumat', 'Kelompok I', 'Pagi', 'Jumat Pagi 08.00 - 11.30', '2026-05-04 01:37:06'),
('fa4147e9-4759-11f1-b558-4ec2e12205b2', 'Jumat', 'Kelompok J', 'Siang', 'Jumat Siang 13.00 - 16.00', '2026-05-04 01:37:06');

-- --------------------------------------------------------

--
-- Struktur dari tabel `logbooks`
--

CREATE TABLE `logbooks` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `date` date NOT NULL,
  `activity` varchar(255) NOT NULL,
  `description` text DEFAULT NULL,
  `status` enum('draft','submitted','approved','rejected') DEFAULT 'draft',
  `supervisor_notes` text DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `logbooks`
--

INSERT INTO `logbooks` (`id`, `intern_id`, `date`, `activity`, `description`, `status`, `supervisor_notes`, `created_at`) VALUES
('43509bef-c596-4f53-8a46-7f6b374c3efa', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '2026-05-21', 'hehe', 'qwerfsad', 'approved', 'Kurang sempurna', '2026-05-21 08:20:04'),
('8c6bceed-a15d-4925-840c-16702198f8df', '30e98192-3ee7-11f1-b558-4ec2e12205b2', '2026-05-26', 'hehe', 'asdada', 'draft', NULL, '2026-05-26 09:46:59');

-- --------------------------------------------------------

--
-- Struktur dari tabel `password_resets`
--

CREATE TABLE `password_resets` (
  `id` char(36) NOT NULL,
  `email` varchar(255) NOT NULL,
  `code` varchar(6) NOT NULL,
  `expires_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Struktur dari tabel `project_proposals`
--

CREATE TABLE `project_proposals` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `judul_project` varchar(500) NOT NULL,
  `deskripsi_project` text DEFAULT NULL,
  `catatan_mahasiswa` text DEFAULT NULL,
  `status` enum('pending','approved','rejected') NOT NULL DEFAULT 'pending',
  `tanggal_pengajuan` date NOT NULL,
  `catatan_reviewer` text DEFAULT NULL,
  `reviewed_by` char(36) DEFAULT NULL,
  `reviewed_at` datetime DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT current_timestamp(),
  `updated_at` datetime NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `project_proposals`
--

INSERT INTO `project_proposals` (`id`, `intern_id`, `judul_project`, `deskripsi_project`, `catatan_mahasiswa`, `status`, `tanggal_pengajuan`, `catatan_reviewer`, `reviewed_by`, `reviewed_at`, `created_at`, `updated_at`) VALUES
('1d8bca08-cbbb-4f85-a53d-74a1cf6ffd34', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', 'Integrasi API Menggunakan Cpanel Berbasis Docker', '', '', 'pending', '2026-05-25', NULL, NULL, NULL, '2026-05-25 03:57:27', '2026-05-25 03:57:27'),
('5890cdbf-506f-48a3-80cb-83cbdca65b96', '30e98192-3ee7-11f1-b558-4ec2e12205b2', 'Hscking berbasis Mobile menggunakan HTML', 'caoiofandojfnkamo[', 'djbfsidf', '', '2026-05-26', 'sebaiknya\n', '1a538468-edf1-4d27-b102-c44c05720415', '2026-06-02 01:53:15', '2026-05-26 09:45:42', '2026-06-02 01:53:15');

-- --------------------------------------------------------

--
-- Struktur dari tabel `tasks`
--

CREATE TABLE `tasks` (
  `id` char(36) NOT NULL,
  `intern_id` char(36) NOT NULL,
  `supervisor_id` char(36) NOT NULL,
  `title` varchar(255) NOT NULL,
  `description` text DEFAULT NULL,
  `status` enum('pending','in_progress','completed','rejected') DEFAULT 'pending',
  `deadline` date DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `submission_file` varchar(255) DEFAULT NULL,
  `feedback` text DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `tasks`
--

INSERT INTO `tasks` (`id`, `intern_id`, `supervisor_id`, `title`, `description`, `status`, `deadline`, `created_at`, `submission_file`, `feedback`) VALUES
('61ff2901-23ce-49e6-aa07-37e049837b56', '29ce8078-3ee7-11f1-b558-4ec2e12205b2', '1a538468-edf1-4d27-b102-c44c05720415', 'buat aplikasi berbasis website menggunakan bahasa pemrograman rust ', 'haha', '', '2026-06-04', '2026-05-20 07:51:07', 'uploads/tasks/bfabf99d-b938-4b32-a913-c20a5b28640b-TATIB 2025 OK.docx', NULL),
('a12bbb72-efbd-46aa-b3e0-804957eb4ee3', '30e98192-3ee7-11f1-b558-4ec2e12205b2', '1a538468-edf1-4d27-b102-c44c05720415', 'buat aplikasi berbasis website menggunakan bahasa pemrograman rust ', 'hhaya', '', '2026-05-30', '2026-05-20 07:52:31', 'uploads/tasks/c5b9910a-ed77-4c57-931e-0278e504eae6-bfabf99d-b938-4b32-a913-c20a5b28640b-TATIB 2025 OK.docx', NULL);

-- --------------------------------------------------------

--
-- Struktur dari tabel `users`
--

CREATE TABLE `users` (
  `id` char(36) NOT NULL,
  `email` varchar(255) NOT NULL,
  `password_hash` varchar(255) NOT NULL,
  `role` enum('admin','supervisor','intern','superadmin') NOT NULL,
  `full_name` varchar(100) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data untuk tabel `users`
--

INSERT INTO `users` (`id`, `email`, `password_hash`, `role`, `full_name`, `created_at`, `updated_at`) VALUES
('1a538468-edf1-4d27-b102-c44c05720415', 'salim@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$jCqPJUYFP0JAjXe8O1rnZg$R8pZ7yWdHBwlvJmpKU0RLPeYZXVr9KIOpq+UyPxJPL0', 'superadmin', 'salim', '2026-04-15 06:25:17', '2026-05-28 15:48:57'),
('33c92461-768b-4ec1-957f-5a7f325ef3c6', 'rizqysalim77@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$NufSttUjVHomyMErLdQdRA$H+mWTgc4PP16/wBrR/I8NmjI3RuY7viWiF9osRAXCvs', 'intern', 'Rizqy Agus Salim', '2026-04-23 05:42:22', '2026-05-26 09:47:39'),
('aa10b59e-946f-4514-8446-453f42e6007c', 'hehe@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$vdnc+sRHKTTkwkzXei7PEA$MXWo8GJhICMv6U4VdCxf6vUMpfPQ1gUhTq9tcQVoeqg', 'intern', 'Muhammad Jawir', '2026-04-16 06:27:54', '2026-05-28 15:49:48'),
('aafc8af4-c479-437f-a2f1-e2362bf4a706', '123456789@xcode.local', '$argon2id$v=19$m=19456,t=2,p=1$4J/SfCR1WKqB/+JuMf8TXg$I4iPRkjVtkUewIuizGcCeJzjOYlYt6kwZ04N2rc4Myc', 'intern', 'Nandito putra siagian', '2026-05-21 17:55:25', '2026-05-21 17:55:25'),
('b5f75255-d713-4b79-9756-e838b7bf0e8e', 'admin@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$9bBUlhQJXOXGD9G//YrHxA$juBNzuJAY4GvEeIw0AKNI/DEHyS3SlMx4Nfw8v/MAis', 'supervisor', 'Admin', '2026-04-15 06:27:01', '2026-04-15 06:27:01');

--
-- Indexes for dumped tables
--

--
-- Indeks untuk tabel `attendances`
--
ALTER TABLE `attendances`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `unique_intern_date` (`intern_id`,`date`),
  ADD KEY `fk_att_user` (`confirmed_by`);

--
-- Indeks untuk tabel `divisions`
--
ALTER TABLE `divisions`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `name` (`name`);

--
-- Indeks untuk tabel `documents`
--
ALTER TABLE `documents`
  ADD PRIMARY KEY (`id`),
  ADD KEY `fk_documents_user` (`user_id`);

--
-- Indeks untuk tabel `evaluations`
--
ALTER TABLE `evaluations`
  ADD PRIMARY KEY (`id`),
  ADD KEY `fk_evaluations_intern` (`intern_id`),
  ADD KEY `fk_evaluations_supervisor` (`supervisor_id`);

--
-- Indeks untuk tabel `interns`
--
ALTER TABLE `interns`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `user_id` (`user_id`),
  ADD KEY `idx_user_id` (`user_id`),
  ADD KEY `idx_status` (`status`);

--
-- Indeks untuk tabel `intern_jadwal`
--
ALTER TABLE `intern_jadwal`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `unique_intern_jadwal` (`intern_id`,`jadwal_id`),
  ADD KEY `fk_intern_jadwal_jadwal` (`jadwal_id`);

--
-- Indeks untuk tabel `jadwal_magang`
--
ALTER TABLE `jadwal_magang`
  ADD PRIMARY KEY (`id`);

--
-- Indeks untuk tabel `logbooks`
--
ALTER TABLE `logbooks`
  ADD PRIMARY KEY (`id`),
  ADD KEY `fk_logbooks_intern` (`intern_id`);

--
-- Indeks untuk tabel `password_resets`
--
ALTER TABLE `password_resets`
  ADD PRIMARY KEY (`id`),
  ADD KEY `idx_email` (`email`);

--
-- Indeks untuk tabel `project_proposals`
--
ALTER TABLE `project_proposals`
  ADD PRIMARY KEY (`id`),
  ADD KEY `fk_proposal_reviewer` (`reviewed_by`),
  ADD KEY `idx_proposal_intern` (`intern_id`),
  ADD KEY `idx_proposal_status` (`status`);

--
-- Indeks untuk tabel `tasks`
--
ALTER TABLE `tasks`
  ADD PRIMARY KEY (`id`),
  ADD KEY `fk_tasks_intern` (`intern_id`),
  ADD KEY `fk_tasks_supervisor` (`supervisor_id`);

--
-- Indeks untuk tabel `users`
--
ALTER TABLE `users`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `email` (`email`),
  ADD KEY `idx_email` (`email`),
  ADD KEY `idx_role` (`role`);

--
-- AUTO_INCREMENT untuk tabel yang dibuang
--

--
-- AUTO_INCREMENT untuk tabel `divisions`
--
ALTER TABLE `divisions`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=7;

--
-- Ketidakleluasaan untuk tabel pelimpahan (Dumped Tables)
--

--
-- Ketidakleluasaan untuk tabel `attendances`
--
ALTER TABLE `attendances`
  ADD CONSTRAINT `fk_att_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE,
  ADD CONSTRAINT `fk_att_user` FOREIGN KEY (`confirmed_by`) REFERENCES `users` (`id`) ON DELETE SET NULL;

--
-- Ketidakleluasaan untuk tabel `documents`
--
ALTER TABLE `documents`
  ADD CONSTRAINT `fk_documents_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE;

--
-- Ketidakleluasaan untuk tabel `evaluations`
--
ALTER TABLE `evaluations`
  ADD CONSTRAINT `fk_evaluations_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE,
  ADD CONSTRAINT `fk_evaluations_supervisor` FOREIGN KEY (`supervisor_id`) REFERENCES `users` (`id`) ON DELETE CASCADE;

--
-- Ketidakleluasaan untuk tabel `interns`
--
ALTER TABLE `interns`
  ADD CONSTRAINT `fk_interns_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Ketidakleluasaan untuk tabel `intern_jadwal`
--
ALTER TABLE `intern_jadwal`
  ADD CONSTRAINT `fk_intern_jadwal_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `fk_intern_jadwal_jadwal` FOREIGN KEY (`jadwal_id`) REFERENCES `jadwal_magang` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Ketidakleluasaan untuk tabel `logbooks`
--
ALTER TABLE `logbooks`
  ADD CONSTRAINT `fk_logbooks_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE;

--
-- Ketidakleluasaan untuk tabel `project_proposals`
--
ALTER TABLE `project_proposals`
  ADD CONSTRAINT `fk_proposal_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `fk_proposal_reviewer` FOREIGN KEY (`reviewed_by`) REFERENCES `users` (`id`) ON DELETE SET NULL ON UPDATE CASCADE;

--
-- Ketidakleluasaan untuk tabel `tasks`
--
ALTER TABLE `tasks`
  ADD CONSTRAINT `fk_tasks_intern` FOREIGN KEY (`intern_id`) REFERENCES `interns` (`id`) ON DELETE CASCADE,
  ADD CONSTRAINT `fk_tasks_supervisor` FOREIGN KEY (`supervisor_id`) REFERENCES `users` (`id`) ON DELETE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
