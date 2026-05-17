-- Add migration script here
DELETE FROM users
WHERE email IS NULL OR password_hash IS NULL;

ALTER TABLE users
ALTER COLUMN email SET NOT NULL;

ALTER TABLE users
ALTER COLUMN password_hash SET NOT NULL;