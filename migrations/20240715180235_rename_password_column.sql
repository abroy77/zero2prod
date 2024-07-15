-- Add migration script here
ALTER TABLE users
RENAME PASSWORD TO password_hash;