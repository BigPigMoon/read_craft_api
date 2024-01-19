-- Add up migration script here
CREATE TABLE IF NOT EXISTS group_user (
    id SERIAL PRIMARY KEY,
    group_id INT REFERENCES card_group(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE
)