-- Add migration script here
-- Create table for subscriptions
CREATE TABLE
    subscriptions (
        id uuid NOT NULL,
        PRIMARY KEY (id),
        email TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        subscribed_at timestamptz NOT NULL
    );