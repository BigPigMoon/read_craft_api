-- Add up migration script here
CREATE TABLE IF NOT EXISTS books (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    language language NOT NULL,
    filename TEXT NOT NULL,
    cover_path TEXT,
    subject TEXT,
    author TEXT,
    progress INTEGER NOT NULL DEFAULT 0
);