-- Add down migration script here
ALTER TABLE links DROP COLUMN IF EXISTS expires_at CASCADE ;
