-- Add up migration script here
CREATE TABLE IF NOT EXISTS book_user (
    id SERIAL PRIMARY KEY,
    chunk INTEGER NOT NULL DEFAULT 0,
    book_id INT REFERENCES books(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE
);