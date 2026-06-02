-- Migration: Fix swapped user_id in interns table
-- Problem: user_id values are swapped between the two intern records,
-- causing the wrong intern name to appear on project proposals.
--
-- Current (WRONG):
--   Intern 29ce8078 (nama_lengkap='Rizqy Agus Salim')  → user_id aa10b59e (user full_name='Muhammad Jawir')
--   Intern 30e98192 (nama_lengkap='Muhammad Jawir')     → user_id 33c92461 (user full_name='Rizqy Agus Saliem')
--
-- After fix (CORRECT):
--   Intern 29ce8078 (nama_lengkap='Rizqy Agus Salim')  → user_id 33c92461 (user full_name='Rizqy Agus Saliem')
--   Intern 30e98192 (nama_lengkap='Muhammad Jawir')     → user_id aa10b59e (user full_name='Muhammad Jawir')

-- Step 1: Set temporary placeholder on intern Rizqy to avoid UNIQUE constraint violation
UPDATE interns SET user_id = 'TEMP_PLACEHOLDER' WHERE id = '29ce8078-3ee7-11f1-b558-4ec2e12205b2';

-- Step 2: Assign Jawir's user_id (aa10b59e) to Jawir's intern record (30e98192)
UPDATE interns SET user_id = 'aa10b59e-946f-4514-8446-453f42e6007c' WHERE id = '30e98192-3ee7-11f1-b558-4ec2e12205b2';

-- Step 3: Assign Rizqy's user_id (33c92461) to Rizqy's intern record (29ce8078)
UPDATE interns SET user_id = '33c92461-768b-4ec1-957f-5a7f325ef3c6' WHERE id = '29ce8078-3ee7-11f1-b558-4ec2e12205b2';
