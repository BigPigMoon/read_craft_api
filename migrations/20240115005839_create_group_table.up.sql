-- Add up migration script here
CREATE TABLE IF NOT EXISTS card_group (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    invite_code TEXT NOT NULL,
    group_id INT REFERENCES card_group(id) ON DELETE CASCADE
);