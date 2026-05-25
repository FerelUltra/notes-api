-- Add migration script here
ALTER TABLE users
ADD COLUMN token_version INTEGER NOT NULL DEFAULT 0;