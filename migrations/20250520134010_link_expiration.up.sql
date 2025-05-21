-- Add up migration script here
ALTER TABLE links ADD column IF NOT EXISTS expires_at timestamp DEFAULT null ;
