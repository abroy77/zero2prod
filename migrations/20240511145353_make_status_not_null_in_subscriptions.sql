-- Add migration script here
-- Added in a transaction so it fails or succeeds as a whole
BEGIN;

UPDATE subscriptions
SET
    status = 'confirmed'
WHERE
    status IS NULL;

-- make status mandatory
ALTER TABLE subscriptions
ALTER COLUMN status
SET NOT NULL;

COMMIT;